use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateUserInput {
    pub id: String,
    pub name: String,
    pub role: i16,
    pub password: String,
}
