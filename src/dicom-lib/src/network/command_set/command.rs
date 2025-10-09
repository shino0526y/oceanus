use crate::core::Tag;

pub struct Command {
    pub(crate) tag: Tag,
    pub(crate) value_field: Vec<u8>,
}

impl Command {
    pub fn tag(&self) -> Tag {
        self.tag
    }

    pub fn value_length(&self) -> u32 {
        self.value_field.len() as u32
    }

    pub fn value_field(&self) -> &[u8] {
        &self.value_field
    }

    pub fn size(&self) -> usize {
        8 + self.value_length() as usize
    }
}

impl From<Command> for Vec<u8> {
    fn from(mut command: Command) -> Self {
        let mut bytes = Vec::with_capacity(command.size());

        bytes.append(&mut command.tag().into());
        bytes.extend(command.value_length().to_le_bytes());
        bytes.append(&mut command.value_field);

        bytes
    }
}
