use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};

use crate::network::upper_layer_protocol::pdu::{AAbort, PDataTf, PduReadError, PduType};

pub enum PDataTfReception {
    PDataTf(PDataTf),
    AAbort(AAbort),
}

pub async fn receive_p_data_tf(
    buf_reader: &mut BufReader<impl AsyncRead + Unpin>,
) -> Result<PDataTfReception, PduReadError> {
    let pdu_type = {
        let b = buf_reader.read_u8().await?;
        match PduType::try_from(b) {
            Ok(pdu_type) => pdu_type,
            Err(_) => {
                return Err(PduReadError::UnrecognizedPdu(b));
            }
        }
    };
    if pdu_type != PduType::PDataTf && pdu_type != PduType::AAbort {
        return Err(PduReadError::UnexpectedPdu(pdu_type));
    }

    buf_reader.read_u8().await?; // Reserved
    let pdu_length = buf_reader.read_u32().await?;

    if pdu_type == PduType::PDataTf {
        match PDataTf::read_from_stream(buf_reader, pdu_length).await {
            Ok(val) => Ok(PDataTfReception::PDataTf(val)),
            Err(e) => Err(PduReadError::InvalidPduParameterValue {
                message: format!("P-DATA-TFのパースに失敗しました: {e}"),
            }),
        }
    } else {
        match AAbort::read_from_stream(buf_reader, pdu_length).await {
            Ok(val) => Ok(PDataTfReception::AAbort(val)),
            Err(e) => Err(PduReadError::InvalidPduParameterValue {
                message: format!("A-ABORTのパースに失敗しました: {e}"),
            }),
        }
    }
}

pub async fn send_p_data_tf(
    socket: &mut (impl AsyncWrite + Unpin),
    p_data_tf_pdus: Vec<PDataTf>,
) -> std::io::Result<()> {
    for p_data_tf in p_data_tf_pdus {
        let bytes: Vec<u8> = p_data_tf.into();
        socket.write_all(&bytes).await?;
    }

    Ok(())
}
