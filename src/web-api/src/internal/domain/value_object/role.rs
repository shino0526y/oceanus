#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// 管理者(開発者): システム全域の操作、内部ログの閲覧
    Admin = 0,
    /// 病院の情シス: ユーザー管理、運用ログのチェック
    ItStaff = 1,
    /// 医師: 診療行為、電子カルテの操作
    Doctor = 2,
    /// 技師: 検査実施、検査データの登録
    Technician = 3,
    /// 事務員・受付: 受付業務、会計、基本情報の登録
    Clerk = 4,
}

impl Role {
    pub fn from_i16(value: i16) -> Result<Self, String> {
        match value {
            0 => Ok(Self::Admin),
            1 => Ok(Self::ItStaff),
            2 => Ok(Self::Doctor),
            3 => Ok(Self::Technician),
            4 => Ok(Self::Clerk),
            _ => Err(format!("不正な職種です: {}", value)),
        }
    }

    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn as_i16(&self) -> i16 {
        *self as i16
    }
}
