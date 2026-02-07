use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponseBody {
    pub user_id: String,
    pub csrf_token: String,
    pub role: i16,
}
