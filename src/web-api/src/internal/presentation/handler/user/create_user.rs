mod input;
mod output;

pub use self::{input::CreateUserInput, output::CreateUserOutput};

use crate::{
    AppState,
    internal::{
        application::user::create_user_use_case::CreateUserError,
        domain::value_object::{Id, Role},
        presentation::error::PresentationError,
    },
};
use axum::{Json, extract::State};

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserInput,
    responses(
        (status = 200, description = "ユーザーの作成に成功", body = CreateUserOutput),
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
pub async fn create_user(
    State(state): State<AppState>,
    Json(input): Json<CreateUserInput>,
) -> Result<Json<CreateUserOutput>, PresentationError> {
    let id = Id::new(input.id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なID: {}", e)))?;
    let role =
        Role::from_i16(input.role as i16).map_err(PresentationError::UnprocessableContent)?;

    let user = state
        .create_user_use_case
        .execute(id, input.name, role, input.password)
        .await
        .map_err(|e| match e {
            CreateUserError::PasswordHashError(msg) => PresentationError::InternalServerError(
                format!("パスワードのハッシュ化に失敗しました: {}", msg),
            ),
            CreateUserError::Repository(e) => PresentationError::from(e),
        })?;

    Ok(Json(CreateUserOutput::from(user)))
}
