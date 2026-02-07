mod response_body;

pub use self::response_body::MeResponseBody;

use crate::internal::{
    domain::{
        entity::Session,
        repository::{SessionRepository, UserRepository},
    },
    presentation::{
        error::{ErrorResponseBody, PresentationError},
        util::CookieHelper,
    },
};
use axum::Json;
use std::sync::Arc;
use tower_cookies::Cookies;

#[utoipa::path(
    get,
    path = "/me",
    responses(
        (status = 200, description = "認証済みユーザー情報を取得", body = MeResponseBody),
        (status = 400, description = "リクエストの形式が無効", body = ErrorResponseBody),
        (status = 401, description = "セッションが確立されていないか期限が切れている", body = ErrorResponseBody),
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "auth"
)]
pub async fn me(
    cookies: Cookies,
    session_repository: Arc<dyn SessionRepository>,
    user_repository: Arc<dyn UserRepository>,
) -> Result<Json<MeResponseBody>, PresentationError> {
    // CookieからセッションIDを取得
    let session_id = cookies
        .get(CookieHelper::SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or(PresentationError::Unauthorized(
            "認証されていません".to_string(),
        ))?;

    // セッションを取得
    let mut session = session_repository
        .find_by_session_id(&session_id)
        .await
        .ok_or(PresentationError::Unauthorized(
            "認証されていません".to_string(),
        ))?;

    // ユーザーID（UUID）とCSRFトークンを取得
    let user_uuid = *session.user_uuid();
    let csrf_token = session.csrf_token().to_string();

    // ユーザーのロールを取得
    let user = user_repository
        .find_by_uuid(&user_uuid)
        .await
        .map_err(|e| PresentationError::InternalServerError(e.to_string()))?
        .ok_or(PresentationError::Unauthorized(
            "認証されていません".to_string(),
        ))?;

    // セッションを延長
    session.extend();
    session_repository.save(session).await;

    // Cookieの有効期限も更新
    let cookie = CookieHelper::create_session_cookie(session_id, Session::DEFAULT_EXPIRY_MINUTES);
    cookies.add(cookie);

    Ok(Json(MeResponseBody {
        user_id: user.id().value().to_string(),
        csrf_token,
        role: user.role().as_i16(),
    }))
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::super::prepare_test_data;
    use crate::{
        internal::{domain::entity::Session, presentation::util::test_helpers},
        startup,
    };
    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
    };
    use chrono::{Duration, Utc};
    use serde_json::Value;
    use tower::ServiceExt;
    use uuid::Uuid;

    #[tokio::test]
    async fn 認証済みユーザー情報を取得できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        // ログインしてセッションを取得
        let (session_id, csrf_token) =
            test_helpers::login(&router, "doctor", "Password#1234").await;
        let user_uuid = Uuid::parse_str("492236d4-2f18-76ab-a82f-84e29fcf92f8").unwrap();

        let request = Request::builder()
            .method("GET")
            .uri("/me")
            .header("cookie", format!("session_id={session_id}"))
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        // ステータスコードの確認
        assert_eq!(response.status(), StatusCode::OK);

        // レスポンスボディの確認
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["userId"], "doctor");
        assert_eq!(body["csrfToken"], csrf_token);
        assert_eq!(body["role"], 2); // Doctor

        // リポジトリに反映されていることの確認（セッションが延長されたことの確認）
        let updated_session = repos
            .session_repository
            .find_by_session_id(&session_id)
            .await
            .unwrap();
        assert_eq!(
            updated_session.user_uuid().to_string(),
            user_uuid.to_string()
        );
        assert_eq!(updated_session.csrf_token(), csrf_token);
        let expected_expiry = Utc::now() + Duration::minutes(Session::DEFAULT_EXPIRY_MINUTES);
        let duration = updated_session
            .expires_at()
            .signed_duration_since(expected_expiry);
        assert!(duration.num_seconds().abs() < 10);
    }
}
