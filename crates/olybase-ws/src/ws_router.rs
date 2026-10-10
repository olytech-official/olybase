use std::collections::HashMap;
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use olybase_core::user::UserData;
use crate::gateway_service::GatewayService;

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

#[derive(Debug, Deserialize)]
pub struct GatewayEvent {
    /// Event name
    pub event: String,
    #[serde(default)]
    /// Payload
    pub payload: Value,
}

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("Not a member of channel")]
    NotMember,

    #[error("Unknown event type: {0}")]
    UnknownEvent(String),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] serde_json::Error),
}

impl HandlerError {
    pub fn to_ws_frame(&self) -> String {
        serde_json::json!({
            "type": "error",
            "message": self.to_string()
        })
        .to_string()
    }
}

pub type ErasedWsHandler<T, S> = Arc<
    dyn Fn(Value, Arc<GatewayService<S>>, UserData<T, S>) -> BoxFuture<Result<Option<String>, HandlerError>>
    + Send
    + Sync,
>;

#[derive(Clone)]
pub struct WsRouter<T, S>
where
    S: Eq + Hash + Clone + Send + Sync + 'static,
{
    routes: HashMap<String, ErasedWsHandler<T, S>>,
}

impl<T, S> Default for WsRouter<T, S>
where
    S: Eq + Hash + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }
}

impl<T, S> WsRouter<T, S>
where
    S: Eq + Hash + Clone + Send + Sync + 'static,
    T: Send + Sync + Clone + 'static,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on<P, F, Fut>(mut self, event_type: &str, handler: F) -> Self
    where
        P: DeserializeOwned + Send + 'static,
        F: Fn(P, Arc<GatewayService<S>>, UserData<T, S>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Option<String>, HandlerError>> + Send + 'static,
    {
        let handler = Arc::new(handler);

        let erased: ErasedWsHandler<T, S> = Arc::new(move |raw_payload, gateway, user| {
            let handler = Arc::clone(&handler);

            Box::pin(async move {
                let payload: P = serde_json::from_value(raw_payload)?;
                handler(payload, gateway, user).await
            })
        });

        self.routes.insert(event_type.to_string(), erased);
        self
    }

    pub fn merge(mut self, other: Self) -> Self {
        for (event_type, handler) in other.routes {
            if self.routes.contains_key(&event_type) {
                panic!("WS Event Route conflict: '{event_type}' is already registered!");
            }
            self.routes.insert(event_type, handler);
        }
        self
    }

    pub async fn dispatch(
        &self,
        event_data: GatewayEvent,
        gateway: Arc<GatewayService<S>>,
        user: UserData<T, S>,
    ) -> Result<Option<String>, HandlerError> {
        let handler = self
            .routes
            .get(&event_data.event)
            .ok_or_else(|| HandlerError::UnknownEvent(event_data.event))?;

        handler(event_data.payload, gateway, user).await
    }

    pub async fn handle_message(
        &self,
        user: UserData<T, S>,
        text: &str,
        gateway: &Arc<GatewayService<S>>,
    ) -> Result<Option<String>, HandlerError> {
        let event = match parse_event(text) {
            Ok(evt) => evt,
            Err(err) => {
                gateway.broadcast_to_user(&user.id, err.to_ws_frame()).await;
                return Err(err);
            }
        };

        match self.dispatch(event, gateway.clone(), user.clone()).await {
            Ok(Some(response)) => {
                gateway.broadcast_to_user(&user.id, response.clone()).await;
                Ok(Some(response))
            }
            Ok(None) => Ok(None),
            Err(err) => {
                gateway.broadcast_to_user(&user.id, err.to_ws_frame()).await;
                Err(err)
            }
        }
    }
}

pub fn parse_event(text: &str) -> Result<GatewayEvent, HandlerError> {
    serde_json::from_str(text).map_err(HandlerError::Deserialization)
}