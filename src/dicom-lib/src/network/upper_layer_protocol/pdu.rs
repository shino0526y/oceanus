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
    #[error("フォーマットが不正です: {message}")]
    InvalidFormat { message: String },
    #[error("データの終端に予期せず到達しました")]
    UnexpectedEndOfBuffer,
    #[error("I/Oエラーが発生しました")]
    IoError(std::io::Error),
}

impl From<std::io::Error> for PduReadError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::UnexpectedEof => PduReadError::UnexpectedEndOfBuffer,
            _ => PduReadError::IoError(e),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
            a_associate_rq::PDU_TYPE => Ok(PduType::AAssociateRq),
            a_associate_ac::PDU_TYPE => Ok(PduType::AAssociateAc),
            0x03 => Ok(PduType::AAssociateRj),
            p_data_tf::PDU_TYPE => Ok(PduType::PDataTf),
            a_release_rq::PDU_TYPE => Ok(PduType::AReleaseRq),
            a_release_rp::PDU_TYPE => Ok(PduType::AReleaseRp),
            a_abort::PDU_TYPE => Ok(PduType::AAbort),
            _ => Err("PDU-typeが不正です"),
        }
    }
}
