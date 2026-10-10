use std::ops::Deref;
use std::sync::Arc;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::Key;
use axum_extra::extract::PrivateCookieJar;
use serde::de::DeserializeOwned;
use olybase_core::response::ApiResponse;
use olybase_core::user::UserData;
use crate::tokens::jwt::JwtService;

// #[derive(Clone)]
// pub struct AuthenticatedUser<T, ID = Uuid> {
//     pub id: ID,
//     pub custom: T,
// }

// pub type UuidUser<T> = AuthenticatedUser<T, Uuid>;
// pub type SnowflakeUser<T> = AuthenticatedUser<T, i64>;

#[derive(Clone)]
pub struct AuthenticatedUser<T, ID>(pub UserData<T, ID>);

impl<T, ID> Deref for AuthenticatedUser<T, ID> {
    type Target = UserData<T, ID>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub enum TokenError {
    Missing,
    Invalid,
    ServiceMissing,
}

impl IntoResponse for TokenError {
    fn into_response(self) -> Response {
        match self {
            TokenError::Missing => {
                ApiResponse::unauthorized("Authentication token is missing").into_response()
            }
            TokenError::Invalid => {
                ApiResponse::unauthorized("Invalid or expired authentication token").into_response()
            }
            TokenError::ServiceMissing => {
                ApiResponse::internal_server_error("Authentication services is unavailable").into_response()
            }
        }
    }
}

impl<S, T, ID> FromRequestParts<S> for AuthenticatedUser<T, ID>
where
    S: Send + Sync,
    Arc<JwtService>: FromRef<S>,
    Key: FromRef<S>,
    T: DeserializeOwned + Send + Sync + 'static,
    ID: DeserializeOwned + Send + Sync + 'static,
{
    type Rejection = TokenError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jwt_svc = Arc::<JwtService>::from_ref(state);

        let jar = PrivateCookieJar::<Key>::from_request_parts(parts, state)
            .await
            .map_err(|_| TokenError::Missing)?;

        let cookie = jar.get("jwt").ok_or(TokenError::Missing)?;

        let token_data = jwt_svc
            .verify::<T, ID>(cookie.value())
            .map_err(|_| TokenError::Invalid)?;

        Ok(AuthenticatedUser(UserData {
            id: token_data.sub,
            custom: token_data.custom,
        }))
    }
}