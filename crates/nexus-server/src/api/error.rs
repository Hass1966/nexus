use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use nexus_common::error::NexusError;

use crate::models::responses::ErrorResponse;

/// Wrapper so we can implement IntoResponse for anyhow::Error.
pub struct AppError(pub anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self.0.downcast_ref::<NexusError>() {
            Some(NexusError::NotFound(msg)) => (StatusCode::NOT_FOUND, msg.clone()),
            Some(NexusError::Auth(msg)) => (StatusCode::UNAUTHORIZED, msg.clone()),
            Some(NexusError::Validation(msg)) => (StatusCode::BAD_REQUEST, msg.clone()),
            Some(NexusError::Llm(msg)) => (StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
            _ => {
                tracing::error!("Internal error: {:?}", self.0);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = Json(ErrorResponse {
            error: message,
            details: None,
        });

        (status, body).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
