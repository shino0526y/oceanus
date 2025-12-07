use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tag(pub u16, pub u16);

impl Tag {
    pub fn group(self) -> u16 {
        self.0
    }

    pub fn element(self) -> u16 {
        self.1
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

impl Into<[u8; 4]> for Tag {
    fn into(self) -> [u8; 4] {
        let mut bytes = [0u8; 4];
        bytes[0..2].copy_from_slice(&self.group().to_le_bytes());
        bytes[2..4].copy_from_slice(&self.element().to_le_bytes());
        bytes
    }
}
