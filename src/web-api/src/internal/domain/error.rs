use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum RepositoryError {
    /// データが競合する（一意制約違反）
    #[error("{resource} の {field} '{value}' が既に存在するため、データが競合します")]
    Conflict {
        /// リソース名（例: "ユーザー", "AEタイトル"）
        resource: String,
        /// 競合した項目名（例: "ID", "名前", "タイトル", "ホスト名とポート番号"）
        field: String,
        /// 競合した値
        value: String,
    },
    /// リソースが見つからない
    #[error("{resource} '{key}' が見つかりません")]
    NotFound { resource: String, key: String },
    /// その他のエラー
    #[error("{message}")]
    Other { message: String },
}
