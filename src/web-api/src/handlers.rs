use crate::models::{ApplicationEntity, CreateApplicationEntity};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sqlx::{Pool, Postgres};

pub async fn list_application_entities(
    State(pool): State<Pool<Postgres>>,
) -> Result<Json<Vec<ApplicationEntity>>, ApiError> {
    let aes = sqlx::query_as::<_, ApplicationEntity>(
        "SELECT title, host, port, comment, created_at, updated_at, deleted_at
         FROM application_entities
         WHERE deleted_at IS NULL
         ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;

    Ok(Json(aes))
}

pub async fn create_application_entity(
    State(pool): State<Pool<Postgres>>,
    Json(payload): Json<CreateApplicationEntity>,
) -> Result<Json<ApplicationEntity>, ApiError> {
    // バリデーション
    if payload.title.is_empty() || payload.title.len() > 16 {
        return Err(ApiError::Validation(
            "titleは1文字以上16文字以内で指定してください".to_string(),
        ));
    }
    if payload.host.is_empty() {
        return Err(ApiError::Validation("hostを指定してください".to_string()));
    }
    if payload.port < 1 || payload.port > 65535 {
        return Err(ApiError::Validation(
            "portは1〜65535の範囲で指定してください".to_string(),
        ));
    }

    let ae = sqlx::query_as::<_, ApplicationEntity>(
        "INSERT INTO application_entities (title, host, port, comment)
         VALUES ($1, $2, $3, $4)
         RETURNING title, host, port, comment, created_at, updated_at, deleted_at",
    )
    .bind(&payload.title)
    .bind(&payload.host)
    .bind(payload.port)
    .bind(&payload.comment)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if let Some(db_err) = e.as_database_error() {
            if db_err.is_unique_violation() {
                return ApiError::Conflict(format!(
                    "AEタイトル'{}'は既に登録されています",
                    payload.title
                ));
            }
        }
        ApiError::Database(e.to_string())
    })?;

    Ok(Json(ae))
}

#[derive(Debug)]
pub enum ApiError {
    Database(String),
    Validation(String),
    Conflict(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
