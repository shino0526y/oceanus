mod args;
mod internal;

use self::{
    args::Args,
    internal::{
        application::{
            application_entity::CreateApplicationEntityUseCase,
            application_entity::DeleteApplicationEntityUseCase,
            application_entity::ListApplicationEntitiesUseCase,
            application_entity::UpdateApplicationEntityUseCase,
            auth::{AuthenticateUserUseCase, LoginUseCase, LogoutUseCase},
            session::{CreateSessionUseCase, DeleteSessionUseCase, ExtendSessionUseCase},
            user::create_user_use_case::CreateUserUseCase,
            user::delete_user_use_case::DeleteUserUseCase,
            user::list_users_use_case::ListUsersUseCase,
            user::reset_login_failure_count_use_case::ResetLoginFailureCountUseCase,
            user::update_user_use_case::UpdateUserUseCase,
        },
        infrastructure::repository::{
            InMemorySessionRepository, PostgresApplicationEntityRepository,
            PostgresLoginFailureCountRepository, PostgresUserRepository,
        },
        presentation::{handler, middleware},
    },
};
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use clap::Parser;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::{io::IsTerminal, net::Ipv4Addr, process::exit, sync::Arc};
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{debug, error, info, level_filters::LevelFilter};
use tracing_subscriber::fmt::time::LocalTime;

// Swagger UI関連
#[cfg(debug_assertions)]
use utoipa::{
    OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};

#[cfg(debug_assertions)]
#[derive(OpenApi)]
#[openapi(
    paths(
        internal::presentation::handler::auth::login::login,
        internal::presentation::handler::auth::logout::logout,
        internal::presentation::handler::auth::me::me,
        internal::presentation::handler::user::create_user::create_user,
        internal::presentation::handler::user::list_users::list_users,
        internal::presentation::handler::user::update_user::update_user,
        internal::presentation::handler::user::delete_user::delete_user,
        internal::presentation::handler::user::reset_login_failure_count::reset_login_failure_count,
        internal::presentation::handler::application_entity::create_application_entity::create_application_entity,
        internal::presentation::handler::application_entity::list_application_entities::list_application_entities,
        internal::presentation::handler::application_entity::update_application_entity::update_application_entity,
        internal::presentation::handler::application_entity::delete_application_entity::delete_application_entity,
    ),
    components(schemas(
        internal::presentation::handler::auth::login::LoginInput,
        internal::presentation::handler::auth::login::LoginOutput,
        internal::presentation::handler::auth::login::ErrorResponse,
        internal::presentation::handler::auth::logout::ErrorResponse,
        internal::presentation::handler::auth::me::MeOutput,
        internal::presentation::handler::auth::me::ErrorResponse,
        internal::presentation::handler::user::create_user::CreateUserInput,
        internal::presentation::handler::user::create_user::CreateUserOutput,
        internal::presentation::handler::user::list_users::ListUsersOutputElement,
        internal::presentation::handler::user::update_user::UpdateUserInput,
        internal::presentation::handler::user::update_user::UpdateUserOutput,
        internal::presentation::handler::application_entity::create_application_entity::CreateApplicationEntityInput,
        internal::presentation::handler::application_entity::create_application_entity::CreateApplicationEntityOutput,
        internal::presentation::handler::application_entity::list_application_entities::ListApplicationEntitiesOutputElement,
        internal::presentation::handler::application_entity::update_application_entity::UpdateApplicationEntityInput,
        internal::presentation::handler::application_entity::update_application_entity::UpdateApplicationEntityOutput,
    )),
    tags(
        (name = "auth", description = "認証API"),
        (name = "users", description = "ユーザー管理API"),
        (name = "application-entities", description = "Application Entity管理API")
    ),
    modifiers(&SecurityAddon),
    info(
        title = "Web API",
        version = "1.0.0",
        description = "DICOM Web APIサーバー",
    )
)]
struct ApiDoc;

#[cfg(debug_assertions)]
struct SecurityAddon;

#[cfg(debug_assertions)]
impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "csrf_token",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-CSRF-Token"))),
            );
            components.add_security_scheme(
                "session_cookie",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("session_id"))),
            );
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub create_application_entity_use_case: Arc<CreateApplicationEntityUseCase>,
    pub list_application_entities_use_case: Arc<ListApplicationEntitiesUseCase>,
    pub update_application_entity_use_case: Arc<UpdateApplicationEntityUseCase>,
    pub delete_application_entity_use_case: Arc<DeleteApplicationEntityUseCase>,
    pub create_user_use_case: Arc<CreateUserUseCase>,
    pub list_users_use_case: Arc<ListUsersUseCase>,
    pub update_user_use_case: Arc<UpdateUserUseCase>,
    pub delete_user_use_case: Arc<DeleteUserUseCase>,
    pub reset_login_failure_count_use_case: Arc<ResetLoginFailureCountUseCase>,
    pub login_use_case: Arc<LoginUseCase>,
    pub logout_use_case: Arc<LogoutUseCase>,
}

#[tokio::main]
async fn main() {
    // 環境変数の読み込み
    let _ = dotenv();

    // コマンドライン引数の解析
    let args = Args::parse();

    // ログ設定
    {
        let is_tty = std::io::stdout().is_terminal();
        let log_level_filter: LevelFilter = args.log_level.into();

        tracing_subscriber::fmt()
            .with_ansi(is_tty)
            .with_timer(LocalTime::rfc_3339())
            .with_max_level(log_level_filter)
            .with_target(false)
            .init();
    }

    // データベース接続
    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&args.database_url)
        .await
    {
        Ok(pool) => {
            debug!("データベースに接続しました");
            pool
        }
        Err(e) => {
            error!("データベースへの接続に失敗しました: {e}");
            exit(1);
        }
    };

    // リポジトリの初期化
    let application_entity_repository =
        Arc::new(PostgresApplicationEntityRepository::new(pool.clone()));
    let user_repository = Arc::new(PostgresUserRepository::new(pool.clone()));
    let login_failure_count_repository =
        Arc::new(PostgresLoginFailureCountRepository::new(pool.clone()));
    let session_repository = Arc::new(InMemorySessionRepository::new());

    // セッションクリーンアップの定期実行ジョブ（メモリ内セッションの期限切れ削除）
    {
        let session_repo = session_repository.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                session_repo.cleanup_expired_sessions();
            }
        });
    }

    // ユースケースの初期化
    let create_application_entity_use_case = Arc::new(CreateApplicationEntityUseCase::new(
        application_entity_repository.clone(),
    ));
    let list_application_entities_use_case = Arc::new(ListApplicationEntitiesUseCase::new(
        application_entity_repository.clone(),
    ));
    let update_application_entity_use_case = Arc::new(UpdateApplicationEntityUseCase::new(
        application_entity_repository.clone(),
    ));
    let delete_application_entity_use_case = Arc::new(DeleteApplicationEntityUseCase::new(
        application_entity_repository.clone(),
    ));
    let create_user_use_case = Arc::new(CreateUserUseCase::new(user_repository.clone()));
    let list_users_use_case = Arc::new(ListUsersUseCase::new(
        user_repository.clone(),
        login_failure_count_repository.clone(),
    ));
    let update_user_use_case = Arc::new(UpdateUserUseCase::new(user_repository.clone()));
    let delete_user_use_case = Arc::new(DeleteUserUseCase::new(
        user_repository.clone(),
        login_failure_count_repository.clone(),
    ));
    let reset_login_failure_count_use_case = Arc::new(ResetLoginFailureCountUseCase::new(
        user_repository.clone(),
        login_failure_count_repository.clone(),
    ));

    // 認証関連UseCaseの初期化
    let authenticate_user_use_case = Arc::new(AuthenticateUserUseCase::new(
        user_repository.clone(),
        login_failure_count_repository.clone(),
    ));
    let create_session_use_case = Arc::new(CreateSessionUseCase::new(session_repository.clone()));
    let delete_session_use_case = Arc::new(DeleteSessionUseCase::new(session_repository.clone()));
    let extend_session_use_case = Arc::new(ExtendSessionUseCase::new(session_repository.clone()));
    let login_use_case = Arc::new(LoginUseCase::new(
        authenticate_user_use_case,
        create_session_use_case.clone(),
        user_repository.clone(),
    ));
    let logout_use_case = Arc::new(LogoutUseCase::new(delete_session_use_case.clone()));

    // アプリケーション状態の初期化
    let app_state = AppState {
        create_application_entity_use_case,
        list_application_entities_use_case,
        update_application_entity_use_case,
        delete_application_entity_use_case,
        create_user_use_case,
        list_users_use_case,
        update_user_use_case,
        delete_user_use_case,
        reset_login_failure_count_use_case,
        login_use_case,
        logout_use_case,
    };

    // ルーター設定
    let session_repository_for_me = session_repository.clone();
    let user_repository_for_me = user_repository.clone();
    let app = Router::new()
        // 認証不要なエンドポイント
        .route("/login", post(handler::auth::login))
        .route(
            "/me",
            get(move |cookies| {
                handler::auth::me(cookies, session_repository_for_me, user_repository_for_me)
            }),
        )
        // 認証が必要なエンドポイントにミドルウェアを適用
        .merge({
            // 認証は必要だが、管理者権限は不要なルート
            let public_auth_router = Router::new().route("/logout", post(handler::auth::logout));

            // 管理者または情シス権限が必要なルート
            let admin_router = Router::new()
                .route(
                    "/application-entities",
                    post(handler::application_entity::create_application_entity),
                )
                .route(
                    "/application-entities",
                    get(handler::application_entity::list_application_entities),
                )
                .route(
                    "/application-entities/{ae_title}",
                    put(handler::application_entity::update_application_entity),
                )
                .route(
                    "/application-entities/{ae_title}",
                    delete(handler::application_entity::delete_application_entity),
                )
                .route("/users", post(handler::user::create_user))
                .route("/users", get(handler::user::list_users))
                .route("/users/{id}", put(handler::user::update_user))
                .route("/users/{id}", delete(handler::user::delete_user))
                .route(
                    "/users/{id}/login-failure-count",
                    delete(handler::user::reset_login_failure_count),
                )
                // 管理者チェックミドルウェアを適用
                .layer(axum::middleware::from_fn(move |request, next| {
                    middleware::require_admin_or_it(request, next, user_repository.clone())
                }));

            // まとめてマージし、セッション認証ミドルウェアを適用
            public_auth_router
                .merge(admin_router)
                .route_layer(axum::middleware::from_fn(move |cookies, request, next| {
                    middleware::session_auth_middleware(
                        cookies,
                        extend_session_use_case.clone(),
                        request,
                        next,
                    )
                }))
        })
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new())
        .with_state(app_state);

    // Swagger UIの設定
    #[cfg(debug_assertions)]
    let app = {
        use utoipa_swagger_ui::SwaggerUi;
        app.merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
                .config(
                    utoipa_swagger_ui::Config::default()
                        .try_it_out_enabled(true)
                        .request_snippets_enabled(true)
                        .with_credentials(true),
                ),
        )
    };

    // サーバー起動
    let port = args.port;
    let listener = match TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("通信の待ち受けに失敗しました (ポート番号={port}): {e}");
            exit(1);
        }
    };
    info!("サーバーが起動しました (ポート={port})");
    if let Err(e) = axum::serve(listener, app).await {
        error!("HTTPサービスの実行に失敗しました: {e}");
        exit(1);
    }
}
