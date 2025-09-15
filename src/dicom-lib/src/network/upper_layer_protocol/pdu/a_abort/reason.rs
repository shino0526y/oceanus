use crate::network::upper_layer_protocol::pdu;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    ReasonNotSpecified = 0,
    UnrecognizedPdu = 1,
    UnexpectedPdu = 2,
    Reserved = 3,
    UnrecognizedPduParameter = 4,
    UnexpectedPduParameter = 5,
    InvalidPduParameterValue = 6,
}

impl TryFrom<u8> for Reason {
    type Error = String;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Reason::ReasonNotSpecified),
            1 => Ok(Reason::UnrecognizedPdu),
            2 => Ok(Reason::UnexpectedPdu),
            3 => Ok(Reason::Reserved),
            4 => Ok(Reason::UnrecognizedPduParameter),
            5 => Ok(Reason::UnexpectedPduParameter),
            6 => Ok(Reason::InvalidPduParameterValue),
            _ => Err(format!("未定義のReason/Diag.です (Reason=0x{val:02X})")),
        }
    }
}

impl From<pdu::PduReadError> for Reason {
    fn from(err: pdu::PduReadError) -> Self {
        match err {
            pdu::PduReadError::UnrecognizedPdu(_) => Reason::UnrecognizedPdu,
            pdu::PduReadError::UnexpectedPdu(_) => Reason::UnexpectedPdu,
            pdu::PduReadError::UnrecognizedPduParameter(_) => Reason::UnrecognizedPduParameter,
            pdu::PduReadError::UnexpectedPduParameter(_) => Reason::UnexpectedPduParameter,
            pdu::PduReadError::InvalidPduParameterValue { message: _ } => {
                Reason::InvalidPduParameterValue
            }
            pdu::PduReadError::IoError(_) => Reason::ReasonNotSpecified,
        }
    }
}
