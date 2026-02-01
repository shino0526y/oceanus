use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum RepositoryError {
    /// データが競合する（一意制約違反）
    #[error("{resource} '{key}' とデータが競合します")]
    Conflict { resource: String, key: String },
    /// リソースが見つからない
    #[error("{resource} '{key}' が見つかりません")]
    NotFound { resource: String, key: String },
    /// その他のエラー
    #[error("{message}")]
    Other { message: String },
}
