mod args;
mod internal;

use self::{
    args::Args,
    internal::{
        application::{
            application_entity::CreateApplicationEntityUseCase,
            application_entity::ListApplicationEntitiesUseCase,
            application_entity::UpdateApplicationEntityUseCase,
            user::list_users_use_case::ListUsersUseCase,
        },
        infrastructure::repository::{PostgresApplicationEntityRepository, PostgresUserRepository},
        presentation::handler,
    },
};
use axum::{
    Router,
    routing::{get, post, put},
};
use clap::Parser;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::{io::IsTerminal, net::Ipv4Addr, process::exit, sync::Arc};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{debug, error, info, level_filters::LevelFilter};
use tracing_subscriber::fmt::time::LocalTime;

#[derive(Clone)]
pub struct AppState {
    pub create_application_entity_use_case: Arc<CreateApplicationEntityUseCase>,
    pub list_application_entities_use_case: Arc<ListApplicationEntitiesUseCase>,
    pub update_application_entity_use_case: Arc<UpdateApplicationEntityUseCase>,
    pub list_users_use_case: Arc<ListUsersUseCase>,
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
    let list_users_use_case = Arc::new(ListUsersUseCase::new(user_repository.clone()));

    // アプリケーション状態の初期化
    let app_state = AppState {
        create_application_entity_use_case,
        list_application_entities_use_case,
        update_application_entity_use_case,
        list_users_use_case,
    };

    // ルーター設定
    let app = Router::new()
        .route(
            "/application-entities",
            get(handler::application_entity::list_application_entities),
        )
        .route(
            "/application-entities",
            post(handler::application_entity::create_application_entity),
        )
        .route(
            "/application-entities/{ae_title}",
            put(handler::application_entity::update_application_entity),
        )
        .route("/users", get(handler::user::list_users))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

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
