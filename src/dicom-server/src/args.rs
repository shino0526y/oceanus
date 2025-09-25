use clap::{Parser, ValueEnum};
use tracing::level_filters::LevelFilter;

#[derive(Parser, Debug)]
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
