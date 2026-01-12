use crate::internal::domain::value_object::Port;
use dicom_lib::core::value::value_representations::ae::AeValue;

pub struct CreateApplicationEntityCommand {
    pub title: AeValue,
    pub host: String,
    pub port: Port,
    pub comment: String,
}
