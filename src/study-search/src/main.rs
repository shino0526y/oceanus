mod app;
mod args;
mod grpc_client;
mod search_form;
mod study_table;

use self::args::Args;
use clap::Parser;
use dotenvy::dotenv;
use gpui::*;
use gpui_component::init;
use grpc_client::GrpcClient;
use std::sync::Arc;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::time::LocalTime;

fn main() {
    // 環境変数の読み込み
    let _ = dotenv();

    // コマンドライン引数の解析
    let args = Args::parse();

    // ログ設定
    {
        let log_level_filter: LevelFilter = args.log_level.into();

        tracing_subscriber::fmt()
            .with_timer(LocalTime::rfc_3339())
            .with_max_level(log_level_filter)
            .with_target(false)
            .init();
    }

    // gRPCクライアントの初期化
    let grpc_client = Arc::new(GrpcClient::new(args.server_address));

    // GPUIアプリケーション起動
    Application::new().run(move |cx: &mut App| {
        // gpui-component の初期化
        init(cx);

        let bounds = Bounds::centered(None, size(px(1200.), px(800.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Oceanus - 検査検索".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                let client = grpc_client.clone();
                let view = cx.new(|cx| app::App::new(window, cx, client));
                cx.new(|cx| gpui_component::Root::new(view, window, cx))
            },
        )
        .expect("ウィンドウの作成に失敗しました");
    });
}
