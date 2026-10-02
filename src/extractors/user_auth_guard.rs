use std::sync::Arc;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::Key;
use axum_extra::extract::PrivateCookieJar;
use serde::de::DeserializeOwned;
use uuid::Uuid;
use crate::services::helpers::response::ApiResponse;
use crate::services::security::tokens::jwt::JwtService;

#[derive(Clone)]
pub struct AuthenticatedUser<T> {
    pub id: Uuid,
    pub custom: T,
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

impl<S, T> FromRequestParts<S> for AuthenticatedUser<T>
where
    S: Send + Sync,
    Arc<JwtService>: FromRef<S>,
    Key: FromRef<S>,
    T: DeserializeOwned + Send + Sync + 'static,
{
    type Rejection = TokenError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jwt_svc = Arc::<JwtService>::from_ref(state);

        let jar = PrivateCookieJar::<Key>::from_request_parts(parts, state)
            .await
            .map_err(|_| TokenError::Missing)?;

        let cookie = jar.get("jwt").ok_or(TokenError::Missing)?;

        let token_data = jwt_svc
            .verify::<T>(cookie.value())
            .map_err(|_| TokenError::Invalid)?;

        Ok(AuthenticatedUser {
            id: token_data.sub,
            custom: token_data.custom,
        })
    }
}