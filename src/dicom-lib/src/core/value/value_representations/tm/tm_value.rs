use chrono::{NaiveTime, Timelike};
use std::{
    fmt::{Display, Formatter},
    str::Utf8Error,
};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct TmValue(NaiveTime);

impl TmValue {
    const MAX_BYTE_LENGTH: usize = 14; // HHMMSS.FFFFFF

    pub fn time(&self) -> &NaiveTime {
        &self.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TmValueError> {
        let str = str::from_utf8(bytes).map_err(TmValueError::InvalidUtf8)?;
        Self::from_string(str)
    }

    pub fn from_string(str: &str) -> Result<Self, TmValueError> {
        if str.len() > Self::MAX_BYTE_LENGTH {
            return Err(TmValueError::InvalidLength {
                string: str.to_string(),
                byte_length: str.len(),
            });
        }

        let trimmed = str.trim_end_matches(' ');
        if trimmed.is_empty() {
            return Err(TmValueError::Empty);
        }

        // 小数点が含まれる場合
        if trimmed.contains('.') {
            let parts: Vec<&str> = trimmed.split('.').collect();
            if parts.len() != 2 {
                return Err(TmValueError::InvalidFormat {
                    string: trimmed.to_string(),
                });
            }

            let time_part = parts[0];
            let frac_part = parts[1];

            // 時刻部分は2、4、または6文字(HH、HHMM、HHMMSS)
            if !matches!(time_part.len(), 2 | 4 | 6) {
                return Err(TmValueError::InvalidFormat {
                    string: trimmed.to_string(),
                });
            }

            // 小数部は1-6文字
            if frac_part.is_empty() || frac_part.len() > 6 {
                return Err(TmValueError::InvalidFormat {
                    string: trimmed.to_string(),
                });
            }

            // 時刻部分が数字のみであることを確認
            for (i, c) in time_part.chars().enumerate() {
                if !c.is_ascii_digit() {
                    return Err(TmValueError::InvalidCharacter {
                        string: trimmed.to_string(),
                        character: c,
                        position: i,
                    });
                }
            }

            // 小数部が数字のみであることを確認
            for (i, c) in frac_part.chars().enumerate() {
                if !c.is_ascii_digit() {
                    return Err(TmValueError::InvalidCharacter {
                        string: trimmed.to_string(),
                        character: c,
                        position: time_part.len() + 1 + i, // +1は小数点
                    });
                }
            }

            // 時刻部分の長さに応じて、秒と分を0で埋める
            let padded_time = match time_part.len() {
                2 => format!("{}0000", time_part), // HH -> HH0000
                4 => format!("{}00", time_part),   // HHMM -> HHMM00
                6 => time_part.to_string(),        // HHMMSS
                _ => unreachable!(),
            };

            // マイクロ秒まで拡張(6桁にパディング)
            let frac_padded = format!("{:0<6}", frac_part);
            let time_str = format!("{}.{}", padded_time, frac_padded);

            let time = NaiveTime::parse_from_str(&time_str, "%H%M%S%.f").map_err(|e| {
                TmValueError::ParseError {
                    string: trimmed.to_string(),
                    error: e,
                }
            })?;

            Ok(Self(time))
        } else {
            // 小数点なし: "HH"、"HHMM"、"HHMMSS"形式の検証
            if !matches!(trimmed.len(), 2 | 4 | 6) {
                return Err(TmValueError::InvalidFormat {
                    string: trimmed.to_string(),
                });
            }

            // 各文字が数字であることを確認
            for (i, c) in trimmed.chars().enumerate() {
                if !c.is_ascii_digit() {
                    return Err(TmValueError::InvalidCharacter {
                        string: trimmed.to_string(),
                        character: c,
                        position: i,
                    });
                }
            }

            // 長さに応じて、秒と分を0で埋める
            let padded_time = match trimmed.len() {
                2 => format!("{}0000", trimmed), // HH -> HH0000
                4 => format!("{}00", trimmed),   // HHMM -> HHMM00
                6 => trimmed.to_string(),        // HHMMSS
                _ => unreachable!(),
            };

            let time = NaiveTime::parse_from_str(&padded_time, "%H%M%S").map_err(|e| {
                TmValueError::ParseError {
                    string: trimmed.to_string(),
                    error: e,
                }
            })?;

            Ok(Self(time))
        }
    }
}

impl Display for TmValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // マイクロ秒がある場合は表示、ない場合はHHMMSSのみ
        if self.0.nanosecond() == 0 {
            write!(f, "{}", self.0.format("%H%M%S"))
        } else {
            // マイクロ秒を取得(ナノ秒を1000で割る)
            let micros = self.0.nanosecond() / 1000;
            // 末尾のゼロを削除
            let frac_str = format!("{:06}", micros).trim_end_matches('0').to_string();
            write!(f, "{}.{}", self.0.format("%H%M%S"), frac_str)
        }
    }
}

#[derive(Error, Debug)]
pub enum TmValueError {
    #[error("空値は許容されません")]
    Empty,

    #[error("文字列の長さが14バイトを超えています (文字列=\"{string}\", バイト数={byte_length})")]
    InvalidLength { string: String, byte_length: usize },

    #[error(
        "文字列に数字以外の文字が含まれています (文字列=\"{string}\", 文字='{character}', 位置={position})"
    )]
    InvalidCharacter {
        string: String,
        character: char,
        position: usize,
    },

    #[error("文字列の形式が不正です (文字列=\"{string}\")")]
    InvalidFormat { string: String },

    #[error("文字列から時刻へのパースに失敗しました (文字列=\"{string}\"): {error}")]
    ParseError {
        string: String,
        error: chrono::ParseError,
    },

    #[error("バイト列をUTF-8として解釈できません: {0}")]
    InvalidUtf8(#[from] Utf8Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        // 準正常系: 不正なUTF-8バイト列(InvalidUtf8)
        {
            // Arrange
            let bytes = b"\xff\xfe";

            // Act
            let result = TmValue::from_bytes(bytes);

            // Assert
            match result.unwrap_err() {
                TmValueError::InvalidUtf8(_) => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_from_string() {
        // 正常系: HHMMSS形式
        {
            // Arrange
            let input = "103045";
            let expected = TmValue(NaiveTime::from_hms_opt(10, 30, 45).unwrap());

            // Act
            let actual = TmValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: HHMMSS.FFFFFF形式(6桁)
        {
            // Arrange
            let input = "103045.123456";
            let expected = TmValue(NaiveTime::from_hms_micro_opt(10, 30, 45, 123456).unwrap());

            // Act
            let actual = TmValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: HHMMSS.F形式(1桁)
        {
            // Arrange
            let input = "103045.1";
            let expected = TmValue(NaiveTime::from_hms_micro_opt(10, 30, 45, 100000).unwrap());

            // Act
            let actual = TmValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: HH形式
        {
            // Arrange
            let input = "10";
            let expected = TmValue(NaiveTime::from_hms_opt(10, 0, 0).unwrap());

            // Act
            let actual = TmValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: HHMM形式
        {
            // Arrange
            let input = "1030";
            let expected = TmValue(NaiveTime::from_hms_opt(10, 30, 0).unwrap());

            // Act
            let actual = TmValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: HH.FFFFFF形式
        {
            // Arrange
            let input = "10.123456";
            let expected = TmValue(NaiveTime::from_hms_micro_opt(10, 0, 0, 123456).unwrap());

            // Act
            let actual = TmValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: HHMM.FFFFFF形式
        {
            // Arrange
            let input = "1030.123456";
            let expected = TmValue(NaiveTime::from_hms_micro_opt(10, 30, 0, 123456).unwrap());

            // Act
            let actual = TmValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 深夜0時
        {
            // Arrange
            let input = "000000";
            let expected = TmValue(NaiveTime::from_hms_opt(0, 0, 0).unwrap());

            // Act
            let actual = TmValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 深夜直前
        {
            // Arrange
            let input = "235959";
            let expected = TmValue(NaiveTime::from_hms_opt(23, 59, 59).unwrap());

            // Act
            let actual = TmValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 空文字列(Empty)
        {
            // Arrange
            let input = "";

            // Act
            let result = TmValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 文字列長さが15バイト(InvalidLength)
        {
            // Arrange
            let input = "103045.12345678";
            assert!(input.len() > 14);

            // Act
            let result = TmValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmValueError::InvalidLength {
                    string,
                    byte_length,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(byte_length, input.len());
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 数字以外の文字(InvalidCharacter)
        {
            // Arrange
            let input = "10A045";

            // Act
            let result = TmValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmValueError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, 'A');
                    assert_eq!(position, 2);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 小数部に数字以外の文字(InvalidCharacter)
        {
            // Arrange
            let input = "103045.12A456";

            // Act
            let result = TmValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmValueError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, 'A');
                    assert_eq!(position, 9); // "103045." + "12" = 9
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 1文字(InvalidFormat)
        {
            // Arrange
            let input = "1";

            // Act
            let result = TmValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmValueError::InvalidFormat { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 3文字(InvalidFormat)
        {
            // Arrange
            let input = "103";

            // Act
            let result = TmValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmValueError::InvalidFormat { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 5文字(InvalidFormat)
        {
            // Arrange
            let input = "10304";

            // Act
            let result = TmValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmValueError::InvalidFormat { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 小数点が複数(InvalidFormat)
        {
            // Arrange
            let input = "10.30.45";

            // Act
            let result = TmValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmValueError::InvalidFormat { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 小数部が空(InvalidFormat)
        {
            // Arrange
            let input = "103045.";

            // Act
            let result = TmValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmValueError::InvalidFormat { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 小数部が7桁(InvalidFormat)
        {
            // Arrange
            let input = "103045.1234567";

            // Act
            let result = TmValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmValueError::InvalidFormat { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 24時(ParseError)
        {
            // Arrange
            let input = "240000";

            // Act
            let result = TmValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmValueError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 60分(ParseError)
        {
            // Arrange
            let input = "106000";

            // Act
            let result = TmValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmValueError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 61秒(ParseError)
        {
            // Arrange
            let input = "103061";

            // Act
            let result = TmValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmValueError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // 正常系: 秒のみ
        {
            // Arrange
            let tm = TmValue(NaiveTime::from_hms_opt(10, 30, 45).unwrap());
            let expected = "103045";

            // Act
            let actual = tm.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 小数秒付き(末尾のゼロは削除)
        {
            // Arrange
            let tm = TmValue(NaiveTime::from_hms_micro_opt(10, 30, 45, 100000).unwrap());
            let expected = "103045.1";

            // Act
            let actual = tm.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 小数秒付き(6桁)
        {
            // Arrange
            let tm = TmValue(NaiveTime::from_hms_micro_opt(10, 30, 45, 123456).unwrap());
            let expected = "103045.123456";

            // Act
            let actual = tm.to_string();

            // Assert
            assert_eq!(expected, actual);
        }
    }
}
