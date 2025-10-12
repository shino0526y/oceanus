use crate::network::upper_layer_protocol::pdu::{
    AAbort, AReleaseRp, AReleaseRq, PduReadError, PduType,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};

pub enum AReleaseRqReception {
    AReleaseRq(AReleaseRq),
    AAbort(AAbort),
}

pub async fn receive_a_release_rq(
    buf_reader: &mut BufReader<impl AsyncRead + Unpin>,
) -> Result<AReleaseRqReception, PduReadError> {
    let pdu_type = {
        let b = buf_reader.read_u8().await?;
        match PduType::try_from(b) {
            Ok(pdu_type) => pdu_type,
            Err(_) => {
                return Err(PduReadError::UnrecognizedPdu(b));
            }
        }
    };
    if pdu_type != PduType::AReleaseRq && pdu_type != PduType::AAbort {
        return Err(PduReadError::UnexpectedPdu(pdu_type));
    }

    buf_reader.read_u8().await?; // Reserved
    let pdu_length = buf_reader.read_u32().await?;

    if pdu_type == PduType::AReleaseRq {
        match AReleaseRq::read_from_stream(buf_reader, pdu_length).await {
            Ok(val) => Ok(AReleaseRqReception::AReleaseRq(val)),
            Err(e) => Err(PduReadError::InvalidPduParameterValue {
                message: format!("A-RELEASE-RQのパースに失敗しました: {e}"),
            }),
        }
    } else {
        match AAbort::read_from_stream(buf_reader, pdu_length).await {
            Ok(val) => Ok(AReleaseRqReception::AAbort(val)),
            Err(e) => Err(PduReadError::InvalidPduParameterValue {
                message: format!("A-ABORTのパースに失敗しました: {e}"),
            }),
        }
    }
}

pub async fn send_a_release_rp(socket: &mut (impl AsyncWrite + Unpin)) -> std::io::Result<()> {
    let a_release_rp = AReleaseRp::new();

    let bytes: Vec<u8> = a_release_rp.into();
    socket.write_all(&bytes).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_receive_a_release_rq() {
        let expected = AReleaseRq::new();

        let actual = {
            let buf = [0x05, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00];
            let mut buf_reader = BufReader::new(&buf[..]);
            match receive_a_release_rq(&mut buf_reader).await.unwrap() {
                AReleaseRqReception::AReleaseRq(value) => value,
                AReleaseRqReception::AAbort(_) => panic!(""),
            }
        };

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_send_a_release_rp() {
        let expected = vec![0x06, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00];

        let actual = {
            let mut buf = vec![];
            send_a_release_rp(&mut buf).await.unwrap();
            buf
        };

        assert_eq!(expected, actual);
    }
}
