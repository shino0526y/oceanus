use dicom_lib::network::{
    CommandSet,
    dimse::c_echo::{CEchoRq, CEchoRsp, c_echo_rsp::Status},
    upper_layer_protocol::pdu::a_abort::Reason,
};
use tracing::{error, info};

pub fn handle_c_echo(
    command_set: CommandSet,
    context_id: u8,
) -> Result<(Vec<u8>, Vec<u8>), Reason> {
    let c_echo_rq = match CEchoRq::try_from(command_set) {
        Ok(val) => val,
        Err(e) => {
            error!("C-ECHO-RQのパースに失敗しました: {e}");
            return Err(Reason::InvalidPduParameterValue);
        }
    };
    let message_id = c_echo_rq.message_id();

    info!("[{context_id}] C-ECHO - Verification SOP Class (MessageID={message_id})");

    let c_echo_rsp = CEchoRsp::new(message_id, Status::Success);

    let command_set_to_be_sent: CommandSet = c_echo_rsp.into();
    let command_set_buf = command_set_to_be_sent.into();

    Ok((command_set_buf, vec![]))
}
