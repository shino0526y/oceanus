#[derive(thiserror::Error, Debug)]
pub enum StreamParseError {
    #[error("フォーマットが不正です: {message}")]
    InvalidFormat { message: String },
    #[error("データの終端に予期せず到達しました")]
    UnexpectedEndOfBuffer,
    #[error("I/O エラーが発生しました")]
    IoError(std::io::Error),
}

impl From<std::io::Error> for StreamParseError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::UnexpectedEof => StreamParseError::UnexpectedEndOfBuffer,
            _ => StreamParseError::IoError(e),
        }
    }
}
