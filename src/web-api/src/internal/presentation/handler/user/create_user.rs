mod input;
mod output;

pub use self::{input::CreateUserInput, output::CreateUserOutput};

use crate::{
    AppState,
    internal::{
        application::user::create_user_use_case::{CreateUserCommand, CreateUserError},
        domain::value_object::{Id, Role},
        presentation::{error::PresentationError, middleware::AuthenticatedUser},
    },
};
use axum::{Extension, Json, extract::State};
use chrono::Utc;

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserInput,
    responses(
        (status = 200, description = "ユーザーの作成に成功", body = CreateUserOutput),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "CSRFトークンが無効または権限がありません"),
        (status = 422, description = "バリデーションに失敗"),
    ),
    security(
        ("session_cookie" = []),
        ("csrf_token" = [])
    ),
    tag = "users"
)]
pub async fn create_user(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(input): Json<CreateUserInput>,
) -> Result<Json<CreateUserOutput>, PresentationError> {
    // バリデーション
    let id = Id::new(input.id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なID: {}", e)))?;
    let role = Role::from_i16(input.role).map_err(PresentationError::UnprocessableContent)?;

    // 登録処理
    let command = CreateUserCommand {
        id,
        name: input.name,
        role,
        password: input.password,
        created_by: user.uuid(),
        created_at: Utc::now(),
    };
    let user = state
        .create_user_use_case
        .execute(command)
        .await
        .map_err(|e| match e {
            CreateUserError::EmptyPassword => {
                PresentationError::UnprocessableContent(e.to_string())
            }
            CreateUserError::Forbidden => PresentationError::Forbidden(e.to_string()),
            CreateUserError::PasswordHashError(msg) => PresentationError::InternalServerError(
                format!("パスワードのハッシュ化に失敗しました: {}", msg),
            ),
            CreateUserError::Repository(e) => PresentationError::from(e),
        })?;

    Ok(Json(CreateUserOutput::from(user)))
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        internal::{
            domain::value_object::Id,
            presentation::{handler::user::prepare_test_data, util::test_helpers},
        },
        utils::{self, make_router},
    };
    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use tower::ServiceExt;

    #[tokio::test]
    async fn create_user__管理者はユーザーを作成できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        let input = json!({
            "id": "john",
            "name": "John Doe",
            "role": 2, // Doctor
            "password": "Password#1234",
        });
        let request = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::from(serde_json::to_string(&input).unwrap()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // HTTPレスポンスの確認
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let output: CreateUserOutput = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(output.id, "john");
        assert_eq!(output.name, "John Doe");
        assert_eq!(output.role, 2);
        let now = Utc::now();
        assert!((now - output.created_at).num_seconds().abs() < 10);
        assert_eq!(output.updated_at, output.created_at);
        // ユーザーが作成されていることの確認
        assert!(
            repos
                .user_repository
                .find_by_id(&Id::new("john").unwrap())
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn create_user__情シスはユーザーを作成できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "it", "Password#1234").await;
        let input = json!({
            "id": "john",
            "name": "John Doe",
            "role": 2, // Doctor
            "password": "Password#1234",
        });
        let request = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::from(serde_json::to_string(&input).unwrap()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // HTTPレスポンスの確認
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let output: CreateUserOutput = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(output.id, "john");
        assert_eq!(output.name, "John Doe");
        assert_eq!(output.role, 2);
        let now = Utc::now();
        assert!((now - output.created_at).num_seconds().abs() < 10);
        assert_eq!(output.updated_at, output.created_at);
        // ユーザーが作成されていることの確認
        assert!(
            repos
                .user_repository
                .find_by_id(&Id::new("john").unwrap())
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn create_user__情シスが管理者を作成しようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "it", "Password#1234").await;
        let input = json!({
            "id": "john",
            "name": "John Doe",
            "role": 0, // Admin
            "password": "Password#1234",
        });
        let request = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::from(serde_json::to_string(&input).unwrap()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn create_user__管理者や情シスでないユーザーがユーザーを作成しようとすると403エラーになる()
     {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "technician", "Password#1234").await;
        let input = json!({
            "id": "john",
            "name": "John Doe",
            "role": 2, // Doctor
            "password": "Password#1234",
        });
        let request = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::from(serde_json::to_string(&input).unwrap()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn create_user__すでに存在するユーザーを作成しようとすると409エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        let input = json!({
            "id": "doctor",
            "name": "医師 太郎",
            "role": 2, // Doctor
            "password": "Password#1234",
        });
        let request = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::from(serde_json::to_string(&input).unwrap()))
            .unwrap();

        // Act
        let exists = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap()
            .is_some();
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // リクエストよりも前からユーザーが存在していることを確認
        assert!(exists);
        // HTTPレスポンスの確認
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn create_user__バリデーション違反の場合は422エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        // IDの指定がないケース
        let input1 = json!({
            "name": "John Doe",
            "role": 2, // Doctor
            "password": "Password#1234",
        });
        let request1 = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf.clone())
            .body(Body::from(serde_json::to_string(&input1).unwrap()))
            .unwrap();
        // 空文字のIDを指定するケース
        let input2 = json!({
            "id": "",
            "name": "John Doe",
            "role": 2, // Doctor
            "password": "Password#1234",
        });
        let request2 = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf.clone())
            .body(Body::from(serde_json::to_string(&input2).unwrap()))
            .unwrap();
        // 名前の指定がないケース
        let input3 = json!({
            "id": "john",
            "role": 2, // Doctor
            "password": "Password#1234",
        });
        let request3 = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf.clone())
            .body(Body::from(serde_json::to_string(&input3).unwrap()))
            .unwrap();
        // TODO: 名前が空文字のケースは現状はエラーにならない。バリデーションを追加する
        // 名前が空文字のケース
        // let input4 = json!({
        //     "id": "john",
        //     "name": "",
        //     "role": 2, // Doctor
        //     "password": "Password#1234",
        // });
        // let request4 = Request::builder()
        //     .method("POST")
        //     .uri("/users")
        //     .header("content-type", "application/json")
        //     .header("cookie", format!("session_id={}", session_id))
        //     .header("x-csrf-token", csrf.clone())
        //     .body(Body::from(serde_json::to_string(&input4).unwrap()))
        //     .unwrap();
        // ロールの指定がないケース
        let input5 = json!({
            "id": "john",
            "name": "John Doe",
            "password": "Password#1234",
        });
        let request5 = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf.clone())
            .body(Body::from(serde_json::to_string(&input5).unwrap()))
            .unwrap();
        // ロールが負の値のケース
        let input6 = json!({
            "id": "john",
            "name": "John Doe",
            "role": -1,
            "password": "Password#1234",
        });
        let request6 = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf.clone())
            .body(Body::from(serde_json::to_string(&input6).unwrap()))
            .unwrap();
        // ロールが5以上の値のケース
        let input7 = json!({
            "id": "john",
            "name": "John Doe",
            "role": 5,
            "password": "Password#1234",
        });
        let request7 = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf.clone())
            .body(Body::from(serde_json::to_string(&input7).unwrap()))
            .unwrap();
        // パスワードの指定がないケース
        let input8 = json!({
            "id": "john",
            "name": "John Doe",
            "role": 2, // Doctor
        });
        let request8 = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf.clone())
            .body(Body::from(serde_json::to_string(&input8).unwrap()))
            .unwrap();
        // パスワードが空文字のケース
        let input9 = json!({
            "id": "john",
            "name": "John Doe",
            "role": 2, // Doctor
            "password": "",
        });
        let request9 = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf.clone())
            .body(Body::from(serde_json::to_string(&input9).unwrap()))
            .unwrap();

        // Act
        let response1 = router.clone().oneshot(request1).await.unwrap();
        let response2 = router.clone().oneshot(request2).await.unwrap();
        let response3 = router.clone().oneshot(request3).await.unwrap();
        // let response4 = router.clone().oneshot(request4).await.unwrap();
        let response5 = router.clone().oneshot(request5).await.unwrap();
        let response6 = router.clone().oneshot(request6).await.unwrap();
        let response7 = router.clone().oneshot(request7).await.unwrap();
        let response8 = router.clone().oneshot(request8).await.unwrap();
        let response9 = router.clone().oneshot(request9).await.unwrap();

        // Assert
        assert_eq!(response1.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response2.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response3.status(), StatusCode::UNPROCESSABLE_ENTITY);
        // assert_eq!(response4.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response5.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response6.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response7.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response8.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response9.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
