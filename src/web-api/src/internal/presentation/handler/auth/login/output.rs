use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginOutput {
    pub user_id: String,
    pub csrf_token: String,
    pub role: i16,
}
