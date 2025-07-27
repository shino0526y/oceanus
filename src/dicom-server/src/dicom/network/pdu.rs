mod a_associate;
pub mod a_associate_rq;

const INVALID_FIELD_LENGTH_ERROR_MESSAGE: &'static str = "フィールドの長さが不正です";
const INVALID_PDU_LENGTH_ERROR_MESSAGE: &'static str = "PDU-length が不正です";
const INVALID_PDU_TYPE_ERROR_MESSAGE: &'static str = "PDU-type が不正です";

struct Pdu<'a> {
    pdu_type: u8,
    length: u32,
    data: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for Pdu<'a> {
    type Error = &'static str;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        if bytes.len() < 6 {
            return Err(INVALID_FIELD_LENGTH_ERROR_MESSAGE);
        }

        let pdu_type = bytes[0];
        let length = u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
        if bytes.len() < (6 + length as usize) {
            return Err(INVALID_PDU_LENGTH_ERROR_MESSAGE);
        }

        Ok(Pdu {
            pdu_type,
            length,
            data: &bytes[6..(6 + length as usize)],
        })
    }
}
