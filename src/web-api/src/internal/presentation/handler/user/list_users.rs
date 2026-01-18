mod output;

pub use output::ListUsersOutputElement;

use crate::{AppState, internal::presentation::error::PresentationError};
use axum::{Json, extract::State};

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "ユーザーの一覧の取得に成功", body = Vec<ListUsersOutputElement>),
        (status = 401, description = "セッションが確立されていない"),
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "users"
)]
pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<ListUsersOutputElement>>, PresentationError> {
    let output = state
        .list_users_use_case
        .execute()
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
