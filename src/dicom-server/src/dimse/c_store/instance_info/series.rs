use dicom_lib::core::value::values::{Cs, Is, Ui};

pub struct Series {
    pub instance_uid: String,
    pub modality: String,
    pub number: Option<i32>,
}

impl Series {
    pub fn new(
        modality: Option<Cs>,
        series_instance_uid: Option<Ui>,
        series_number: Option<Is>,
    ) -> Result<Self, String> {
        let instance_uid = series_instance_uid
            .ok_or("Series Instance UIDが見つかりませんでした".to_string())?
            .uid()
            .to_string();

        let modality = modality
            .ok_or("Modalityが見つかりませんでした".to_string())?
            .code()
            .to_string();

        let number = if let Some(series_number) = series_number {
            Some(series_number.value())
        } else {
            None
        };

        Ok(Series {
            instance_uid,
            modality,
            number,
        })
    }
}
