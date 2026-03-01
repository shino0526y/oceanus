use gpui::*;
use gpui_component::ActiveTheme as _;
use gpui_component::table::{Column, TableDelegate, TableState};
use proto::oceanus::v1::StudyRecord;

/// テーブルの列定義
const COLUMN_DEFS: &[(&str, &str, f32)] = &[
    ("patient_id", "患者ID", 100.0),
    ("patient_name_kanji", "患者名（漢字）", 150.0),
    ("patient_name_hiragana", "患者名（カナ）", 150.0),
    ("patient_sex", "性別", 60.0),
    ("patient_birth_date", "生年月日", 110.0),
    ("study_date", "検査日", 110.0),
    ("study_time", "検査時刻", 90.0),
    ("study_id", "検査ID", 100.0),
    ("accession_number", "受付番号", 120.0),
    ("modalities", "モダリティ", 100.0),
];

/// 検査テーブルのデリゲート
pub struct StudyTableDelegate {
    studies: Vec<StudyRecord>,
    columns: Vec<Column>,
}

impl StudyTableDelegate {
    pub fn new() -> Self {
        let columns = COLUMN_DEFS
            .iter()
            .map(|(key, name, width)| Column::new(*key, *name).width(px(*width)))
            .collect();

        Self {
            studies: Vec::new(),
            columns,
        }
    }

    /// 検索結果でテーブルを更新する
    pub fn set_studies(&mut self, studies: Vec<StudyRecord>) {
        self.studies = studies;
    }
}

impl TableDelegate for StudyTableDelegate {
    fn columns_count(&self, _cx: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &App) -> usize {
        self.studies.len()
    }

    fn column(&self, col_ix: usize, _cx: &App) -> &Column {
        &self.columns[col_ix]
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let text_color = theme.foreground;

        let text = if let Some(study) = self.studies.get(row_ix) {
            match col_ix {
                0 => study.patient_id.clone(),
                1 => study.patient_name_kanji.clone(),
                2 => study.patient_name_hiragana.clone(),
                3 => sex_to_string(study.patient_sex),
                4 => study.patient_birth_date.clone(),
                5 => study.study_date.clone(),
                6 => study.study_time.clone(),
                7 => study.study_id.clone(),
                8 => study.accession_number.clone(),
                9 => study.modalities.join(", "),
                _ => String::new(),
            }
        } else {
            String::new()
        };

        div().px_2().text_sm().text_color(text_color).child(text)
    }
}

/// ISO 5218 性別コードを文字列に変換する
fn sex_to_string(sex: i32) -> String {
    match sex {
        1 => "男性".to_string(),
        2 => "女性".to_string(),
        9 => "その他".to_string(),
        _ => "不明".to_string(),
    }
}
