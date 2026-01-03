use chrono::NaiveDate;
use std::{
    fmt::{Display, Formatter},
    str::Utf8Error,
};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct DaValue(NaiveDate);

impl DaValue {
    const BYTE_LENGTH: usize = 8;

    pub fn date(&self) -> &NaiveDate {
        &self.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DaValueError> {
        let str = str::from_utf8(bytes).map_err(DaValueError::InvalidUtf8)?;
        Self::from_string(str)
    }

    pub fn from_string(str: &str) -> Result<Self, DaValueError> {
        if str.is_empty() {
            return Err(DaValueError::Empty);
        }

        if str.len() != Self::BYTE_LENGTH {
            return Err(DaValueError::InvalidLength {
                string: str.to_string(),
                length: str.len(),
            });
        }

        // 各文字が数字であることを確認
        for (i, c) in str.chars().enumerate() {
            if !c.is_ascii_digit() {
                return Err(DaValueError::InvalidCharacter {
                    string: str.to_string(),
                    character: c,
                    position: i,
                });
            }
        }

        let date =
            NaiveDate::parse_from_str(str, "%Y%m%d").map_err(|e| DaValueError::ParseError {
                string: str.to_string(),
                error: e,
            })?;

        Ok(Self(date))
    }
}

impl Display for DaValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.format("%Y%m%d"))
    }
}

#[derive(Error, Debug)]
pub enum DaValueError {
    #[error("空値は許容されません")]
    Empty,

    #[error("文字列の長さが8バイトではありません (文字列=\"{string}\", 長さ={length})")]
    InvalidLength { string: String, length: usize },

    #[error(
        "文字列に数字以外の文字が含まれています (文字列=\"{string}\", 文字='{character}', 位置={position})"
    )]
    InvalidCharacter {
        string: String,
        character: char,
        position: usize,
    },

    #[error("文字列から日付へのパースに失敗しました (文字列=\"{string}\"): {error}")]
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
            let bytes = b"\xff\xfe\xff\xfe\xff\xfe\xff\xfe";

            // Act
            let result = DaValue::from_bytes(bytes);

            // Assert
            match result.unwrap_err() {
                DaValueError::InvalidUtf8(_) => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_from_string() {
        // 正常系: 通常の日付
        {
            // Arrange
            let input = "20251130";
            let expected = DaValue(NaiveDate::from_ymd_opt(2025, 11, 30).unwrap());

            // Act
            let actual = DaValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 閏年の日付
        {
            // Arrange
            let input = "20240229";
            let expected = DaValue(NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());

            // Act
            let actual = DaValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 年初の日付
        {
            // Arrange
            let input = "20250101";
            let expected = DaValue(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());

            // Act
            let actual = DaValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 年末の日付
        {
            // Arrange
            let input = "20251231";
            let expected = DaValue(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap());

            // Act
            let actual = DaValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 空文字列(Empty)
        {
            // Arrange
            let input = "";

            // Act
            let result = DaValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                DaValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 6バイトの日付(InvalidLength)
        {
            // Arrange
            let input = "202511";

            // Act
            let result = DaValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                DaValueError::InvalidLength { string, length } => {
                    assert_eq!(string, input);
                    assert_eq!(length, 6);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 14バイトの日付(InvalidLength)
        {
            // Arrange
            let input = "20251130000000";

            // Act
            let result = DaValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                DaValueError::InvalidLength { string, length } => {
                    assert_eq!(string, input);
                    assert_eq!(length, 14);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 数字以外の文字を含む日付(InvalidCharacter)
        {
            // Arrange
            let input = "2025A130";

            // Act
            let result = DaValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                DaValueError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, 'A');
                    assert_eq!(position, 4);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 存在しない月(ParseError)
        {
            // Arrange
            let input = "20251315";

            // Act
            let result = DaValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                DaValueError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 存在しない日付(ParseError)
        {
            // Arrange
            let input = "20250230";

            // Act
            let result = DaValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                DaValueError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 閏年でない年の2月29日(ParseError)
        {
            // Arrange
            let input = "20250229";

            // Act
            let result = DaValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                DaValueError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let da = DaValue(NaiveDate::from_ymd_opt(2025, 11, 30).unwrap());
        let expected = "20251130";

        // Act
        let actual = da.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
