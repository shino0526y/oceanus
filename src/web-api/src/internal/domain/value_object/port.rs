#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Port(u16);

impl Port {
    pub fn value(&self) -> u16 {
        self.0
    }

    pub fn from_u16(value: u16) -> Result<Self, String> {
        if value == 0 {
            return Err(format!(
                "ポート番号は1〜65535の範囲である必要があります (指定された値={value})"
            ));
        }
        Ok(Self(value))
    }

    pub fn from_i32(value: i32) -> Result<Self, String> {
        if !(1..=65535).contains(&value) {
            return Err(format!(
                "ポート番号は1〜65535の範囲である必要があります (指定された値={value})"
            ));
        }
        Ok(Self(value as u16))
    }
}
