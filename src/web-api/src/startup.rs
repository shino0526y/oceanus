use crate::internal::{
    application::{
        application_entity::{
            CreateApplicationEntityUseCase, DeleteApplicationEntityUseCase,
            ListApplicationEntitiesUseCase, UpdateApplicationEntityUseCase,
        },
        auth::{AuthenticateUserUseCase, LoginUseCase, LogoutUseCase},
        session::{CreateSessionUseCase, DeleteSessionUseCase, ExtendSessionUseCase},
        user::{
            create_user_use_case::CreateUserUseCase, delete_user_use_case::DeleteUserUseCase,
            list_users_use_case::ListUsersUseCase,
            reset_login_failure_count_use_case::ResetLoginFailureCountUseCase,
            update_user_use_case::UpdateUserUseCase,
        },
    },
    domain::repository::{
        ApplicationEntityRepository, LoginFailureCountRepository, SessionRepository, UserRepository,
    },
    infrastructure::repository::{
        InMemorySessionRepository, PostgresApplicationEntityRepository,
        PostgresLoginFailureCountRepository, PostgresUserRepository,
    },
    presentation::{self, handler},
};
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tower_cookies::CookieManagerLayer;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub struct Repos {
    pub application_entity_repository: Arc<dyn ApplicationEntityRepository>,
    pub user_repository: Arc<dyn UserRepository>,
    pub login_failure_count_repository: Arc<dyn LoginFailureCountRepository>,
    pub session_repository: Arc<dyn SessionRepository>,
}

impl Repos {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            application_entity_repository: Arc::new(PostgresApplicationEntityRepository::new(
                pool.clone(),
            )),
            user_repository: Arc::new(PostgresUserRepository::new(pool.clone())),
            login_failure_count_repository: Arc::new(PostgresLoginFailureCountRepository::new(
                pool.clone(),
            )),
            session_repository: Arc::new(InMemorySessionRepository::new()),
        }
    }

    #[cfg(test)]
    pub fn new_for_test() -> Self {
        use crate::internal::infrastructure::repository::{
            TestApplicationEntityRepository, TestLoginFailureCountRepository,
            TestSessionRepository, TestUserRepository,
        };

        Self {
            application_entity_repository: Arc::new(TestApplicationEntityRepository::new()),
            user_repository: Arc::new(TestUserRepository::new()),
            login_failure_count_repository: Arc::new(TestLoginFailureCountRepository::new()),
            session_repository: Arc::new(TestSessionRepository::new()),
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
    pub extend_session_use_case: Arc<ExtendSessionUseCase>,
}

pub fn make_state(repos: &Repos) -> AppState {
    let create_application_entity_use_case = Arc::new(CreateApplicationEntityUseCase::new(
        repos.application_entity_repository.clone(),
    ));
    let list_application_entities_use_case = Arc::new(ListApplicationEntitiesUseCase::new(
        repos.application_entity_repository.clone(),
    ));
    let update_application_entity_use_case = Arc::new(UpdateApplicationEntityUseCase::new(
        repos.application_entity_repository.clone(),
    ));
    let delete_application_entity_use_case = Arc::new(DeleteApplicationEntityUseCase::new(
        repos.application_entity_repository.clone(),
    ));

    let create_user_use_case = Arc::new(CreateUserUseCase::new(repos.user_repository.clone()));
    let list_users_use_case = Arc::new(ListUsersUseCase::new(
        repos.user_repository.clone(),
        repos.login_failure_count_repository.clone(),
    ));
    let update_user_use_case = Arc::new(UpdateUserUseCase::new(repos.user_repository.clone()));
    let delete_user_use_case = Arc::new(DeleteUserUseCase::new(
        repos.user_repository.clone(),
        repos.login_failure_count_repository.clone(),
        repos.session_repository.clone(),
    ));
    let reset_login_failure_count_use_case = Arc::new(ResetLoginFailureCountUseCase::new(
        repos.user_repository.clone(),
        repos.login_failure_count_repository.clone(),
    ));

    let authenticate_user_use_case = Arc::new(AuthenticateUserUseCase::new(
        repos.user_repository.clone(),
        repos.login_failure_count_repository.clone(),
    ));
    let create_session_use_case =
        Arc::new(CreateSessionUseCase::new(repos.session_repository.clone()));
    let login_use_case = Arc::new(LoginUseCase::new(
        authenticate_user_use_case,
        create_session_use_case,
        repos.user_repository.clone(),
    ));
    let logout_use_case = Arc::new(LogoutUseCase::new(Arc::new(DeleteSessionUseCase::new(
        repos.session_repository.clone(),
    ))));

    let extend_session_use_case =
        Arc::new(ExtendSessionUseCase::new(repos.session_repository.clone()));

    AppState {
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
        extend_session_use_case,
    }
}

pub fn make_router(state: AppState, repos: &Repos) -> Router {
    let session_repository_for_me = repos.session_repository.clone();
    let user_repository_for_me = repos.user_repository.clone();

    let user_repository = repos.user_repository.clone();

    let extend_session_use_case = state.extend_session_use_case.clone();

    Router::new()
        // 認証不要なエンドポイント
        .route("/health", get(handler::health::respond_if_healthy))
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
                    presentation::middleware::require_admin_or_it(
                        request,
                        next,
                        user_repository.clone(),
                    )
                }));

            // まとめてマージし、セッション認証ミドルウェアを適用
            public_auth_router
                .merge(admin_router)
                .route_layer(axum::middleware::from_fn(move |cookies, request, next| {
                    presentation::middleware::session_auth_middleware(
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
        .with_state(state)
}
