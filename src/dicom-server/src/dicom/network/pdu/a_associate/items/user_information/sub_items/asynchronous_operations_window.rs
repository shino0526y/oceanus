use crate::dicom::network::pdu::a_associate::items::{
    INVALID_ITEM_LENGTH_ERROR_MESSAGE, INVALID_ITEM_TYPE_ERROR_MESSAGE, Item,
};

pub const ITEM_TYPE: u8 = 0x53;

pub struct AsynchronousOperationsWindow {
    maximum_number_operations_invoked: u16,
    maximum_number_operations_performed: u16,
}

impl AsynchronousOperationsWindow {
    pub fn size(&self) -> usize {
        8
    }

    pub fn length(&self) -> u32 {
        4
    }

    pub fn maximum_number_operations_invoked(&self) -> u16 {
        self.maximum_number_operations_invoked
    }

    pub fn maximum_number_operations_performed(&self) -> u16 {
        self.maximum_number_operations_performed
    }
}

impl TryFrom<&[u8]> for AsynchronousOperationsWindow {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let item = Item::try_from(bytes)?;
        if item.item_type != ITEM_TYPE {
            return Err(INVALID_ITEM_TYPE_ERROR_MESSAGE);
        }
        if item.length != 4 {
            return Err(INVALID_ITEM_LENGTH_ERROR_MESSAGE);
        }

        let maximum_number_operations_invoked = u16::from_be_bytes([item.data[0], item.data[1]]);
        let maximum_number_operations_performed = u16::from_be_bytes([item.data[2], item.data[3]]);

        Ok(AsynchronousOperationsWindow {
            maximum_number_operations_invoked,
            maximum_number_operations_performed,
        })
    }
}
