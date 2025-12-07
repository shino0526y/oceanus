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

impl From<DataElement> for Vec<u8> {
    fn from(mut v: DataElement) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(v.size as usize);

        match v.encoding {
            Encoding::ImplicitVrLittleEndian => {
                bytes.extend_from_slice(&v.tag.into() as &[u8; 4]); // Tag
                bytes.extend_from_slice(&v.value_length.to_le_bytes()); // Value Length
                bytes.append(&mut v.value_field); // Value Field
            }

            Encoding::ExplicitVrLittleEndian => {
                debug_assert!(v.vr.is_some());
                let vr = v.vr.as_ref().unwrap();
                debug_assert!(vr.is_empty() || vr.len() == 2);

                bytes.extend_from_slice(&v.tag.into() as &[u8; 4]); // Tag
                bytes.extend_from_slice(vr.as_bytes()); // VR
                match vr.as_str() {
                    "" => {
                        bytes.extend_from_slice(&v.value_length.to_le_bytes()); // Value Length
                    }
                    "AE" | "AS" | "AT" | "CS" | "DA" | "DS" | "DT" | "FL" | "FD" | "IS" | "LO"
                    | "LT" | "PN" | "SH" | "SL" | "SS" | "ST" | "TM" | "UI" | "UL" | "US" => {
                        bytes.extend_from_slice(&(v.value_length as u16).to_le_bytes()); // Value Length
                    }
                    _ => {
                        bytes.extend_from_slice(&[0x00, 0x00]); // Reserved
                        bytes.extend_from_slice(&v.value_length.to_le_bytes()); // Value Length
                    }
                }
                bytes.append(&mut v.value_field); // Value Field
            }

            Encoding::ExplicitVrBigEndian => unimplemented!(),
        }

        bytes
    }
}
