use std::{
    fmt::{Display, Formatter},
    io::{Cursor, Read},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tag(pub u16, pub u16);

impl Tag {
    pub fn group(self) -> u16 {
        self.0
    }

    pub fn element(self) -> u16 {
        self.1
    }

    pub(crate) fn from_cur(cur: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let tag_group = {
            let mut buf = [0u8; 2];
            cur.read_exact(&mut buf)?;
            u16::from_le_bytes(buf)
        };
        let tag_element = {
            let mut buf = [0u8; 2];
            cur.read_exact(&mut buf)?;
            u16::from_le_bytes(buf)
        };

        Ok(Tag(tag_group, tag_element))
    }
}

impl std::fmt::Debug for Tag {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Tag({:#06x?}, {:#06x?})", self.0, self.1)
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({:04X},{:04X})", self.0, self.1)
    }
}

impl From<Tag> for Vec<u8> {
    fn from(tag: Tag) -> Self {
        let mut bytes = Vec::with_capacity(4);

        bytes.extend(tag.group().to_le_bytes());
        bytes.extend(tag.element().to_le_bytes());

        bytes
    }
}

impl From<Tag> for [u8; 4] {
    fn from(v: Tag) -> [u8; 4] {
        let mut bytes = [0u8; 4];
        bytes[0..2].copy_from_slice(&v.group().to_le_bytes());
        bytes[2..4].copy_from_slice(&v.element().to_le_bytes());
        bytes
    }
}
