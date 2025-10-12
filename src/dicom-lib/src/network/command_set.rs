pub mod command;
pub mod utils;

pub use command::Command;

use crate::core::Tag;
use std::{ops::Index, slice::Iter};

const INVALID_BUFFER_LENGTH_ERROR_MESSAGE: &str = "バッファの長さが不正です";

pub struct CommandSet {
    pub(crate) size: usize,
    pub(crate) commands: Vec<Command>,
}

impl CommandSet {
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn iter(&self) -> Iter<'_, Command> {
        self.commands.iter()
    }

    pub fn new(commands: Vec<Command>) -> Result<Self, &'static str> {
        if commands.is_empty() {
            return Err("コマンドセットは少なくとも1つのコマンドを含む必要があります");
        }

        let mut size = commands[0].size();
        for i in 1..commands.len() {
            if commands[i - 1].tag() >= commands[i].tag() {
                return Err("コマンドはタグの昇順で並んでいる必要があります");
            }
            size += commands[i].size();
        }

        Ok(Self { size, commands })
    }
}

impl Index<usize> for CommandSet {
    type Output = Command;

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

impl TryFrom<Vec<u8>> for CommandSet {
    type Error = String;

    fn try_from(buf: Vec<u8>) -> Result<Self, Self::Error> {
        let mut commands = vec![];

        let mut offset = 0;
        while offset < buf.len() {
            if buf.len() < offset + 8 {
                return Err(INVALID_BUFFER_LENGTH_ERROR_MESSAGE.to_string());
            }

            let tag_group = u16::from_le_bytes([buf[offset], buf[offset + 1]]);
            if tag_group != 0x0000 {
                return Err(format!(
                    "タググループは0x0000でなければなりません (タググループ=0x{tag_group:04X})"
                ));
            }
            let tag_element = u16::from_le_bytes([buf[offset + 2], buf[offset + 3]]);
            let value_length = u32::from_le_bytes([
                buf[offset + 4],
                buf[offset + 5],
                buf[offset + 6],
                buf[offset + 7],
            ]);
            let value_field = buf[offset + 8..offset + 8 + value_length as usize].to_vec();
            if value_length as usize != value_field.len() {
                return Err(INVALID_BUFFER_LENGTH_ERROR_MESSAGE.to_string());
            }

            let command = Command {
                tag: Tag::new(tag_group, tag_element),
                value_field,
            };

            offset += command.size();
            commands.push(command);
        }
        let size = buf.len();

        Ok(CommandSet { size, commands })
    }
}

impl From<CommandSet> for Vec<u8> {
    fn from(val: CommandSet) -> Self {
        let mut buf = Vec::with_capacity(val.size());

        for command in val.commands {
            buf.append(&mut command.into());
        }

        buf
    }
}
