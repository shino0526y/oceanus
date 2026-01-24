use crate::{
    AppState,
    internal::{
        application::user::reset_login_failure_count_use_case::{
            ResetLoginFailureCountCommand, ResetLoginFailureCountError,
        },
        domain::value_object::Id,
        presentation::error::PresentationError,
    },
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
};

#[utoipa::path(
    delete,
    path = "/users/{id}/login-failure-count",
    params(
        ("id" = String, Path, description = "ユーザーID")
    ),
    responses(
        (status = 204, description = "ログイン失敗回数のリセットに成功"),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "CSRFトークンが無効または権限がありません"),
        (status = 404, description = "ユーザーが見つからない"),
        (status = 422, description = "バリデーションに失敗"),
    ),
    security(
        ("session_cookie" = []),
        ("csrf_token" = [])
    ),
    tag = "users"
)]
pub async fn reset_login_failure_count(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, PresentationError> {
    // バリデーション
    let id = Id::new(id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なID: {}", e)))?;

    // リセット処理
    let command = ResetLoginFailureCountCommand { id };
    state
        .reset_login_failure_count_use_case
        .execute(command)
        .await
        .map_err(|e| match e {
            ResetLoginFailureCountError::Repository(repo_err) => PresentationError::from(repo_err),
        })?;

    Ok(StatusCode::NO_CONTENT)
}
