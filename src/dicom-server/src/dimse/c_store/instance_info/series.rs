use dicom_lib::{
    core::value::value_representations::{cs::CsValue, is::IsValue, ui::UiValue},
    network::service_class::storage::{Status, status::code::DataSetMismatch},
};

pub struct Series {
    instance_uid: UiValue,
    modality: CsValue,
    number: Option<IsValue>,
}

impl Series {
    pub fn instance_uid(&self) -> &str {
        self.instance_uid.uid()
    }

    pub fn modality(&self) -> &str {
        self.modality.code()
    }

    pub fn number(&self) -> Option<i32> {
        self.number.as_ref().map(|is_value| is_value.value())
    }

    pub fn new(
        modality: Option<CsValue>,
        series_instance_uid: Option<UiValue>,
        series_number: Option<IsValue>,
    ) -> Result<Self, (String, Status)> {
        let instance_uid = series_instance_uid.ok_or((
            "Series Instance UIDが見つかりませんでした".to_string(),
            Status::DataSetDoesNotMatchSopClassError(DataSetMismatch::new(0xa900).unwrap()),
        ))?;
        let modality = modality.ok_or((
            "Modalityが見つかりませんでした".to_string(),
            Status::DataSetDoesNotMatchSopClassError(DataSetMismatch::new(0xa900).unwrap()),
        ))?;
        let number = series_number;

        Ok(Series {
            instance_uid,
            modality,
            number,
        })
    }
}
