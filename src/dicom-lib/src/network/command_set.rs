pub mod command;
pub mod utils;

pub use command::Command;

use crate::core::Tag;
use std::{
    io::{Cursor, Read},
    ops::Index,
    slice::Iter,
};

pub struct CommandSet {
    pub(crate) size: usize,
    pub(crate) commands: Vec<Command>,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("I/Oエラーが発生しました: {0}")]
    IoError(#[from] std::io::Error),

    #[error("タググループが0000ではありません (タグ={0})")]
    InvalidTagGroup(Tag),
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

    pub fn from_cur(cur: &mut Cursor<&[u8]>) -> Result<Self, ParseError> {
        let mut commands = vec![];

        let size = cur.get_ref().len();
        let mut offset = 0;
        while offset < size {
            let tag = Tag::from_cur(cur)?;
            if tag.group() != 0x0000 {
                return Err(ParseError::InvalidTagGroup(tag));
            }
            let value_length = {
                let mut buf = [0u8; 4];
                cur.read_exact(&mut buf)?;
                u32::from_le_bytes(buf)
            };
            let value_field = {
                let mut buf = vec![0u8; value_length as usize];
                cur.read_exact(&mut buf)?;
                buf
            };

            let command = Command { tag, value_field };

            offset += command.size();
            commands.push(command);
        }

        Ok(CommandSet { size, commands })
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

impl From<CommandSet> for Vec<u8> {
    fn from(val: CommandSet) -> Self {
        let mut buf = Vec::with_capacity(val.size());

        for command in val.commands {
            buf.append(&mut command.into());
        }

        buf
    }
}
