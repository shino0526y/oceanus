#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "ヘルスチェックに成功"),
    ),
    tag = "health"
)]
pub async fn respond_if_healthy() {}
