use crate::internal::{
    application::study::{SearchStudiesUseCase, study_search::SearchStudiesQuery},
    infrastructure::query::study_search::PostgresSearchStudiesQuery,
    presentation::service::StudySearchServiceImpl,
};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

/// クエリの集約
pub struct Queries {
    pub search_studies_query: Arc<dyn SearchStudiesQuery>,
}

impl Queries {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            search_studies_query: Arc::new(PostgresSearchStudiesQuery::new(pool)),
        }
    }
}

/// gRPCサービスの集約
pub struct GrpcServices {
    pub study_search: StudySearchServiceImpl,
}

/// クエリ群からgRPCサービス群を組み立てる
pub fn make_services(queries: &Queries) -> GrpcServices {
    let search_studies_use_case = Arc::new(SearchStudiesUseCase::new(
        queries.search_studies_query.clone(),
    ));

    GrpcServices {
        study_search: StudySearchServiceImpl::new(search_studies_use_case),
    }
}
