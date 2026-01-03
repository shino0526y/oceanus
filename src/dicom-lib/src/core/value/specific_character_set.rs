pub(super) mod encoding;
pub(super) mod iso_2022_ir_13_and_iso_2022_ir_87;
pub(super) mod iso_2022_ir_6_and_iso_2022_ir_13_and_iso_2022_ir_87;
pub(super) mod iso_2022_ir_6_and_iso_2022_ir_87;
pub(super) mod iso_ir_13;
pub(super) mod iso_ir_192;
pub(super) mod none;

use crate::constants::specific_character_sets::{
    ISO_2022_IR_6, ISO_2022_IR_13, ISO_2022_IR_87, ISO_IR_13, ISO_IR_192, NONE,
};
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecificCharacterSet {
    /// 1バイト コード拡張なし デフォルト文字セット（ASCII）
    ///
    /// 対応する (0008,0005) Specific Character Set の値は`r""`。
    None,

    /// 1バイト コード拡張なし 日本文字セット（半角カタカナ＆ローマ字）
    ///
    /// 対応する (0008,0005) Specific Character Set の値は`r"ISO_IR 13"`。
    IsoIr13,

    /// マルチバイト コード拡張なし Unicode（UTF-8）文字セット
    ///
    /// 対応する (0008,0005) Specific Character Set の値は`r"ISO_IR 192"`。
    IsoIr192,

    /// 以下の文字セットの組み合わせ
    /// - 1バイト コード拡張あり デフォルト文字セット（ASCII）
    /// - マルチバイト コード拡張あり 日本文字セット（JIS漢字）
    ///
    /// 対応する (0008,0005) Specific Character Set の値は`r"ISO 2022 IR 6\ISO 2022 IR 87"`。
    Iso2022Ir6AndIso2022Ir87,

    /// 以下の文字セットの組み合わせ
    /// - 1バイト コード拡張あり 日本文字セット（半角カタカナ＆ローマ字）
    /// - マルチバイト コード拡張あり 日本文字セット（JIS漢字）
    ///
    /// 対応する (0008,0005) Specific Character Set の値は`r"ISO 2022 IR 13\ISO 2022 IR 87"`。
    Iso2022Ir13AndIso2022Ir87,

    /// 以下の文字セットの組み合わせ
    /// - 1バイト コード拡張あり デフォルト文字セット（ASCII）
    /// - 1バイト コード拡張あり 日本文字セット（半角カタカナ＆ローマ字）
    /// - マルチバイト コード拡張あり 日本文字セット（JIS漢字）
    ///
    ///  対応する (0008,0005) Specific Character Set の値は`r"ISO 2022 IR 6\ISO 2022 IR 13\ISO 2022 IR 87"`。
    Iso2022Ir6AndIso2022Ir13AndIso2022Ir87,
}

impl Display for SpecificCharacterSet {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "{}", NONE),
            Self::IsoIr13 => write!(f, "{}", ISO_IR_13),
            Self::IsoIr192 => write!(f, "{}", ISO_IR_192),
            Self::Iso2022Ir6AndIso2022Ir87 => write!(f, "{}\\{}", ISO_2022_IR_6, ISO_2022_IR_87),
            Self::Iso2022Ir13AndIso2022Ir87 => write!(f, "{}\\{}", ISO_2022_IR_13, ISO_2022_IR_87),
            Self::Iso2022Ir6AndIso2022Ir13AndIso2022Ir87 => write!(
                f,
                "{}\\{}\\{}",
                ISO_2022_IR_6, ISO_2022_IR_13, ISO_2022_IR_87
            ),
        }
    }
}

impl TryFrom<&[u8]> for SpecificCharacterSet {
    type Error = String;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        use FromStr;
        SpecificCharacterSet::from_str(&String::from_utf8_lossy(bytes))
    }
}

impl FromStr for SpecificCharacterSet {
    type Err = String;

    /// 文字列からSpecificCharacterSetを生成する。
    ///
    /// # 例
    ///
    /// ```
    /// use dicom_lib::core::value::SpecificCharacterSet;
    /// use std::str::FromStr;
    ///
    /// let expected = Ok(SpecificCharacterSet::Iso2022Ir6AndIso2022Ir87);
    ///
    /// let result_1 = SpecificCharacterSet::from_str(r"\ISO 2022 IR 87");
    /// let result_2 = SpecificCharacterSet::from_str(r"\ISO 2022 IR 87 ");
    /// let result_3 = SpecificCharacterSet::from_str(r"ISO 2022 IR 6\ISO 2022 IR 87");
    ///
    /// assert_eq!(result_1, expected);
    /// assert_eq!(result_2, expected);
    /// assert_eq!(result_3, expected);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let terms = {
            let strs = s
                .split('\\')
                .map(|s| s.trim_end_matches(' '))
                .collect::<Vec<&str>>();

            match strs.len() {
                1 => strs,
                _ => {
                    assert!(!strs.is_empty());
                    let first_str = match *strs.first().unwrap() {
                        "" => ISO_2022_IR_6,
                        s => s,
                    };
                    let mut subsequent_strs = strs[1..].to_vec();
                    subsequent_strs.sort();
                    [vec![first_str], subsequent_strs].concat()
                }
            }
        };

        let err = Err(format!(
            "Specific Character Setとして使用できない文字列です (文字列=\"{s}\")"
        ));
        match terms.len() {
            1 => match terms[0] {
                NONE => Ok(Self::None),
                ISO_IR_13 => Ok(Self::IsoIr13),
                ISO_IR_192 => Ok(Self::IsoIr192),
                _ => err,
            },
            2 => match (terms[0], terms[1]) {
                (ISO_2022_IR_6, ISO_2022_IR_87) => Ok(Self::Iso2022Ir6AndIso2022Ir87),
                (ISO_2022_IR_13, ISO_2022_IR_87) => Ok(Self::Iso2022Ir13AndIso2022Ir87),
                _ => err,
            },
            3 => match (terms[0], terms[1], terms[2]) {
                (ISO_2022_IR_6, ISO_2022_IR_13, ISO_2022_IR_87) => {
                    Ok(Self::Iso2022Ir6AndIso2022Ir13AndIso2022Ir87)
                }
                _ => err,
            },
            _ => {
                assert!(!terms.is_empty());
                err
            }
        }
    }
}
