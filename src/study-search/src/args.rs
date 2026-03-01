use clap::{Parser, ValueEnum};
use tracing::level_filters::LevelFilter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// gRPCサーバーのアドレス
    #[arg(
        long = "server-address",
        env = "SERVER_ADDRESS",
        default_value = "http://localhost:50051"
    )]
    pub server_address: String,

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
