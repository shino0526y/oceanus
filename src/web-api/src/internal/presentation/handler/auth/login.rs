mod request_body;
mod response_body;

pub use self::{request_body::LoginRequestBody, response_body::LoginResponseBody};

use crate::{
    internal::{
        application::auth::{AuthenticationError, LoginCommand},
        domain::{entity::Session, value_object::Id},
        presentation::{
            error::{ErrorResponseBody, PresentationError},
            util::CookieHelper,
        },
    },
    startup::AppState,
};
use axum::{Json, extract::State};
use tower_cookies::Cookies;

#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginRequestBody,
    responses(
        (status = 200, description = "ログインに成功", body = LoginResponseBody),
        (status = 400, description = "リクエストの形式が無効", body = ErrorResponseBody),
        (status = 401, description = "認証に失敗", body = ErrorResponseBody),
        (status = 403, description = "アカウントがロックされている", body = ErrorResponseBody),
        (status = 422, description = "バリデーション失敗", body = ErrorResponseBody),
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(request_body): Json<LoginRequestBody>,
) -> Result<Json<LoginResponseBody>, PresentationError> {
    // バリデーション
    let user_id = Id::new(&request_body.user_id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なユーザーID: {e}")))?;

    // ログイン処理
    let command = LoginCommand {
        user_id: user_id.clone(),
        password: request_body.password,
    };
    let (session_id, csrf_token, role) =
        state
            .login_use_case
            .execute(command)
            .await
            .map_err(|e| match e {
                AuthenticationError::InvalidCredentials => {
                    PresentationError::Unauthorized(e.to_string())
                }
                AuthenticationError::Locked => PresentationError::Forbidden(e.to_string()),
                AuthenticationError::Other { .. } => {
                    PresentationError::InternalServerError(e.to_string())
                }
            })?;

    // Cookie設定
    let cookie = CookieHelper::create_session_cookie(session_id, Session::DEFAULT_EXPIRY_MINUTES);
    cookies.add(cookie);

    Ok(Json(LoginResponseBody {
        user_id: user_id.value().to_string(),
        csrf_token,
        role,
    }))
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::super::prepare_test_data;
    use crate::startup;
    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
    };
    use chrono::Utc;
    use futures::future::join_all;
    use serde_json::{Value, json};
    use tower::ServiceExt;
    use uuid::Uuid;

    #[tokio::test]
    async fn 正しいIDとパスワードでログインできる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let body = json!({
            "userId": "doctor",
            "password": "Password#1234"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        // ステータスコードの確認
        assert_eq!(response.status(), StatusCode::OK);

        // セッションがリポジトリに保存されていることを確認
        // Set-CookieヘッダーからセッションIDを抽出
        let session_id = {
            let cookie_header = response
                .headers()
                .get("set-cookie")
                .unwrap()
                .to_str()
                .unwrap();
            cookie_header
                .split(';')
                .next()
                .unwrap()
                .split('=')
                .nth(1)
                .unwrap()
                .to_string()
        };

        // レスポンスボディの確認
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["userId"], "doctor");
        assert!(!body["csrfToken"].as_str().unwrap().is_empty());
        assert_eq!(body["role"], 2); // doctor

        let session = repos
            .session_repository
            .find_by_session_id(&session_id)
            .await;
        assert!(session.is_some());
        let session = session.unwrap();
        assert_eq!(session.csrf_token(), body["csrfToken"].as_str().unwrap());
    }

    #[tokio::test]
    async fn 存在しないユーザーIDの場合は401エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let body = json!({
            "userId": "notfound",
            "password": "Password#1234"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        // ステータスコードの確認
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn パスワードが間違っている場合は401エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let body = json!({
            "userId": "doctor",
            "password": "wrongpassword"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        // ステータスコードの確認
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn ログイン失敗回数が5回に達したらユーザーがロックされ403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let wrong_body = json!({
            "userId": "doctor",
            "password": "wrongpassword"
        });
        let correct_body = json!({
            "userId": "doctor",
            "password": "Password#1234"
        });
        let wrong_request = || {
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&wrong_body).unwrap()))
                .unwrap()
        };
        let correct_request = || {
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&correct_body).unwrap()))
                .unwrap()
        };

        // Act
        // 5回連続で間違ったパスワードを送信
        let response1 = router.clone().oneshot(wrong_request()).await.unwrap();
        let response2 = router.clone().oneshot(wrong_request()).await.unwrap();
        let response3 = router.clone().oneshot(wrong_request()).await.unwrap();
        let response4 = router.clone().oneshot(wrong_request()).await.unwrap();
        let response5 = router.clone().oneshot(wrong_request()).await.unwrap();
        // その後、正しいパスワードでログインを試みる
        let response6 = router.clone().oneshot(correct_request()).await.unwrap();
        // またパスワードを間違える
        let response7 = router.clone().oneshot(wrong_request()).await.unwrap();

        // Assert
        // 4回失敗までは401エラー
        assert_eq!(response1.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response2.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response3.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response4.status(), StatusCode::UNAUTHORIZED);

        // 5回目でロック（403エラー）
        assert_eq!(response5.status(), StatusCode::FORBIDDEN);

        // ロック後は正しいパスワードでも403エラー
        assert_eq!(response6.status(), StatusCode::FORBIDDEN);

        // ロック後に再度間違ったパスワードを送っても403エラー。ログイン失敗回数は増える
        assert_eq!(response7.status(), StatusCode::FORBIDDEN);

        // ログイン失敗回数がリポジトリに反映されていることを確認
        let login_failure_count = repos
            .login_failure_count_repository
            .find_by_user_uuid(&Uuid::parse_str("492236d4-2f18-76ab-a82f-84e29fcf92f8").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(login_failure_count.failure_count(), 6); // 合計6回の失敗
        let last_failure_at = login_failure_count.last_failure_at().unwrap();
        let duration = Utc::now().signed_duration_since(*last_failure_at);
        assert!(duration.num_seconds().abs() < 10);
    }

    #[tokio::test]
    async fn バリデーション違反の場合は422エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let bodies = [
            json!({ // フィールドなし
                "password": "Password#1234"
            }),
            json!({ // 空文字
                "userId": "",
                "password": "Password#1234"
            }),
        ];
        let requests = bodies.iter().map(|body| {
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap()
        });

        // Act
        let responses = join_all(requests.map(|req| router.clone().oneshot(req))).await;

        // Assert
        responses.into_iter().for_each(|res| {
            assert_eq!(res.unwrap().status(), StatusCode::UNPROCESSABLE_ENTITY);
        });
    }
}
