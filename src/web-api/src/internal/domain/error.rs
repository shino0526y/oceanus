use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum RepositoryError {
    /// リソースが既に存在している(重複エラー)
    #[error("{resource} '{key}' は既に存在しています")]
    Duplicate { resource: String, key: String },
    /// その他のエラー
    #[error("{message}")]
    Other { message: String },
}
