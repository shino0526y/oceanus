mod create_user;
mod list_users;
mod update_user;

use self::{
    create_user::{CreateUserInput, CreateUserOutput},
    list_users::ListUsersOutputElement,
    update_user::{UpdateUserInput, UpdateUserOutput},
};
use crate::{
    AppState,
    internal::{
        application::user::{
            create_user_use_case::CreateUserError,
            update_user_use_case::{UpdateUserCommand, UpdateUserError},
        },
        domain::value_object::{Id, Role},
        presentation::error::PresentationError,
    },
};
use axum::{
    Json,
    extract::{Path, State},
};

pub async fn create_user(
    State(state): State<AppState>,
    Json(input): Json<CreateUserInput>,
) -> Result<Json<CreateUserOutput>, PresentationError> {
    let id =
        Id::new(input.id).map_err(|e| PresentationError::BadRequest(format!("無効なID: {}", e)))?;
    let role = Role::from_i16(input.role as i16).map_err(PresentationError::BadRequest)?;

    let user = state
        .create_user_use_case
        .create_user(id, input.name, role, input.password)
        .await
        .map_err(|e| match e {
            CreateUserError::PasswordHashError(msg) => PresentationError::InternalServerError(
                format!("パスワードのハッシュ化に失敗しました: {}", msg),
            ),
            CreateUserError::Repository(e) => PresentationError::from(e),
        })?;

    Ok(Json(CreateUserOutput::from(user)))
}

pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<ListUsersOutputElement>>, PresentationError> {
    let output = state
        .list_users_use_case
        .list_users()
        .await
        .map(|users| {
            users
                .into_iter()
                .map(ListUsersOutputElement::from)
                .collect()
        })
        .map_err(PresentationError::from)?;

    Ok(Json(output))
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateUserInput>,
) -> Result<Json<UpdateUserOutput>, PresentationError> {
    // バリデーション
    let old_id =
        Id::new(id).map_err(|e| PresentationError::BadRequest(format!("無効なID: {}", e)))?;
    let new_id =
        Id::new(input.id).map_err(|e| PresentationError::BadRequest(format!("無効なID: {}", e)))?;
    let role = Role::from_i16(input.role).map_err(PresentationError::BadRequest)?;

    let command = UpdateUserCommand::new(new_id, input.name, role, input.password);

    let user = state
        .update_user_use_case
        .update_user(&old_id, command)
        .await
        .map_err(|e| match e {
            UpdateUserError::PasswordHashError(msg) => PresentationError::InternalServerError(
                format!("パスワードのハッシュ化に失敗しました: {}", msg),
            ),
            UpdateUserError::Repository(e) => PresentationError::from(e),
        })?;

    Ok(Json(UpdateUserOutput::from(user)))
}
