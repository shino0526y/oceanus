use crate::dimse::{DimseHandler, c_echo::handle_c_echo, c_store::handle_c_store};
use dicom_lib::constants::{
    sop_class_uids::{MR_IMAGE_STORAGE, VERIFICATION},
    transfer_syntax_uids::IMPLICIT_VR_LITTLE_ENDIAN,
};
use phf::{Map, phf_map};

// <root>.<app>.<type>.<version>
// root: 1.3.6.1.4.1.64183 (https://www.iana.org/assignments/enterprise-numbers/)
// app: 1 (Oceanus)
// type: 1 (DICOM Server)
// version: x (major version)
pub const IMPLEMENTATION_CLASS_UID: &str =
    concat!("1.3.6.1.4.1.64183.1.1.", env!("CARGO_PKG_VERSION_MAJOR"));
pub const IMPLEMENTATION_VERSION_NAME: &str = concat!("OCEANUS_", env!("CARGO_PKG_VERSION")); // OCEANUS_x.y.z

pub const MAXIMUM_LENGTH: u32 = 0; // 制限なし

pub const SUPPORTED_ABSTRACT_SYNTAX_UIDS: &[&str] = &[VERIFICATION, MR_IMAGE_STORAGE];
pub const SUPPORTED_TRANSFER_SYNTAX_UIDS: &[&str] = &[IMPLICIT_VR_LITTLE_ENDIAN];

pub const ABSTRACT_SYNTAX_UID_TO_HANDLER: Map<&'static str, DimseHandler> = phf_map! {
    // NOTE: phf_mapはキーとしてリテラルしか扱えない
    "1.2.840.10008.1.1" => handle_c_echo, // Verification
    "1.2.840.10008.5.1.4.1.1.4" => handle_c_store, // MR Image Storage
};
