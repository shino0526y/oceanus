use crate::core::Tag;

pub struct MetaDataElement {
    pub(super) tag: Tag,
    pub(super) vr: &'static str,
    pub(super) value_length: u32,
    pub(super) value_field: Vec<u8>,
}

impl MetaDataElement {
    pub fn tag(&self) -> Tag {
        self.tag
    }

    pub fn vr(&self) -> &str {
        self.vr
    }

    pub fn value_length(&self) -> u32 {
        self.value_length
    }

    pub fn value_field(&self) -> &[u8] {
        &self.value_field
    }

    pub fn size(&self) -> usize {
        4 // タグ
        + 2 // VR
        + match self.vr {
            "AE" | "AS" | "AT" | "CS" | "DA" | "DS" | "DT" | "FL" | "FD" | "IS" | "LO" | "LT"
            | "PN" | "SH" | "SL" | "SS" | "ST" | "TM" | "UI" | "UL" | "US" => {
                2 // Value Length
            }
            _ => {
                2 // Reserved
                + 4 // Value Length
            },
        }
        + self.value_field.len() // Value Field
    }
}

impl From<MetaDataElement> for Vec<u8> {
    fn from(mut v: MetaDataElement) -> Vec<u8> {
        let mut buf = Vec::with_capacity(v.size());

        buf.extend_from_slice(&v.tag.into() as &[u8; 4]); // Tag
        buf.extend_from_slice(v.vr.as_bytes()); // VR
        match v.vr {
            "AE" | "AS" | "AT" | "CS" | "DA" | "DS" | "DT" | "FL" | "FD" | "IS" | "LO" | "LT"
            | "PN" | "SH" | "SL" | "SS" | "ST" | "TM" | "UI" | "UL" | "US" => {
                buf.extend_from_slice(&(v.value_length as u16).to_le_bytes()); // Value Length
            }
            _ => {
                buf.extend_from_slice(&[0x00, 0x00]); // Reserved
                buf.extend_from_slice(&v.value_length.to_le_bytes()); // Value Length
            }
        }
        buf.append(&mut v.value_field); // Value Field

        buf
    }
}
