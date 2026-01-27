use crate::internal::presentation::handler::auth::login::LoginOutput;
use axum::{
    Router,
    body::{self, Body},
    http::Request,
};
use serde_json::json;
use tower::ServiceExt;

pub async fn login(router: &Router, user_id: &str, password: &str) -> (String, String) {
    let payload = json!({
        "userId": user_id,
        "password": password,
    });
    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    let set_cookie = response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let session_id = set_cookie
        .split("session_id=")
        .nth(1)
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    let bytes = body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let parsed: LoginOutput = serde_json::from_slice(&bytes).unwrap();

    (session_id, parsed.csrf_token)
}
