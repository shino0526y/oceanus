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

pub(crate) const INVALID_PDU_LENGTH_ERROR_MESSAGE: &str = "PDU-length が不正です";

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
            _ => Err("PDU タイプが不正です"),
        }
    }
}
