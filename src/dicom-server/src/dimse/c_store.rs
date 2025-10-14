use crate::dimse::{DimseMessage, buf_to_command_set};
use dicom_lib::network::{
    CommandSet,
    dimse::c_store::{CStoreRq, CStoreRsp, c_store_rsp::Status},
    upper_layer_protocol::pdu::a_abort::Reason,
};
use tracing::{error, info};

pub fn handle_c_store(dimse_message: DimseMessage) -> Result<(Vec<u8>, Vec<u8>), Reason> {
    let command_set_received = buf_to_command_set(dimse_message.command_set_buf)?;

    let c_store_rq = match CStoreRq::try_from(command_set_received) {
        Ok(val) => val,
        Err(e) => {
            error!("C-STORE-RQのパースに失敗しました: {e}");
            return Err(Reason::InvalidPduParameterValue);
        }
    };
    let message_id = c_store_rq.message_id();
    let priority = c_store_rq.priority();
    let affected_sop_instance_uid = c_store_rq.affected_sop_instance_uid();

    info!(
        "[{}] C-STORE - XXX Storage (メッセージID={message_id}, 優先度={priority}, SOPインスタンスUID=\"{affected_sop_instance_uid}\")",
        dimse_message.context_id
    );

    let c_store_rsp = CStoreRsp::new(
        message_id,
        Status::Success,
        c_store_rq.affected_sop_class_uid(), // TODO: Stringを借用するようにする
        affected_sop_instance_uid,           // TODO: Stringを借用するようにする
    );

    let command_set_to_be_sent: CommandSet = c_store_rsp.into();
    let command_set_buf = command_set_to_be_sent.into();

    Ok((command_set_buf, vec![]))
}
