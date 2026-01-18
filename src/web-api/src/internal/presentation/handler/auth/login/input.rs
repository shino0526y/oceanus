use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginInput {
    pub user_id: String,
    pub password: String,
}
