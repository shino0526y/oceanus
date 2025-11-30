use std::{
    fmt::{Display, Formatter},
    num::ParseIntError,
    str::Utf8Error,
};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct IsValue(i32);

impl IsValue {
    const MAX_BYTE_LENGTH: usize = 12;

    pub fn value(&self) -> i32 {
        self.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, IsValueError> {
        let str = str::from_utf8(bytes).map_err(IsValueError::InvalidUtf8)?;
        Self::from_string(str)
    }

    pub fn from_string(str: &str) -> Result<Self, IsValueError> {
        if str.len() > Self::MAX_BYTE_LENGTH {
            return Err(IsValueError::InvalidLength {
                string: str.to_string(),
                byte_length: str.len(),
            });
        }

        let trimmed = str.trim_matches(' ');
        if trimmed.is_empty() {
            return Err(IsValueError::Empty);
        }

        // 各文字が許可された文字(数字、+、-)であることを確認
        for (i, c) in trimmed.chars().enumerate() {
            if !matches!(c, '0'..='9' | '+' | '-') {
                return Err(IsValueError::InvalidCharacter {
                    string: trimmed.to_string(),
                    character: c,
                    position: i,
                });
            }
        }

        let value = trimmed
            .parse::<i32>()
            .map_err(|e| IsValueError::ParseError {
                string: trimmed.to_string(),
                error: e,
            })?;

        Ok(Self(value))
    }
}

impl Display for IsValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum IsValueError {
    #[error("空値は許容されません")]
    Empty,

    #[error("文字列の長さが12バイトを超えています (文字列=\"{string}\", バイト数={byte_length})")]
    InvalidLength { string: String, byte_length: usize },

    #[error(
        "文字列に不正な文字が含まれています (文字列=\"{string}\", 文字='{character}', 位置={position})"
    )]
    InvalidCharacter {
        string: String,
        character: char,
        position: usize,
    },

    #[error("文字列から数値へのパースに失敗しました (文字列=\"{string}\"): {error}")]
    ParseError {
        string: String,
        error: ParseIntError,
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
            let result = IsValue::from_bytes(bytes);

            // Assert
            match result.unwrap_err() {
                IsValueError::InvalidUtf8(_) => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_from_string() {
        // 正常系: 正の整数
        {
            // Arrange
            let input = "12345";
            let expected = IsValue(12345);

            // Act
            let actual = IsValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 負の整数
        {
            // Arrange
            let input = "-12345";
            let expected = IsValue(-12345);

            // Act
            let actual = IsValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: プラス記号付きの整数
        {
            // Arrange
            let input = "+12345";
            let expected = IsValue(12345);

            // Act
            let actual = IsValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: ゼロ
        {
            // Arrange
            let input = "0";
            let expected = IsValue(0);

            // Act
            let actual = IsValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最大値 (2^31 - 1)
        {
            // Arrange
            let input = "2147483647";
            let expected = IsValue(2147483647);

            // Act
            let actual = IsValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最小値 (-2^31)
        {
            // Arrange
            let input = "-2147483648";
            let expected = IsValue(-2147483648);

            // Act
            let actual = IsValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 前後に空白を含む文字列(空白は削除される)
        {
            // Arrange
            let input = " 123  ";
            let expected = IsValue(123);

            // Act
            let actual = IsValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 空文字列(Empty)
        {
            // Arrange
            let input = "";

            // Act
            let result = IsValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: スペースのみ(Empty)
        {
            // Arrange
            let input = "  ";

            // Act
            let result = IsValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 13バイトの文字列(InvalidLength)
        {
            // Arrange
            let input = "1234567890123";
            assert_eq!(input.len(), 13);

            // Act
            let result = IsValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsValueError::InvalidLength {
                    string,
                    byte_length,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(byte_length, 13);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 不正な文字を含む文字列(InvalidCharacter)
        {
            // Arrange
            let input = "123.45";

            // Act
            let result = IsValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsValueError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, '.');
                    assert_eq!(position, 3);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: アルファベットを含む文字列(InvalidCharacter)
        {
            // Arrange
            let input = "123A";

            // Act
            let result = IsValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsValueError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, 'A');
                    assert_eq!(position, 3);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 埋め込みスペースを含む(InvalidCharacter)
        {
            // Arrange
            let input = "12 34";

            // Act
            let result = IsValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsValueError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, ' ');
                    assert_eq!(position, 2);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 符号が途中にある文字列(ParseError)
        {
            // Arrange
            let input = "12-34";

            // Act
            let result = IsValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsValueError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 範囲外の値(2^31以上)(ParseError)
        {
            // Arrange
            let input = "2147483648";

            // Act
            let result = IsValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsValueError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 範囲外の値(-2^31未満)(ParseError)
        {
            // Arrange
            let input = "-2147483649";

            // Act
            let result = IsValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsValueError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let is = IsValue(12345);
        let expected = "12345";

        // Act
        let actual = is.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
