pub mod application_context;
pub mod presentation_context;
pub mod user_information;

use crate::dicom::network::upper_layer_protocol::pdu::INVALID_FIELD_LENGTH_ERROR_MESSAGE;
pub use application_context::ApplicationContext;
pub use user_information::UserInformation;

pub(crate) const INVALID_ITEM_TYPE_ERROR_MESSAGE: &str = "Item-type が不正です";
const INVALID_ITEM_LENGTH_ERROR_MESSAGE: &str = "Item-length が不正です";

pub(crate) struct Item<'a> {
    pub item_type: u8,
    pub length: u16,
    pub data: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for Item<'a> {
    type Error = &'static str;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        if bytes.len() < 4 {
            return Err(INVALID_FIELD_LENGTH_ERROR_MESSAGE);
        }

        let item_type = bytes[0];
        let length = u16::from_be_bytes([bytes[2], bytes[3]]);
        if bytes.len() < (4 + length as usize) {
            return Err(INVALID_ITEM_LENGTH_ERROR_MESSAGE);
        }

        Ok(Item {
            item_type,
            length,
            data: &bytes[4..(4 + length as usize)],
        })
    }
}
