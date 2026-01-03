use crate::core::value::{self, SpecificCharacterSet};
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct LoValue(String);

impl LoValue {
    const MAX_CHAR_COUNT: usize = 64;

    pub fn string(&self) -> &str {
        &self.0
    }

    pub fn from_bytes_lossy(
        bytes: &[u8],
        char_set: SpecificCharacterSet,
    ) -> Result<Self, LoValueError> {
        let str = value::generate_string_lossy(bytes, char_set);
        Self::from_string(&str)
    }

    pub fn from_string(str: &str) -> Result<Self, LoValueError> {
        if str.chars().count() > Self::MAX_CHAR_COUNT {
            return Err(LoValueError::InvalidLength {
                string: str.to_string(),
                char_count: str.chars().count(),
            });
        }

        let trimmed = str.trim_matches(' ');
        if trimmed.is_empty() {
            return Err(LoValueError::Empty);
        }

        Ok(Self(trimmed.to_string()))
    }
}

impl Display for LoValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum LoValueError {
    #[error("空値は許容されません")]
    Empty,

    #[error("文字列の長さが64文字を超えています (文字列=\"{string}\", 文字数={char_count})")]
    InvalidLength { string: String, char_count: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        // 正常系: 通常
        {
            // Arrange
            let input = "Long String";
            let expected = LoValue("Long String".to_string());

            // Act
            let actual = LoValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 前後に空白を含む
        {
            // Arrange
            let input = " Long String  ";
            let expected = LoValue("Long String".to_string());

            // Act
            let actual = LoValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最大長
        {
            // Arrange
            let input = "1234567890123456789012345678901234567890123456789012345678901234";
            assert_eq!(input.chars().count(), 64);
            let expected = LoValue(input.to_string());

            // Act
            let actual = LoValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 日本語を含む文字列
        {
            // Arrange
            let input = "日本語の文字列";
            let expected = LoValue("日本語の文字列".to_string());

            // Act
            let actual = LoValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 空文字列(空値)
        {
            // Arrange
            let input = "";

            // Act
            let result = LoValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                LoValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: スペースのみ(空値)
        {
            // Arrange
            let input = "  ";

            // Act
            let result = LoValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                LoValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 65文字の文字列(InvalidLength)
        {
            // Arrange
            let input = "12345678901234567890123456789012345678901234567890123456789012345";
            assert_eq!(input.chars().count(), 65);

            // Act
            let result = LoValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                LoValueError::InvalidLength { string, char_count } => {
                    assert_eq!(string, input);
                    assert_eq!(char_count, 65);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let lo = LoValue("Long String".to_string());
        let expected = "Long String";

        // Act
        let actual = lo.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
