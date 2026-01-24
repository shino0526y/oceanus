mod input;
mod output;

pub use self::{input::CreateUserInput, output::CreateUserOutput};

use crate::{
    AppState,
    internal::{
        application::user::create_user_use_case::{CreateUserCommand, CreateUserError},
        domain::value_object::{Id, Role},
        presentation::{error::PresentationError, middleware::AuthenticatedUser},
    },
};
use axum::{Extension, Json, extract::State};
use chrono::Utc;

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserInput,
    responses(
        (status = 200, description = "ユーザーの作成に成功", body = CreateUserOutput),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "CSRFトークンが無効または権限がありません"),
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
    Extension(user): Extension<AuthenticatedUser>,
    Json(input): Json<CreateUserInput>,
) -> Result<Json<CreateUserOutput>, PresentationError> {
    // バリデーション
    let id = Id::new(input.id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なID: {}", e)))?;
    let role = Role::from_i16(input.role).map_err(PresentationError::UnprocessableContent)?;

    // 登録処理
    let command = CreateUserCommand {
        id,
        name: input.name,
        role,
        password: input.password,
        created_by: user.uuid(),
        created_at: Utc::now(),
    };
    let user = state
        .create_user_use_case
        .execute(command)
        .await
        .map_err(|e| match e {
            CreateUserError::EmptyPassword => {
                PresentationError::UnprocessableContent(e.to_string())
            }
            CreateUserError::PasswordHashError(msg) => PresentationError::InternalServerError(
                format!("パスワードのハッシュ化に失敗しました: {}", msg),
            ),
            CreateUserError::Repository(e) => PresentationError::from(e),
        })?;

    Ok(Json(CreateUserOutput::from(user)))
}
