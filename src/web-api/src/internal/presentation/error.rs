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
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    UnprocessableContent(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    InternalServerError(String),
}

impl IntoResponse for PresentationError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            PresentationError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            PresentationError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            PresentationError::UnprocessableContent(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            PresentationError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            PresentationError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

impl From<RepositoryError> for PresentationError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::Conflict { .. } => PresentationError::Conflict(error.to_string()),
            RepositoryError::NotFound { .. } => PresentationError::NotFound(error.to_string()),
            RepositoryError::Other { .. } => {
                PresentationError::InternalServerError(error.to_string())
            }
        }
    }
}
