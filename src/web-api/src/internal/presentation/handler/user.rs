mod list_users;

use self::list_users::ListUsersOutputElement;
use crate::{AppState, internal::presentation::error::PresentationError};
use axum::{Json, extract::State};

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
