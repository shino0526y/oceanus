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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::Tag,
        network::{
            CommandSet, command_set::Command,
            upper_layer_protocol::pdu::p_data_tf::PresentationDataValue,
        },
    };

    #[tokio::test]
    async fn test_receive_p_data_tf() {
        let expected = PDataTf::new(vec![PresentationDataValue::new(
            1,
            true,
            true,
            CommandSet::new(vec![
                Command::new(Tag(0x0000, 0x0000), 56u32.to_le_bytes().to_vec()),
                Command::new(
                    Tag(0x0000, 0x0002),
                    "1.2.840.10008.1.1\0".as_bytes().to_vec(),
                ),
                Command::new(Tag(0x0000, 0x0100), 0x0030u16.to_le_bytes().to_vec()),
                Command::new(Tag(0x0000, 0x0110), 1u16.to_le_bytes().to_vec()),
                Command::new(Tag(0x0000, 0x0800), 0x0101u16.to_le_bytes().to_vec()),
            ])
            .unwrap(),
        )]);

        let actual = {
            let buf = [
                0x04, 0x00, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x46, 0x01, 0x03, 0x00, 0x00,
                0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00,
                0x12, 0x00, 0x00, 0x00, 0x31, 0x2e, 0x32, 0x2e, 0x38, 0x34, 0x30, 0x2e, 0x31, 0x30,
                0x30, 0x30, 0x38, 0x2e, 0x31, 0x2e, 0x31, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x00,
                0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0x10, 0x01, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00,
                0x00, 0x00, 0x00, 0x08, 0x02, 0x00, 0x00, 0x00, 0x01, 0x01,
            ];
            let mut buf_reader = BufReader::new(&buf[..]);
            match receive_p_data_tf(&mut buf_reader).await.unwrap() {
                PDataTfReception::PDataTf(value) => value,
                PDataTfReception::AAbort(_) => panic!(""),
            }
        };

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_send_p_data_tf() {
        let expected = vec![
            0x04, 0x00, 0x00, 0x00, 0x00, 0x54, 0x00, 0x00, 0x00, 0x50, 0x01, 0x03, 0x00, 0x00,
            0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00,
            0x12, 0x00, 0x00, 0x00, 0x31, 0x2e, 0x32, 0x2e, 0x38, 0x34, 0x30, 0x2e, 0x31, 0x30,
            0x30, 0x30, 0x38, 0x2e, 0x31, 0x2e, 0x31, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x00,
            0x00, 0x00, 0x30, 0x80, 0x00, 0x00, 0x20, 0x01, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x08, 0x02, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x09,
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let actual = {
            let mut buf = vec![];
            send_p_data_tf(
                &mut buf,
                vec![PDataTf::new(vec![PresentationDataValue::new(
                    1,
                    true,
                    true,
                    CommandSet::new(vec![
                        Command::new(Tag(0x0000, 0x0000), 66u32.to_le_bytes().to_vec()),
                        Command::new(
                            Tag(0x0000, 0x0002),
                            "1.2.840.10008.1.1\0".as_bytes().to_vec(),
                        ),
                        Command::new(Tag(0x0000, 0x0100), 0x8030u16.to_le_bytes().to_vec()),
                        Command::new(Tag(0x0000, 0x0120), 1u16.to_le_bytes().to_vec()),
                        Command::new(Tag(0x0000, 0x0800), 0x0101u16.to_le_bytes().to_vec()),
                        Command::new(Tag(0x0000, 0x0900), 0x00u16.to_le_bytes().to_vec()),
                    ])
                    .unwrap(),
                )])],
            )
            .await
            .unwrap();
            buf
        };

        assert_eq!(expected, actual);
    }
}
