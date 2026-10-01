use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T = ()> {
    pub status: String,
    pub status_code: u16,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (status, Json(self)).into_response()
    }
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(message: &str, data: Option<T>) -> Self {
        Self {
            status: "success".to_string(),
            status_code: StatusCode::OK.as_u16(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data,
            message: Some(message.to_string()),
        }
    }

    pub fn success_with_code(message: &str, data: Option<T>, code: StatusCode) -> Self {
        Self {
            status: "success".to_string(),
            status_code: code.as_u16(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data,
            message: Some(message.to_string()),
        }
    }
}

impl ApiResponse<()> {
    pub fn error(code: StatusCode, message: &str) -> Self {
        Self {
            status: "error".to_string(),
            status_code: code.as_u16(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: None,
            message: Some(message.to_string()),
        }
    }

    pub fn bad_request(message: &str) -> Self {
        Self::error(StatusCode::BAD_REQUEST, message)
    }

    pub fn unauthorized(message: &str) -> Self {
        Self::error(StatusCode::UNAUTHORIZED, message)
    }

    pub fn forbidden(message: &str) -> Self {
        Self::error(StatusCode::FORBIDDEN, message)
    }

    pub fn not_found(message: &str) -> Self {
        Self::error(StatusCode::NOT_FOUND, message)
    }

    pub fn conflict(message: &str) -> Self {
        Self::error(StatusCode::CONFLICT, message)
    }

    pub fn unprocessable(message: &str) -> Self {
        Self::error(StatusCode::UNPROCESSABLE_ENTITY, message)
    }

    pub fn too_many_requests(message: &str) -> Self {
        Self::error(StatusCode::TOO_MANY_REQUESTS, message)
    }

    pub fn internal_server_error(message: &str) -> Self {
        Self::error(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    pub fn bad_gateway(message: &str) -> Self {
        Self::error(StatusCode::BAD_GATEWAY, message)
    }
}