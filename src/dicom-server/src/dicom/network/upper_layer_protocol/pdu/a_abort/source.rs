#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Source {
    User = 0,
    Reserved = 1,
    Provider = 2,
}

impl TryFrom<u8> for Source {
    type Error = String;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Source::User),
            1 => Ok(Source::Reserved),
            2 => Ok(Source::Provider),
            _ => Err(format!("未定義の Source です (source=0x{val:02X})")),
        }
    }
}
