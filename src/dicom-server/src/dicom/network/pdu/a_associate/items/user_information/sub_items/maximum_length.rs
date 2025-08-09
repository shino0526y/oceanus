use crate::dicom::network::pdu::a_associate::items::{
    INVALID_ITEM_LENGTH_ERROR_MESSAGE, INVALID_ITEM_TYPE_ERROR_MESSAGE, Item,
};

pub const ITEM_TYPE: u8 = 0x51;

pub struct MaximumLength {
    maximum_length: u32,
}

impl MaximumLength {
    pub fn size(&self) -> usize {
        8
    }

    pub fn length(&self) -> u16 {
        4
    }

    pub fn maximum_length(&self) -> u32 {
        self.maximum_length
    }
}

impl TryFrom<&[u8]> for MaximumLength {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let item = Item::try_from(bytes)?;
        if item.item_type != ITEM_TYPE {
            return Err(INVALID_ITEM_TYPE_ERROR_MESSAGE);
        }
        if item.length != 4 {
            return Err(INVALID_ITEM_LENGTH_ERROR_MESSAGE);
        }

        let maximum_length =
            u32::from_be_bytes([item.data[0], item.data[1], item.data[2], item.data[3]]);

        Ok(MaximumLength { maximum_length })
    }
}
