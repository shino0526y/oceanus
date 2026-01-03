pub mod vr;

pub use vr::Vr;

use crate::core::tag::Tag;

pub struct DataElement {
    pub(crate) tag: Tag,
    pub(crate) vr: Option<Vr>,
    pub(crate) value_length: u32,
    pub(crate) value_field: Vec<u8>,
    pub(crate) size: usize,
}

impl DataElement {
    pub fn tag(&self) -> Tag {
        self.tag
    }

    pub fn vr(&self) -> Option<Vr> {
        self.vr
    }

    pub fn value_length(&self) -> u32 {
        self.value_length
    }

    pub fn value_field(&self) -> &[u8] {
        &self.value_field
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn new(tag: Tag, vr: Option<Vr>, value_length: u32, value_field: Vec<u8>) -> Self {
        let size = 4 // Tag
        + match vr {
            None => 4, // Value Length
            Some(vr) => {
                2 // VR
                + match vr {
                    Vr::Ae
                    | Vr::As
                    | Vr::At
                    | Vr::Cs
                    | Vr::Da
                    | Vr::Ds
                    | Vr::Dt
                    | Vr::Fl
                    | Vr::Fd
                    | Vr::Is
                    | Vr::Lo
                    | Vr::Lt
                    | Vr::Pn
                    | Vr::Sh
                    | Vr::Sl
                    | Vr::Ss
                    | Vr::St
                    | Vr::Tm
                    | Vr::Ui
                    | Vr::Ul
                    | Vr::Us => {
                        2 // Value Length
                    }
                    _ => {
                        2 // Reserved
                        + 4 // Value Length
                    }
                }
            },
        } + value_field.len();

        Self {
            tag,
            vr,
            value_length,
            value_field,
            size,
        }
    }
}

impl From<DataElement> for Vec<u8> {
    fn from(mut v: DataElement) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(v.size);

        match v.vr {
            None => {
                bytes.extend_from_slice(&v.tag.into() as &[u8; 4]); // Tag
                bytes.extend_from_slice(&v.value_length.to_le_bytes()); // Value Length
                bytes.append(&mut v.value_field); // Value Field
            }

            Some(vr) => {
                bytes.extend_from_slice(&v.tag.into() as &[u8; 4]); // Tag
                bytes.extend_from_slice(&vr.into() as &[u8; 2]); // VR
                match vr {
                    Vr::Ae
                    | Vr::As
                    | Vr::At
                    | Vr::Cs
                    | Vr::Da
                    | Vr::Ds
                    | Vr::Dt
                    | Vr::Fl
                    | Vr::Fd
                    | Vr::Is
                    | Vr::Lo
                    | Vr::Lt
                    | Vr::Pn
                    | Vr::Sh
                    | Vr::Sl
                    | Vr::Ss
                    | Vr::St
                    | Vr::Tm
                    | Vr::Ui
                    | Vr::Ul
                    | Vr::Us => {
                        bytes.extend_from_slice(&(v.value_length as u16).to_le_bytes()); // Value Length
                    }
                    _ => {
                        bytes.extend_from_slice(&[0x00, 0x00]); // Reserved
                        bytes.extend_from_slice(&v.value_length.to_le_bytes()); // Value Length
                    }
                }
                bytes.append(&mut v.value_field); // Value Field
            }
        }

        bytes
    }
}
