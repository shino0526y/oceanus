use crate::{
    AppState,
    internal::{
        application::user::delete_user_use_case::{DeleteUserCommand, DeleteUserError},
        domain::value_object::Id,
        presentation::{error::PresentationError, middleware::AuthenticatedUser},
    },
};
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(
        ("id" = String, Path, description = "ユーザーID")
    ),
    responses(
        (status = 204, description = "ユーザーの削除に成功"),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "CSRFトークンが無効"),
        (status = 404, description = "ユーザーが見つからない"),
        (status = 422, description = "バリデーションに失敗、または自分自身を削除しようとした"),
    ),
    security(
        ("session_cookie" = []),
        ("csrf_token" = [])
    ),
    tag = "users"
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, PresentationError> {
    // バリデーション
    let id = Id::new(id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なID: {}", e)))?;

    // 削除処理
    let deleted_at = Utc::now();
    let command = DeleteUserCommand {
        id,
        deleted_by: user.uuid(),
        deleted_at,
    };
    state
        .delete_user_use_case
        .execute(command)
        .await
        .map_err(|e| match e {
            DeleteUserError::CannotDeleteSelf => {
                PresentationError::UnprocessableContent(e.to_string())
            }
            DeleteUserError::Repository(repo_err) => PresentationError::from(repo_err),
        })?;

    Ok(StatusCode::NO_CONTENT)
}
