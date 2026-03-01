use async_trait::async_trait;

use super::super::{
    entity::study_record::StudyRecord, error::RepositoryError,
    value_object::search_criteria::SearchCriteria,
};

/// 検査検索リポジトリトレイト
#[async_trait]
pub trait StudySearchRepository: Send + Sync {
    /// 検索条件に一致する検査を取得する
    ///
    /// # 戻り値
    /// `(検査レコードのリスト, 総件数)` のタプル
    async fn search(
        &self,
        criteria: &SearchCriteria,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<StudyRecord>, i32), RepositoryError>;
}
