pub mod c_echo;
pub mod c_store;

use dicom_lib::network::{CommandSet, upper_layer_protocol::pdu::a_abort::Reason};
use tracing::error;

pub struct DimseMessage {
    pub context_id: u8,
    pub abstract_syntax_uid: String,
    pub _transfer_syntax_uid: &'static str,
    pub command_set_buf: Vec<u8>,
    pub data_set_buf: Vec<u8>,
    pub is_command_received: bool,
    pub is_data_received: bool,
}

pub type DimseHandler = fn(DimseMessage) -> Result<(Vec<u8>, Vec<u8>), Reason>;

fn buf_to_command_set(command_set_buf: Vec<u8>) -> Result<CommandSet, Reason> {
    match CommandSet::try_from(command_set_buf) {
        Ok(val) => Ok(val),
        Err(e) => {
            error!("コマンドセットのパースに失敗しました: {e}");
            Err(Reason::InvalidPduParameterValue)
        }
    }
}
