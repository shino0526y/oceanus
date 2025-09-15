pub mod a_abort;
mod a_associate;
pub mod a_associate_ac;
pub mod a_associate_rq;
pub mod a_release_rp;
pub mod a_release_rq;
pub mod p_data_tf;

pub use a_abort::AAbort;
pub use a_associate_ac::AAssociateAc;
pub use a_associate_rq::AAssociateRq;
pub use a_release_rp::AReleaseRp;
pub use a_release_rq::AReleaseRq;
pub use p_data_tf::PDataTf;

pub(crate) const INVALID_PDU_LENGTH_ERROR_MESSAGE: &str = "PDU-lengthが不正です";

#[derive(thiserror::Error, Debug)]
pub enum PduReadError {
    UnrecognizedPdu(u8),
    UnexpectedPdu(PduType),
    UnrecognizedPduParameter(u8),
    UnexpectedPduParameter(ItemType),
    InvalidPduParameterValue { message: String },
    IoError(#[from] std::io::Error),
}

impl std::fmt::Display for PduReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnrecognizedPdu(pdu_type) => {
                write!(f, "不明なPDU-typeです (PDU-type=0x{pdu_type:02X})")
            }
            Self::UnexpectedPdu(pdu_type) => {
                write!(
                    f,
                    "想定外のPDU-typeです (PDU-type=0x{:02X})",
                    *pdu_type as u8
                )
            }
            Self::UnrecognizedPduParameter(item_type) => {
                write!(f, "不明なPDUパラメータです (Item-type=0x{item_type:02X})")
            }
            Self::UnexpectedPduParameter(item_type) => {
                write!(
                    f,
                    "想定外のPDUパラメータです (Item-type=0x{:02X})",
                    *item_type as u8
                )
            }
            Self::InvalidPduParameterValue { message } => {
                write!(f, "不正なPDUパラメータ値です: {}", message)
            }
            Self::IoError(err) => {
                write!(f, "I/Oエラーが発生しました: {}", err)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PduType {
    AAssociateRq = a_associate_rq::PDU_TYPE as isize,
    AAssociateAc = a_associate_ac::PDU_TYPE as isize,
    AAssociateRj = 0x03,
    PDataTf = p_data_tf::PDU_TYPE as isize,
    AReleaseRq = a_release_rq::PDU_TYPE as isize,
    AReleaseRp = a_release_rp::PDU_TYPE as isize,
    AAbort = a_abort::PDU_TYPE as isize,
}

impl TryFrom<u8> for PduType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            a_associate_rq::PDU_TYPE => Ok(Self::AAssociateRq),
            a_associate_ac::PDU_TYPE => Ok(Self::AAssociateAc),
            0x03 => Ok(Self::AAssociateRj),
            p_data_tf::PDU_TYPE => Ok(Self::PDataTf),
            a_release_rq::PDU_TYPE => Ok(Self::AReleaseRq),
            a_release_rp::PDU_TYPE => Ok(Self::AReleaseRp),
            a_abort::PDU_TYPE => Ok(Self::AAbort),
            _ => Err("不正なPDU-typeです"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    ApplicationContextItem = a_associate_rq::application_context::ITEM_TYPE as isize,
    PresentationContextItemInAAssociateRq =
        a_associate_rq::presentation_context::ITEM_TYPE as isize,
    PresentationContextItemInAAssociateAc =
        a_associate_ac::presentation_context::ITEM_TYPE as isize,
    AbstractSyntaxSubItem =
        a_associate_rq::presentation_context::abstract_syntax::ITEM_TYPE as isize,
    TransferSyntaxSubItem = a_associate::presentation_context::transfer_syntax::ITEM_TYPE as isize,
    UserInformationItem = a_associate::user_information::ITEM_TYPE as isize,
    MaximumLengthSubItem = a_associate::user_information::maximum_length::ITEM_TYPE as isize,
    ImplementationClassUidSubItem =
        a_associate::user_information::implementation_class_uid::ITEM_TYPE as isize,
    AsynchronousOperationsWindowSubItem = 0x53,
    ScpScuRoleSelectionSubItem = 0x54,
    ImplementationVersionNameSubItem =
        a_associate::user_information::implementation_version_name::ITEM_TYPE as isize,
    SopClassExtendedNegotiationSubItem = 0x56,
    SopClassCommonExtendedNegotiationSubItem = 0x57,
    UserIdentitySubItemInAAssociateRq = 0x58,
    UserIdentitySubItemInAAssociateAc = 0x59,
}

impl ItemType {
    pub(crate) async fn read_from_stream(
        buf_reader: &mut tokio::io::BufReader<impl tokio::io::AsyncRead + Unpin>,
    ) -> Result<Self, PduReadError> {
        use tokio::io::AsyncReadExt;

        let b = buf_reader.read_u8().await?;
        ItemType::try_from(b).map_err(|_| PduReadError::UnrecognizedPduParameter(b))
    }
}

impl TryFrom<u8> for ItemType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            a_associate_rq::application_context::ITEM_TYPE => Ok(Self::ApplicationContextItem),
            a_associate_rq::presentation_context::ITEM_TYPE => {
                Ok(Self::PresentationContextItemInAAssociateRq)
            }
            a_associate_ac::presentation_context::ITEM_TYPE => {
                Ok(Self::PresentationContextItemInAAssociateAc)
            }
            a_associate_rq::presentation_context::abstract_syntax::ITEM_TYPE => {
                Ok(Self::AbstractSyntaxSubItem)
            }
            a_associate::presentation_context::transfer_syntax::ITEM_TYPE => {
                Ok(Self::TransferSyntaxSubItem)
            }
            a_associate::user_information::ITEM_TYPE => Ok(Self::UserInformationItem),
            a_associate::user_information::maximum_length::ITEM_TYPE => {
                Ok(Self::MaximumLengthSubItem)
            }
            a_associate::user_information::implementation_class_uid::ITEM_TYPE => {
                Ok(Self::ImplementationClassUidSubItem)
            }
            0x53 => Ok(Self::AsynchronousOperationsWindowSubItem),
            0x54 => Ok(Self::ScpScuRoleSelectionSubItem),
            a_associate::user_information::implementation_version_name::ITEM_TYPE => {
                Ok(Self::ImplementationVersionNameSubItem)
            }
            0x56 => Ok(Self::SopClassExtendedNegotiationSubItem),
            0x57 => Ok(Self::SopClassCommonExtendedNegotiationSubItem),
            0x58 => Ok(Self::UserIdentitySubItemInAAssociateRq),
            0x59 => Ok(Self::UserIdentitySubItemInAAssociateAc),
            _ => Err("不正なItem-typeです"),
        }
    }
}

pub async fn receive_a_associate_rq(
    buf_reader: &mut tokio::io::BufReader<impl tokio::io::AsyncRead + Unpin>,
) -> Result<AAssociateRq, PduReadError> {
    use tokio::io::AsyncReadExt;

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
            message: format!("A-ASSOCIATE-RQのパースに失敗しました: {e:?}"),
        }),
    }
}

pub enum PDataTfReception {
    PDataTf(PDataTf),
    AAbort(AAbort),
}
pub async fn receive_p_data_tf(
    buf_reader: &mut tokio::io::BufReader<impl tokio::io::AsyncRead + Unpin>,
) -> Result<PDataTfReception, PduReadError> {
    use tokio::io::AsyncReadExt;

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
                message: format!("P-DATA-TFのパースに失敗しました: {e:?}"),
            }),
        }
    } else {
        match AAbort::read_from_stream(buf_reader, pdu_length).await {
            Ok(val) => Ok(PDataTfReception::AAbort(val)),
            Err(e) => Err(PduReadError::InvalidPduParameterValue {
                message: format!("A-ABORTのパースに失敗しました: {e:?}"),
            }),
        }
    }
}

pub enum AReleaseRqReception {
    AReleaseRq(AReleaseRq),
    AAbort(AAbort),
}
pub async fn receive_a_release_rq(
    buf_reader: &mut tokio::io::BufReader<impl tokio::io::AsyncRead + Unpin>,
) -> Result<AReleaseRqReception, PduReadError> {
    use tokio::io::AsyncReadExt;

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
                message: format!("A-RELEASE-RQのパースに失敗しました: {e:?}"),
            }),
        }
    } else {
        match AAbort::read_from_stream(buf_reader, pdu_length).await {
            Ok(val) => Ok(AReleaseRqReception::AAbort(val)),
            Err(e) => Err(PduReadError::InvalidPduParameterValue {
                message: format!("A-ABORTのパースに失敗しました: {e:?}"),
            }),
        }
    }
}

pub async fn send_a_associate_ac<T>(
    socket: &mut T,
    called_ae_title: &str,
    calling_ae_title: &str,
    application_context: a_associate::ApplicationContext,
    presentation_contexts: Vec<a_associate_ac::PresentationContext>,
    user_information: a_associate::UserInformation,
) -> std::io::Result<()>
where
    T: tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::AsyncWriteExt;

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

pub async fn send_p_data_tf<T>(socket: &mut T, p_data_tf_pdus: &[PDataTf]) -> std::io::Result<()>
where
    T: tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::AsyncWriteExt;

    for p_data_tf in p_data_tf_pdus {
        let bytes: Vec<u8> = p_data_tf.into();
        socket.write_all(&bytes).await?;
    }

    Ok(())
}

pub async fn send_a_release_rp<T>(socket: &mut T) -> std::io::Result<()>
where
    T: tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::AsyncWriteExt;

    let a_release_rp = AReleaseRp::new();

    let bytes: Vec<u8> = a_release_rp.into();
    socket.write_all(&bytes).await?;

    Ok(())
}

pub async fn send_a_abort<T>(
    socket: &mut T,
    source: a_abort::Source,
    reason: a_abort::Reason,
) -> std::io::Result<()>
where
    T: tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::AsyncWriteExt;

    let a_abort = AAbort::new(source, reason);

    let bytes: Vec<u8> = a_abort.into();
    socket.write_all(&bytes).await?;

    Ok(())
}
