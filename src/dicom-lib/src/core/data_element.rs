use super::encoding::Encoding;
use crate::core::tag::Tag;

pub struct DataElement {
    tag: Tag,
    vr: Option<String>,
    value_length: u32,
    value_field: Vec<u8>,
    encoding: Encoding,
    size: u64,
}

impl DataElement {
    pub fn tag(&self) -> Tag {
        self.tag
    }

    pub fn vr(&self) -> Option<&str> {
        self.vr.as_deref()
    }

    pub fn value_length(&self) -> u32 {
        self.value_length
    }

    pub fn value_field(&self) -> &[u8] {
        &self.value_field
    }

    pub fn encoding(&self) -> Encoding {
        self.encoding
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub(crate) fn new(
        tag: Tag,
        vr: Option<String>,
        value_length: u32,
        value_field: Vec<u8>,
        encoding: Encoding,
        size: u64,
    ) -> Self {
        Self {
            tag,
            vr,
            value_length,
            value_field,
            encoding,
            size,
        }
    }
}
