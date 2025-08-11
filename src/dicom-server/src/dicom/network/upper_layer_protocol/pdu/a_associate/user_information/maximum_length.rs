use crate::dicom::network::upper_layer_protocol::pdu::a_associate::{
    INVALID_ITEM_LENGTH_ERROR_MESSAGE, INVALID_ITEM_TYPE_ERROR_MESSAGE, Item,
};

pub(crate) const ITEM_TYPE: u8 = 0x51;

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

    pub fn new(maximum_length: u32) -> Self {
        Self { maximum_length }
    }
}

impl From<MaximumLength> for Vec<u8> {
    fn from(val: MaximumLength) -> Self {
        let mut bytes = Vec::with_capacity(val.size());

        bytes.push(ITEM_TYPE);
        bytes.push(0); // Reserved
        bytes.extend(4u16.to_be_bytes());
        bytes.extend(val.maximum_length.to_be_bytes());

        bytes
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
