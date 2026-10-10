use dashmap::DashMap;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use std::hash::Hash;

#[derive(Clone)]
pub struct GatewayService<T>
where
    T: Eq + Hash + Clone,
{
    users: DashMap<T, Sender<String>>,
}

impl<T: Eq + Hash + Clone> GatewayService<T> {
    pub fn new() -> Self {
        Self {
            users: DashMap::new(),
        }
    }

    pub async fn get_or_create(&self, user_id: &T) -> Sender<String> {
        self.users
            .entry(user_id.clone())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);
                tx
            })
            .clone()
    }

    pub async fn broadcast_to_user(&self, user_id: &T, payload: String) {
        if let Some(tx) = self.users.get(user_id) {
            let _ = tx.send(payload);
        }
    }

    pub fn cleanup(&self) {
        self.users.retain(|_, tx| tx.receiver_count() > 0);
    }
}