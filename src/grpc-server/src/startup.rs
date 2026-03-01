use std::sync::Arc;

use sqlx::{Pool, Postgres};

use crate::internal::{
    application::study_search::SearchStudiesUseCase,
    domain::repository::study_search::StudySearchRepository,
    infrastructure::repository::PostgresStudySearchRepository,
    presentation::study_search::StudySearchServiceImpl,
};

/// リポジトリの集約
pub struct Repos {
    pub study_search_repository: Arc<dyn StudySearchRepository>,
}

impl Repos {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            study_search_repository: Arc::new(PostgresStudySearchRepository::new(pool)),
        }
    }
}

/// gRPCサービスの集約
pub struct GrpcServices {
    pub study_search: StudySearchServiceImpl,
}

/// リポジトリ群からgRPCサービス群を組み立てる
pub fn make_services(repos: &Repos) -> GrpcServices {
    let search_studies_use_case = Arc::new(SearchStudiesUseCase::new(
        repos.study_search_repository.clone(),
    ));

    GrpcServices {
        study_search: StudySearchServiceImpl::new(search_studies_use_case),
    }
}
