use crate::internal::domain::error::RepositoryError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PresentationError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    InternalServerError(String),
}

impl IntoResponse for PresentationError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            PresentationError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            PresentationError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            PresentationError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

impl From<RepositoryError> for PresentationError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::Duplicate { .. } => PresentationError::Conflict(error.to_string()),
            RepositoryError::Other { .. } => {
                PresentationError::InternalServerError(error.to_string())
            }
        }
    }
}
