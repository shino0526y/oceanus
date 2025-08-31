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
