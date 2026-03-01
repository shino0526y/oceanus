mod args;
mod internal;
mod startup;

use self::args::Args;
use clap::Parser;
use dotenvy::dotenv;
use proto::oceanus::v1::study_search_service_server::StudySearchServiceServer;
use sqlx::postgres::PgPoolOptions;
use std::{io::IsTerminal, net::Ipv4Addr, process::exit};
use tonic::transport::Server;
use tracing::{debug, error, info, level_filters::LevelFilter};
use tracing_subscriber::fmt::time::LocalTime;

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

    // リポジトリとサービスの初期化
    let repos = startup::Repos::new(pool);
    let services = startup::make_services(&repos);

    // gRPCサーバー起動
    let addr = (Ipv4Addr::UNSPECIFIED, args.port).into();
    info!("gRPCサーバーが起動しました (ポート={})", args.port);

    if let Err(e) = Server::builder()
        .add_service(StudySearchServiceServer::new(services.study_search))
        .serve_with_shutdown(addr, shutdown_signal())
        .await
    {
        error!("gRPCサービスの実行に失敗しました: {e}");
        exit(1);
    }
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut sigterm =
            signal(SignalKind::terminate()).expect("SIGTERMハンドラの登録に失敗しました");
        let mut sigint =
            signal(SignalKind::interrupt()).expect("SIGINTハンドラの登録に失敗しました");

        tokio::select! {
            _ = sigterm.recv() => info!("SIGTERMを受信しました"),
            _ = sigint.recv() => info!("SIGINTを受信しました"),
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("Ctrl+Cハンドラの登録に失敗しました");
        info!("Ctrl+Cを受信しました");
    }

    info!("サーバーを停止します");
}
