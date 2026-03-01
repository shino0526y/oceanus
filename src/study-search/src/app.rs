use gpui::*;
use gpui_component::ActiveTheme as _;
use gpui_component::table::{Table, TableState};

use crate::grpc_client::GrpcClient;
use crate::search_form::{SearchForm, SearchRequested};
use crate::study_table::StudyTableDelegate;
use std::sync::Arc;
use tracing::{error, info};

/// アプリケーションのルートView
pub struct App {
    search_form: Entity<SearchForm>,
    table_state: Entity<TableState<StudyTableDelegate>>,
    grpc_client: Arc<GrpcClient>,
    status_message: String,
}

impl App {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, grpc_client: Arc<GrpcClient>) -> Self {
        let client = grpc_client.clone();
        let search_form = cx.new(|cx| SearchForm::new(window, cx, client));
        let table_state = cx.new(|cx| {
            let delegate = StudyTableDelegate::new();
            TableState::new(delegate, window, cx)
        });

        // 検索フォームのイベントを購読
        cx.subscribe(&search_form, Self::on_search_requested)
            .detach();

        Self {
            search_form,
            table_state,
            grpc_client,
            status_message: "検索条件を入力して「検索」ボタンを押してください".to_string(),
        }
    }

    fn on_search_requested(
        &mut self,
        _: Entity<SearchForm>,
        event: &SearchRequested,
        cx: &mut Context<Self>,
    ) {
        let request = event.request.clone();
        let client = self.grpc_client.clone();
        let table_state = self.table_state.clone();

        self.status_message = "検索中...".to_string();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let runtime = client.runtime().clone();
            let result = runtime
                .spawn(async move { client.search_studies(request).await })
                .await;

            cx.update(|cx| match result {
                Ok(Ok(response)) => {
                    let count = response.studies.len();
                    let total = response.total_count;
                    info!("検査の検索が完了しました (取得件数={count}, 総件数={total})");

                    table_state.update(cx, |state, cx| {
                        state.delegate_mut().set_studies(response.studies);
                        cx.notify();
                    });
                    let _ = this.update(cx, |this, cx| {
                        this.status_message = format!("{total}件中 {count}件を表示");
                        cx.notify();
                    });
                }
                Ok(Err(e)) => {
                    error!("検査の検索に失敗しました: {e}");
                    let _ = this.update(cx, |this, cx| {
                        this.status_message = format!("エラー: {e}");
                        cx.notify();
                    });
                }
                Err(e) => {
                    error!("非同期タスクの実行に失敗しました: {e}");
                    let _ = this.update(cx, |this, cx| {
                        this.status_message = format!("エラー: {e}");
                        cx.notify();
                    });
                }
            })
        })
        .detach();
    }
}

impl Render for App {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(self.search_form.clone())
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(Table::new(&self.table_state)),
            )
            .child(
                // ステータスバー
                div()
                    .flex()
                    .items_center()
                    .px_4()
                    .py_1()
                    .border_t_1()
                    .border_color(theme.border)
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(self.status_message.clone()),
            )
    }
}
