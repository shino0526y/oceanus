mod input;
mod output;

pub use self::{input::UpdateUserInput, output::UpdateUserOutput};

use crate::{
    AppState,
    internal::{
        application::user::update_user_use_case::{UpdateUserCommand, UpdateUserError},
        domain::value_object::{Id, Role},
        presentation::{error::PresentationError, middleware::AuthenticatedUser},
    },
};
use axum::{
    Extension, Json,
    extract::{Path, State},
};
use chrono::Utc;

#[utoipa::path(
    put,
    path = "/users/{id}",
    request_body = UpdateUserInput,
    params(
        ("id" = String, Path, description = "ユーザーID")
    ),
    responses(
        (status = 200, description = "ユーザーの更新に成功", body = UpdateUserOutput),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "CSRFトークンが無効"),
        (status = 422, description = "バリデーションに失敗"),
    ),
    security(
        ("session_cookie" = []),
        ("csrf_token" = [])
    ),
    tag = "users"
)]
pub async fn update_user(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
    Json(input): Json<UpdateUserInput>,
) -> Result<Json<UpdateUserOutput>, PresentationError> {
    // バリデーション
    let old_id = Id::new(id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なID: {}", e)))?;
    let new_id = Id::new(input.id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なID: {}", e)))?;
    let role = Role::from_i16(input.role).map_err(PresentationError::UnprocessableContent)?;

    // 更新処理
    let command = UpdateUserCommand {
        old_id,
        id: new_id,
        name: input.name,
        role,
        password: input.password,
        updated_by: user.uuid(),
        updated_at: Utc::now(),
    };
    let user = state
        .update_user_use_case
        .execute(command)
        .await
        .map_err(|e| match e {
            UpdateUserError::PasswordHashError(msg) => PresentationError::InternalServerError(
                format!("パスワードのハッシュ化に失敗しました: {}", msg),
            ),
            UpdateUserError::Repository(e) => PresentationError::from(e),
        })?;

    Ok(Json(UpdateUserOutput::from(user)))
}
