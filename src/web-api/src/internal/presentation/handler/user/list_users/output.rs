use crate::internal::application::user::list_users_use_case::UserWithLoginFailureCount;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListUsersOutputElement {
    pub id: String,
    pub name: String,
    pub role: i16,
    pub login_failure_count: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserWithLoginFailureCount> for ListUsersOutputElement {
    fn from(data: UserWithLoginFailureCount) -> Self {
        Self {
            id: data.user.id().value().into(),
            name: data.user.name().value().into(),
            role: data.user.role().as_i16(),
            login_failure_count: data.login_failure_count,
            created_at: *data.user.created_at(),
            updated_at: *data.user.updated_at(),
        }
    }
}
