use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserInput {
    pub id: String,
    pub name: String,
    pub role: u8,
    pub password: String,
}
