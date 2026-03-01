use std::sync::Arc;

use thiserror::Error;

use crate::internal::domain::{
    entity::study_record::StudyRecord, error::DomainError, error::RepositoryError,
    repository::study_search::StudySearchRepository, value_object::search_criteria::SearchCriteria,
};

/// 検査検索ユースケース
pub struct SearchStudiesUseCase {
    study_search_repository: Arc<dyn StudySearchRepository>,
}

impl SearchStudiesUseCase {
    pub fn new(study_search_repository: Arc<dyn StudySearchRepository>) -> Self {
        Self {
            study_search_repository,
        }
    }

    /// 検査を検索する
    pub async fn execute(
        &self,
        criteria: SearchCriteria,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<StudyRecord>, i32), SearchStudiesError> {
        // 検索条件のバリデーション
        criteria.validate()?;

        // limit / offset のデフォルト処理
        let limit = if limit <= 0 { 100 } else { limit.min(1000) };
        let offset = if offset < 0 { 0 } else { offset };

        let (studies, total_count) = self
            .study_search_repository
            .search(&criteria, limit, offset)
            .await?;

        Ok((studies, total_count))
    }
}

#[derive(Debug, Error)]
pub enum SearchStudiesError {
    #[error(transparent)]
    InvalidCriteria(#[from] DomainError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
