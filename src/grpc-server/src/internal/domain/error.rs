use thiserror::Error;

/// ドメイン層のエラー
#[derive(Debug, Clone, Error)]
pub enum DomainError {
    /// 検索条件が不正
    #[error("{message}")]
    InvalidSearchCriteria { message: String },
}

/// リポジトリ層のエラー
#[derive(Debug, Clone, Error)]
pub enum RepositoryError {
    /// その他のエラー
    #[error("{message}")]
    Other { message: String },
}
