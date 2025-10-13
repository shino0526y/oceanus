use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Priority {
    Low = 0x0002,
    Medium = 0x0000,
    High = 0x0001,
}

impl TryFrom<u16> for Priority {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0002 => Ok(Priority::Low),
            0x0000 => Ok(Priority::Medium),
            0x0001 => Ok(Priority::High),
            _ => Err("値は0x0000・0x0001・0x0002のいずれかでなければなりません"),
        }
    }
}

impl Display for Priority {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Priority::Low => "低",
            Priority::Medium => "中",
            Priority::High => "高",
        };
        write!(f, "{s}")
    }
}
