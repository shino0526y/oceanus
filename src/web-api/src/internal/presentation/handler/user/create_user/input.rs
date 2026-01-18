use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserInput {
    pub id: String,
    pub name: String,
    pub role: u8,
    pub password: String,
}
