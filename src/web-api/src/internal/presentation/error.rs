use crate::internal::domain::error::RepositoryError;
use axum::{
    Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

const DEFAULT_PROBLEM_TYPE: &str = "about:blank";

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponseBody {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
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
        let (status, detail) = match self {
            PresentationError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            PresentationError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            PresentationError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            PresentationError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            PresentationError::UnprocessableContent(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            PresentationError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            PresentationError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
        };

        let body = ErrorResponseBody {
            problem_type: DEFAULT_PROBLEM_TYPE.to_string(),
            title: status
                .canonical_reason()
                .unwrap_or("Unknown")
                .to_string(),
            status: status.as_u16(),
            detail,
        };

        (
            status,
            [(header::CONTENT_TYPE, "application/problem+json")],
            Json(body),
        )
            .into_response()
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
