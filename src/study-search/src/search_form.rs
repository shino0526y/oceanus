use gpui::*;
use gpui_component::ActiveTheme as _;
use gpui_component::button::{Button, ButtonVariants as _};
use gpui_component::input::{Input, InputState};

use proto::oceanus::v1::SearchStudiesRequest;
use std::sync::Arc;

use crate::grpc_client::GrpcClient;

/// 検索実行イベント
pub struct SearchRequested {
    pub request: SearchStudiesRequest,
}

/// 検索フォームの状態
pub struct SearchForm {
    patient_id_input: Entity<InputState>,
    patient_name_input: Entity<InputState>,
    study_date_from_input: Entity<InputState>,
    study_date_to_input: Entity<InputState>,
    accession_number_input: Entity<InputState>,
    modality_input: Entity<InputState>,
    study_id_input: Entity<InputState>,
    _grpc_client: Arc<GrpcClient>,
}

impl SearchForm {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, grpc_client: Arc<GrpcClient>) -> Self {
        let patient_id_input = cx.new(|cx| InputState::new(window, cx).placeholder("患者ID"));
        let patient_name_input = cx.new(|cx| InputState::new(window, cx).placeholder("患者名"));
        let study_date_from_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("検査日From (YYYY-MM-DD)"));
        let study_date_to_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("検査日To (YYYY-MM-DD)"));
        let accession_number_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("受付番号"));
        let modality_input = cx.new(|cx| InputState::new(window, cx).placeholder("モダリティ"));
        let study_id_input = cx.new(|cx| InputState::new(window, cx).placeholder("検査ID"));

        Self {
            patient_id_input,
            patient_name_input,
            study_date_from_input,
            study_date_to_input,
            accession_number_input,
            modality_input,
            study_id_input,
            _grpc_client: grpc_client,
        }
    }

    /// 検索リクエストを構築する
    fn build_request(&self, cx: &App) -> SearchStudiesRequest {
        SearchStudiesRequest {
            patient_id: self.patient_id_input.read(cx).text().to_string(),
            patient_name: self.patient_name_input.read(cx).text().to_string(),
            study_date_from: self.study_date_from_input.read(cx).text().to_string(),
            study_date_to: self.study_date_to_input.read(cx).text().to_string(),
            accession_number: self.accession_number_input.read(cx).text().to_string(),
            modality: self.modality_input.read(cx).text().to_string(),
            study_id: self.study_id_input.read(cx).text().to_string(),
            limit: 100,
            offset: 0,
        }
    }
}

impl EventEmitter<SearchRequested> for SearchForm {}

impl Render for SearchForm {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let border_color = theme.border;

        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .border_b_1()
            .border_color(border_color)
            .child(
                // 1行目: 患者ID, 患者名, モダリティ
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .flex_1()
                            .child(div().text_sm().child("患者ID"))
                            .child(Input::new(&self.patient_id_input)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .flex_1()
                            .child(div().text_sm().child("患者名"))
                            .child(Input::new(&self.patient_name_input)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .flex_1()
                            .child(div().text_sm().child("モダリティ"))
                            .child(Input::new(&self.modality_input)),
                    ),
            )
            .child(
                // 2行目: 検査日From, 検査日To, 受付番号, 検査ID
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .flex_1()
                            .child(div().text_sm().child("検査日From"))
                            .child(Input::new(&self.study_date_from_input)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .flex_1()
                            .child(div().text_sm().child("検査日To"))
                            .child(Input::new(&self.study_date_to_input)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .flex_1()
                            .child(div().text_sm().child("受付番号"))
                            .child(Input::new(&self.accession_number_input)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .flex_1()
                            .child(div().text_sm().child("検査ID"))
                            .child(Input::new(&self.study_id_input)),
                    ),
            )
            .child(
                // 3行目: 検索ボタン
                div().flex().justify_end().child(
                    Button::new("search-btn")
                        .primary()
                        .label("検索")
                        .on_click(cx.listener(move |this, _event, _window, cx| {
                            let request = this.build_request(cx);
                            cx.emit(SearchRequested { request });
                        })),
                ),
            )
    }
}
