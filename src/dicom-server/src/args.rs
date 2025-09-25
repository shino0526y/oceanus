#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// サーバのAEタイトル（必須）
    pub ae_title: String,

    /// 受信ポート番号
    #[arg(short = 'p', long = "port", default_value_t = 104)]
    pub port: u16,

    /// ログレベル
    #[arg(long = "log-level", value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for tracing_subscriber::filter::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => tracing_subscriber::filter::LevelFilter::ERROR,
            LogLevel::Warn => tracing_subscriber::filter::LevelFilter::WARN,
            LogLevel::Info => tracing_subscriber::filter::LevelFilter::INFO,
            LogLevel::Debug => tracing_subscriber::filter::LevelFilter::DEBUG,
            LogLevel::Trace => tracing_subscriber::filter::LevelFilter::TRACE,
        }
    }
}
