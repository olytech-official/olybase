/// user_auth_guard and websocket uses this struct
#[derive(Debug, Clone)]
pub struct UserData<T, ID> {
    pub id: ID,
    pub custom: T,
}