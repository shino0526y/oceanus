use crate::dicom::core::Tag;

pub struct Command {
    pub(crate) tag: Tag,
    pub(crate) value_length: u32,
    pub(crate) value_field: Vec<u8>,
}

impl Command {
    pub fn tag(&self) -> Tag {
        self.tag
    }

    pub fn value_length(&self) -> u32 {
        self.value_length
    }

    pub fn value_field(&self) -> &[u8] {
        &self.value_field
    }

    pub fn size(&self) -> usize {
        8 + self.value_length as usize
    }
}
