use dashmap::DashMap;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use uuid::Uuid;

#[derive(Clone)]
pub struct GatewayService {
    users: DashMap<Uuid, Sender<String>>,
}

impl GatewayService {
    pub async fn new() -> Self {
        Self {
            users: DashMap::new(),
        }
    }

    pub async fn get_or_create(&self, user_id: &Uuid) -> Sender<String> {
        self.users
            .entry(*user_id)
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);
                tx
            })
            .clone()
    }

    pub async fn broadcast_to_user(&self, user_id: Uuid, payload: String) {
        if let Some(tx) = self.users.get(&user_id) {
            let _ = tx.send(payload);
        }
    }

    pub async fn cleanup(&self) {
        self.users.retain(|_, tx| tx.receiver_count() > 0);
    }
}