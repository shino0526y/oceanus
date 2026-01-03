/// 警告 ステータスコード (0x0001, 0xb000〜0xbfff, 0x0107, 0x0116)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Warning(pub(crate) u16);

impl Warning {
    pub fn new(code: u16) -> Result<Self, String> {
        if code == 0x0001 || (0xb000..=0xbfff).contains(&code) || code == 0x0107 || code == 0x0116 {
            Ok(Self(code))
        } else {
            Err(format!(
                "コードは0x0001, 0xb000〜0xbfff, 0x0107, 0x0116のいずれかである必要があります (コード={code:#06X})",
            ))
        }
    }

    pub fn get(&self) -> u16 {
        self.0
    }
}

/// 拒否: リソース不足 ステータスコード (0xa700〜0xa7ff)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OutOfResources(pub(crate) u16);

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
pub struct DataSetDoesNotMatchSopClass(pub(crate) u16);

impl DataSetDoesNotMatchSopClass {
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
pub struct CannotUnderstand(pub(crate) u16);

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

/// その他の失敗 ステータスコード
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OtherFailure(pub(crate) u16);

impl OtherFailure {
    pub fn new(code: u16) -> Result<Self, String> {
        let is_valid = (0xa000..=0xafff).contains(&code)
            && !(0xa700..=0xa7ff).contains(&code)
            && !(0xa900..=0xa9ff).contains(&code)
            || (0xc000..=0xcfff).contains(&code)
            || ((0x0100..=0x01ff).contains(&code)
                && code != 0x0107
                && code != 0x0116
                && code != 0x0117
                && code != 0x0122
                && code != 0x0124)
            || ((0x0200..=0x02ff).contains(&code)
                && code != 0x0210
                && code != 0x0211
                && code != 0x0212);

        if is_valid {
            Ok(Self(code))
        } else {
            Err(format!("範囲外のコードです (コード={code:#06X})"))
        }
    }

    pub fn get(&self) -> u16 {
        self.0
    }
}
