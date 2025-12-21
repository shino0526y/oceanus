/// 拒否: リソース不足 ステータスコード (0xa700〜0xa7ff)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OutOfResources(pub(super) u16);

impl OutOfResources {
    pub fn new(code: u16) -> Result<Self, String> {
        if (0xa700..=0xa7ff).contains(&code) {
            Ok(Self(code))
        } else {
            Err(format!(
                "コードは0xa700〜0xa7ffの範囲である必要があります (コード={code:#06X})"
            ))
        }
    }

    pub fn get(&self) -> u16 {
        self.0
    }
}

/// エラー: データセットがSOPクラスと一致しない ステータスコード (0xa900〜0xa9ff)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DataSetMismatch(pub(super) u16);

impl DataSetMismatch {
    pub fn new(code: u16) -> Result<Self, String> {
        if (0xa900..=0xa9ff).contains(&code) {
            Ok(Self(code))
        } else {
            Err(format!(
                "コードは0xa900〜0xa9ffの範囲である必要があります (コード={code:#06X})"
            ))
        }
    }

    pub fn get(&self) -> u16 {
        self.0
    }
}

/// エラー: 理解できない ステータスコード (0xc000〜0xcfff)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CannotUnderstand(pub(super) u16);

impl CannotUnderstand {
    pub fn new(code: u16) -> Result<Self, String> {
        if (0xc000..=0xcfff).contains(&code) {
            Ok(Self(code))
        } else {
            Err(format!(
                "コードは0xc000〜0xcfffの範囲である必要があります (コード={code:#06X})"
            ))
        }
    }

    pub fn get(&self) -> u16 {
        self.0
    }
}
