use dicom_lib::{
    core::value::value_representations::{is::IsValue, ui::UiValue},
    network::service_class::storage::{Status, status::code::DataSetMismatch},
};

pub struct SopInstance {
    instance_uid: UiValue,
    class_uid: UiValue,
    number: Option<IsValue>,
}

impl SopInstance {
    pub fn instance_uid(&self) -> &str {
        self.instance_uid.uid()
    }

    pub fn class_uid(&self) -> &str {
        self.class_uid.uid()
    }

    pub fn number(&self) -> Option<i32> {
        self.number.as_ref().map(|is_value| is_value.value())
    }

    pub fn new(
        sop_class_uid: Option<UiValue>,
        sop_instance_uid: Option<UiValue>,
        instance_number: Option<IsValue>,
    ) -> Result<Self, (String, Status)> {
        let instance_uid = sop_instance_uid.ok_or((
            "SOP Instance UIDが見つかりませんでした".to_string(),
            Status::DataSetDoesNotMatchSopClassError(DataSetMismatch::new(0xa900).unwrap()),
        ))?;
        let class_uid = sop_class_uid.ok_or((
            "SOP Class UIDが見つかりませんでした".to_string(),
            Status::DataSetDoesNotMatchSopClassError(DataSetMismatch::new(0xa900).unwrap()),
        ))?;
        let number = instance_number;

        Ok(SopInstance {
            instance_uid,
            class_uid,
            number,
        })
    }
}
