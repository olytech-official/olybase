use std::marker::PhantomData;

#[derive(Clone)]
pub struct AppState<T> {
    pub user_state: T,
    pub _marker: PhantomData<fn() -> T>,
}

impl<T> AppState<T> {
    pub fn new(
        user_state: T,
    ) -> Self {
        Self {
            user_state,
            _marker: PhantomData,
        }
    }
}