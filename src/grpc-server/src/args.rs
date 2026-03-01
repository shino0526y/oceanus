use clap::{Parser, ValueEnum};
use tracing::level_filters::LevelFilter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// データベース接続文字列
    #[arg(long = "database-url", env = "DATABASE_URL")]
    pub database_url: String,

    /// gRPCサーバーポート番号
    #[arg(short = 'p', long = "port", env = "PORT", default_value_t = 50051)]
    pub port: u16,

    /// ログレベル
    #[arg(long = "log-level", env = "LOG_LEVEL", value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
        }
    }
}
