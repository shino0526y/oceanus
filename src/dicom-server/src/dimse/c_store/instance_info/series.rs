use dicom_lib::core::value::value_representations::{cs::CsValue, is::IsValue, ui::UiValue};

pub struct Series {
    pub instance_uid: String,
    pub modality: String,
    pub number: Option<i32>,
}

impl Series {
    pub fn new(
        modality: Option<CsValue>,
        series_instance_uid: Option<UiValue>,
        series_number: Option<IsValue>,
    ) -> Result<Self, String> {
        let instance_uid = series_instance_uid
            .ok_or("Series Instance UIDが見つかりませんでした".to_string())?
            .uid()
            .to_string();

        let modality = modality
            .ok_or("Modalityが見つかりませんでした".to_string())?
            .code()
            .to_string();

        let number = series_number.map(|series_number| series_number.value());

        Ok(Series {
            instance_uid,
            modality,
            number,
        })
    }
}
