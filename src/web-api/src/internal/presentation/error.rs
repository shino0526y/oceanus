use crate::internal::domain::error::RepositoryError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponseBody {
    pub error: String,
}

#[derive(Debug, Error)]
pub enum PresentationError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
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
            PresentationError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            PresentationError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            PresentationError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            PresentationError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            PresentationError::UnprocessableContent(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            PresentationError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            PresentationError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
        };

        (status, Json(ErrorResponseBody { error: message })).into_response()
    }
}

impl From<RepositoryError> for PresentationError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::NotFound { .. } => PresentationError::NotFound(error.to_string()),
            RepositoryError::Conflict { .. } => PresentationError::Conflict(error.to_string()),
            RepositoryError::Other { .. } => {
                PresentationError::InternalServerError(error.to_string())
            }
        }
    }
}
