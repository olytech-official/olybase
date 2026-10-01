use std::sync::Arc;
use std::time::Duration;
use axum::{
    extract::State,
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
};
use axum::extract::ws::Message;
use futures_util::{SinkExt, StreamExt};
use tokio::time::{interval_at, Instant};
use crate::AppState;
use crate::extractors::user_auth_guard::AuthenticatedUser;
use crate::services::gateway::service::GatewayService;
use crate::services::gateway::ws_router::WsRouter;

pub async fn ws_handler<T: Send + 'static>(
    ws: WebSocketUpgrade,
    State(state): State<AppState<T>>,
    user: AuthenticatedUser<T>
) -> Response
where
    T: Send + Sync + Clone + 'static,
{
    ws.on_upgrade(move |socket| handle_socket(socket, state.gateway, state.ws_router, user))
}

async fn handle_socket<T>(
    socket: WebSocket,
    gateway: Arc<GatewayService>,
    ws_router: WsRouter<T>,
    user: AuthenticatedUser<T>
) where
    T: Send + Sync + Clone + 'static,
{
    let (mut sender, mut receiver) = socket.split();

    let tx = gateway.get_or_create(&user.id).await;
    let mut rx = tx.subscribe();

    let mut heartbeat = interval_at(Instant::now() + Duration::from_secs(30), Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                if sender.send(Message::Ping(vec![].into())).await.is_err() {
                    break;
                }
            }

            // incoming from user
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let _ = ws_router.handle_message(&user, &text, &gateway).await;
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }

            // outgoing gateway
            msg_from_gateway = rx.recv() => {
                match msg_from_gateway {
                    Ok(text) => {
                        if sender.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    drop(rx);
    drop(sender);
    drop(receiver);

    gateway.cleanup().await;
}