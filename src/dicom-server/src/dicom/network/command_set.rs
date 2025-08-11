pub mod command;

use crate::dicom::core::Tag;
pub use command::Command;
use std::{ops::Index, slice::Iter};

const INVALID_BUFFER_LENGTH_ERROR_MESSAGE: &str = "バッファの長さが不正です";

pub struct CommandSet {
    commands: Vec<Command>,
}

impl CommandSet {
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    #[inline(always)]
    pub fn iter(&self) -> Iter<'_, Command> {
        self.commands.iter()
    }
}

impl Index<usize> for CommandSet {
    type Output = Command;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.commands[index]
    }
}

impl<'a> IntoIterator for &'a CommandSet {
    type Item = &'a Command;
    type IntoIter = Iter<'a, Command>;

    fn into_iter(self) -> Self::IntoIter {
        self.commands.iter()
    }
}

impl TryFrom<&[u8]> for CommandSet {
    type Error = String;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut commands = Vec::new();

        let mut offset = 0;
        while offset < bytes.len() {
            if bytes.len() < offset + 8 {
                return Err(INVALID_BUFFER_LENGTH_ERROR_MESSAGE.to_string());
            }

            let tag_group = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
            if tag_group != 0x0000 {
                return Err(format!(
                    "タググループは 0x0000 でなければなりませんが、0x{tag_group:0>4x} です"
                ));
            }
            let tag_element = u16::from_le_bytes([bytes[offset + 2], bytes[offset + 3]]);
            let value_length = u32::from_le_bytes([
                bytes[offset + 4],
                bytes[offset + 5],
                bytes[offset + 6],
                bytes[offset + 7],
            ]);
            let value_field = bytes[offset + 8..offset + 8 + value_length as usize].to_vec();
            if value_length as usize != value_field.len() {
                return Err(INVALID_BUFFER_LENGTH_ERROR_MESSAGE.to_string());
            }

            let command = Command {
                tag: Tag::new(tag_group, tag_element),
                value_length,
                value_field,
            };

            offset += command.size();
            commands.push(command);
        }

        Ok(CommandSet { commands })
    }
}
