/// user_auth_guard and websocket uses this struct
#[derive(Clone)]
pub struct UserData<T, ID> {
    pub id: ID,
    pub custom: T,
}