use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequestBody {
    pub id: String,
    pub name: String,
    pub role: i16,
    pub password: String,
}
