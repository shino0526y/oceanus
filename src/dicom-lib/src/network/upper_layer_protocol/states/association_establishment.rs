use crate::network::upper_layer_protocol::pdu::{
    AAssociateAc, AAssociateRj, AAssociateRq, PduReadError, PduType, a_associate, a_associate_ac,
    a_associate_rj,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};

pub async fn receive_a_associate_rq(
    buf_reader: &mut BufReader<impl AsyncRead + Unpin>,
) -> Result<AAssociateRq, PduReadError> {
    let pdu_type = {
        let b = buf_reader.read_u8().await?;
        match PduType::try_from(b) {
            Ok(pdu_type) => pdu_type,
            Err(_) => {
                return Err(PduReadError::UnrecognizedPdu(b));
            }
        }
    };
    if pdu_type != PduType::AAssociateRq {
        return Err(PduReadError::UnexpectedPdu(pdu_type));
    }

    buf_reader.read_u8().await?; // Reserved
    let pdu_length = buf_reader.read_u32().await?;

    match AAssociateRq::read_from_stream(buf_reader, pdu_length).await {
        Ok(val) => Ok(val),
        Err(e) => Err(PduReadError::InvalidPduParameterValue {
            message: format!("A-ASSOCIATE-RQのパースに失敗しました: {e}"),
        }),
    }
}

pub async fn send_a_associate_ac(
    socket: &mut (impl AsyncWrite + Unpin),
    called_ae_title: &str,
    calling_ae_title: &str,
    application_context: a_associate::ApplicationContext,
    presentation_contexts: Vec<a_associate_ac::PresentationContext>,
    user_information: a_associate::UserInformation,
) -> std::io::Result<()> {
    let a_associate_ac = AAssociateAc::new(
        1,
        called_ae_title,
        calling_ae_title,
        application_context,
        presentation_contexts,
        user_information,
    )
    .unwrap();

    let bytes: Vec<u8> = a_associate_ac.into();
    socket.write_all(&bytes).await?;

    Ok(())
}

pub async fn send_a_associate_rj(
    socket: &mut (impl AsyncWrite + Unpin),
    result: a_associate_rj::Result,
    source_and_reason: a_associate_rj::SourceAndReason,
) -> std::io::Result<()> {
    let a_associate_rj = AAssociateRj::new(result, source_and_reason);

    let bytes: Vec<u8> = a_associate_rj.into();
    socket.write_all(&bytes).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::upper_layer_protocol::pdu::{
        a_associate::{
            ApplicationContext, UserInformation,
            user_information::{ImplementationClassUid, ImplementationVersionName, MaximumLength},
        },
        a_associate_ac::presentation_context::TransferSyntax,
        a_associate_rq::{PresentationContext, presentation_context::AbstractSyntax},
    };

    const BUF: [u8; 211] = [
        0x01, 0x00, 0x00, 0x00, 0x00, 0xcd, 0x00, 0x01, 0x00, 0x00, 0x4f, 0x43, 0x45, 0x41, 0x4e,
        0x55, 0x53, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x44, 0x43, 0x4d, 0x54,
        0x4b, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
        0x00, 0x00, 0x15, 0x31, 0x2e, 0x32, 0x2e, 0x38, 0x34, 0x30, 0x2e, 0x31, 0x30, 0x30, 0x30,
        0x38, 0x2e, 0x33, 0x2e, 0x31, 0x2e, 0x31, 0x2e, 0x31, 0x20, 0x00, 0x00, 0x2e, 0x01, 0x00,
        0xff, 0x00, 0x30, 0x00, 0x00, 0x11, 0x31, 0x2e, 0x32, 0x2e, 0x38, 0x34, 0x30, 0x2e, 0x31,
        0x30, 0x30, 0x30, 0x38, 0x2e, 0x31, 0x2e, 0x31, 0x40, 0x00, 0x00, 0x11, 0x31, 0x2e, 0x32,
        0x2e, 0x38, 0x34, 0x30, 0x2e, 0x31, 0x30, 0x30, 0x30, 0x38, 0x2e, 0x31, 0x2e, 0x32, 0x50,
        0x00, 0x00, 0x3a, 0x51, 0x00, 0x00, 0x04, 0x00, 0x00, 0x40, 0x00, 0x52, 0x00, 0x00, 0x1b,
        0x31, 0x2e, 0x32, 0x2e, 0x32, 0x37, 0x36, 0x2e, 0x30, 0x2e, 0x37, 0x32, 0x33, 0x30, 0x30,
        0x31, 0x30, 0x2e, 0x33, 0x2e, 0x30, 0x2e, 0x33, 0x2e, 0x36, 0x2e, 0x39, 0x55, 0x00, 0x00,
        0x0f, 0x4f, 0x46, 0x46, 0x49, 0x53, 0x5f, 0x44, 0x43, 0x4d, 0x54, 0x4b, 0x5f, 0x33, 0x36,
        0x39,
    ];

    #[tokio::test]
    async fn test_receive_a_associate_rq() {
        let expected = AAssociateRq::new(
            1,
            "OCEANUS",
            "DCMTK",
            ApplicationContext::new("1.2.840.10008.3.1.1.1"),
            vec![PresentationContext::new(
                1,
                AbstractSyntax::new("1.2.840.10008.1.1").unwrap(),
                vec![TransferSyntax::new("1.2.840.10008.1.2").unwrap()],
            )],
            UserInformation::new(
                Some(MaximumLength::new(16384)),
                ImplementationClassUid::new("1.2.276.0.7230010.3.0.3.6.9").unwrap(),
                Some(ImplementationVersionName::new("OFFIS_DCMTK_369").unwrap()),
            ),
        )
        .unwrap();

        let mut buf_reader = BufReader::new(&BUF[..]);
        let actual = receive_a_associate_rq(&mut buf_reader).await.unwrap();

        assert_eq!(expected, actual);
    }
}
