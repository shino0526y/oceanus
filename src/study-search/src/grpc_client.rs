use std::sync::Arc;

use proto::oceanus::v1::{
    SearchStudiesRequest, SearchStudiesResponse,
    study_search_service_client::StudySearchServiceClient,
};
use tokio::runtime::Runtime;
use tonic::transport::Channel;
use tracing::error;

/// gRPC クライアントラッパー
///
/// バックグラウンドスレッドで Tokio ランタイムを保持し、
/// GPUI のメインスレッドからブロッキングで gRPC 呼び出しを行う。
pub struct GrpcClient {
    runtime: Arc<Runtime>,
    server_address: String,
}

impl GrpcClient {
    pub fn new(server_address: String) -> Self {
        let runtime = Arc::new(Runtime::new().expect("Tokioランタイムの作成に失敗しました"));
        Self {
            runtime,
            server_address,
        }
    }

    /// 検査を検索する
    pub async fn search_studies(
        &self,
        request: SearchStudiesRequest,
    ) -> Result<SearchStudiesResponse, String> {
        let addr = self.server_address.clone();
        let channel = Channel::from_shared(addr)
            .map_err(|e| format!("gRPCチャネルの作成に失敗しました: {e}"))?
            .connect()
            .await
            .map_err(|e| format!("gRPCサーバーへの接続に失敗しました: {e}"))?;

        let mut client = StudySearchServiceClient::new(channel);

        let response = client.search_studies(request).await.map_err(|e| {
            error!("検査の検索に失敗しました: {e}");
            format!("検査の検索に失敗しました: {e}")
        })?;

        Ok(response.into_inner())
    }

    /// Tokioランタイムへの参照を返す
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }
}
