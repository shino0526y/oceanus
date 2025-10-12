pub mod c_echo;

use dicom_lib::network::upper_layer_protocol::pdu::a_abort::Reason;

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
