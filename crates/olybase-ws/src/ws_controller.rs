use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use axum::{
    extract::ws::{WebSocket},
};
use axum::extract::ws::Message;
use futures_util::{SinkExt, StreamExt};
use tokio::time::{interval_at, Instant};
use olybase_core::user::UserData;
use crate::gateway_service::GatewayService;
use crate::ws_router::WsRouter;

async fn handle_socket<T, ID>(
    socket: WebSocket,
    gateway: Arc<GatewayService<ID>>,
    ws_router: WsRouter<T, ID>,
    user: UserData<T, ID>,
) where
    T: Send + Sync + Clone + 'static,
    ID: Eq + Hash + Clone + Send + Sync + 'static,
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
                        let _ = ws_router.handle_message(user.clone(), &text, &gateway).await;
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

    gateway.cleanup();
}