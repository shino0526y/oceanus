mod query;

use chrono::NaiveDate;
use std::sync::Arc;

pub use self::query::{QueryError, SearchStudiesQuery, SearchStudiesRecord};

/// 検査検索ユースケースエラー
#[derive(Debug, thiserror::Error)]
pub enum SearchStudiesError {
    /// 検索条件が不正
    #[error("検査日の開始日が終了日より後になっています (開始日=\"{from}\", 終了日=\"{to}\")")]
    InvalidDateRange { from: NaiveDate, to: NaiveDate },
    /// クエリの実行に失敗
    #[error(transparent)]
    Query(#[from] QueryError),
}

/// 検査検索コマンド
pub struct SearchStudiesCommand {
    pub patient_id: Option<String>,
    pub patient_name: Option<String>,
    pub study_date_from: Option<NaiveDate>,
    pub study_date_to: Option<NaiveDate>,
    pub accession_number: Option<String>,
    pub modality: Option<String>,
    pub study_id: Option<String>,
}

/// 検査検索ユースケース
pub struct SearchStudiesUseCase {
    search_studies_query: Arc<dyn SearchStudiesQuery>,
}

impl SearchStudiesUseCase {
    pub fn new(study_search_query: Arc<dyn SearchStudiesQuery>) -> Self {
        Self {
            search_studies_query: study_search_query,
        }
    }

    /// 検査を検索する
    pub async fn execute(
        &self,
        command: SearchStudiesCommand,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<SearchStudiesRecord>, i32), SearchStudiesError> {
        // 検索条件のバリデーション
        if let (Some(from), Some(to)) = (command.study_date_from, command.study_date_to)
            && from > to
        {
            return Err(SearchStudiesError::InvalidDateRange { from, to });
        }

        // limit / offset のデフォルト処理
        let limit = if limit <= 0 { 100 } else { limit.min(1000) };
        let offset = if offset < 0 { 0 } else { offset };

        let (studies, total_count) = self
            .search_studies_query
            .search(
                &command.patient_id,
                &command.patient_name,
                &command.study_date_from,
                &command.study_date_to,
                &command.accession_number,
                &command.modality,
                &command.study_id,
                limit,
                offset,
            )
            .await?;

        Ok((studies, total_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSearchStudiesQuery;

    #[async_trait::async_trait]
    impl SearchStudiesQuery for MockSearchStudiesQuery {
        async fn search(
            &self,
            _patient_id: &Option<String>,
            _patient_name: &Option<String>,
            _study_date_from: &Option<NaiveDate>,
            _study_date_to: &Option<NaiveDate>,
            _accession_number: &Option<String>,
            _modality: &Option<String>,
            _study_id: &Option<String>,
            _limit: i32,
            _offset: i32,
        ) -> Result<(Vec<SearchStudiesRecord>, i32), QueryError> {
            Ok((vec![], 0))
        }
    }

    fn make_use_case() -> SearchStudiesUseCase {
        SearchStudiesUseCase::new(Arc::new(MockSearchStudiesQuery))
    }

    fn default_command() -> SearchStudiesCommand {
        SearchStudiesCommand {
            patient_id: None,
            patient_name: None,
            study_date_from: None,
            study_date_to: None,
            accession_number: None,
            modality: None,
            study_id: None,
        }
    }

    #[tokio::test]
    async fn test_valid_date_range() {
        let uc = make_use_case();
        let command = SearchStudiesCommand {
            study_date_from: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            study_date_to: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            ..default_command()
        };
        assert!(uc.execute(command, 10, 0).await.is_ok());
    }

    #[tokio::test]
    async fn test_same_date() {
        let uc = make_use_case();
        let command = SearchStudiesCommand {
            study_date_from: Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
            study_date_to: Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
            ..default_command()
        };
        assert!(uc.execute(command, 10, 0).await.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_date_range() {
        let uc = make_use_case();
        let command = SearchStudiesCommand {
            study_date_from: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            study_date_to: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            ..default_command()
        };
        assert!(matches!(
            uc.execute(command, 10, 0).await,
            Err(SearchStudiesError::InvalidDateRange { .. })
        ));
    }

    #[tokio::test]
    async fn test_no_dates() {
        let uc = make_use_case();
        assert!(uc.execute(default_command(), 10, 0).await.is_ok());
    }

    #[tokio::test]
    async fn test_only_from_date() {
        let uc = make_use_case();
        let command = SearchStudiesCommand {
            study_date_from: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            ..default_command()
        };
        assert!(uc.execute(command, 10, 0).await.is_ok());
    }

    #[tokio::test]
    async fn test_only_to_date() {
        let uc = make_use_case();
        let command = SearchStudiesCommand {
            study_date_to: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            ..default_command()
        };
        assert!(uc.execute(command, 10, 0).await.is_ok());
    }
}
