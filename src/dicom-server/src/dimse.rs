pub mod c_echo;
pub mod c_store;

use crate::dimse::c_echo::handle_c_echo;
use dicom_lib::{
    constants::sop_class_uids::{
        COMPUTED_RADIOGRAPHY_IMAGE_STORAGE, CT_IMAGE_STORAGE,
        DIGITAL_MAMMOGRAPHY_X_RAY_IMAGE_STORAGE_FOR_PRESENTATION,
        DIGITAL_X_RAY_IMAGE_STORAGE_FOR_PRESENTATION, MR_IMAGE_STORAGE,
        SECONDARY_CAPTURE_IMAGE_STORAGE, VERIFICATION, X_RAY_ANGIOGRAPHIC_IMAGE_STORAGE,
        X_RAY_RADIOFLUOROSCOPIC_IMAGE_STORAGE,
    },
    core::{DataSet, Encoding},
    network::{CommandSet, upper_layer_protocol::pdu::a_abort::Reason},
};
use std::io::Cursor;
use tracing::error;

pub struct DimseMessage {
    pub context_id: u8,
    pub abstract_syntax_uid: String,
    pub transfer_syntax_uid: &'static str,
    pub command_set_buf: Vec<u8>,
    pub data_set_buf: Vec<u8>,
    pub is_command_received: bool,
    pub is_data_received: bool,
}

fn buf_to_command_set(command_set_buf: Vec<u8>) -> Result<CommandSet, Reason> {
    match CommandSet::try_from(command_set_buf) {
        Ok(val) => Ok(val),
        Err(e) => {
            error!("コマンドセットのパースに失敗しました: {e}");
            Err(Reason::InvalidPduParameterValue)
        }
    }
}

fn buf_to_data_set(buf: &[u8], encoding: Encoding) -> Result<DataSet, Reason> {
    let mut cur = Cursor::new(buf);
    match DataSet::read_from_cur(&mut cur, encoding) {
        Ok(val) => Ok(val),
        Err(e) => {
            error!("データセットのパースに失敗しました: {e}");
            Err(Reason::InvalidPduParameterValue)
        }
    }
}

pub async fn handle_dimse_message(
    dimse_message: DimseMessage,
    ae_title: &str,
) -> Result<(Vec<u8>, Vec<u8>), Reason> {
    match dimse_message.abstract_syntax_uid.as_str() {
        VERIFICATION => handle_c_echo(dimse_message),
        COMPUTED_RADIOGRAPHY_IMAGE_STORAGE
        | DIGITAL_X_RAY_IMAGE_STORAGE_FOR_PRESENTATION
        | DIGITAL_MAMMOGRAPHY_X_RAY_IMAGE_STORAGE_FOR_PRESENTATION
        | CT_IMAGE_STORAGE
        | MR_IMAGE_STORAGE
        | SECONDARY_CAPTURE_IMAGE_STORAGE
        | X_RAY_ANGIOGRAPHIC_IMAGE_STORAGE
        | X_RAY_RADIOFLUOROSCOPIC_IMAGE_STORAGE => {
            c_store::handle_c_store(dimse_message, ae_title).await
        }
        _ => unreachable!(),
    }
}
