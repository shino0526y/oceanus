use std::{
    fmt::{Display, Formatter},
    str::Utf8Error,
};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct CsValue(String);

impl CsValue {
    const MAX_BYTE_LENGTH: usize = 16;

    pub fn code(&self) -> &str {
        &self.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CsValueError> {
        let str = str::from_utf8(bytes).map_err(CsValueError::InvalidUtf8)?;
        Self::from_string(str)
    }

    pub fn from_string(str: &str) -> Result<Self, CsValueError> {
        if str.len() > Self::MAX_BYTE_LENGTH {
            return Err(CsValueError::InvalidLength {
                string: str.to_string(),
                byte_length: str.len(),
            });
        }

        let trimmed = str.trim_matches(' ');
        if trimmed.is_empty() {
            return Err(CsValueError::Empty);
        }

        // 各文字が許可された文字(大文字、数字、スペース、アンダースコア)であることを確認
        for (i, c) in trimmed.chars().enumerate() {
            if !matches!(c, 'A'..='Z' | '0'..='9' | ' ' | '_') {
                return Err(CsValueError::InvalidCharacter {
                    string: trimmed.to_string(),
                    character: c,
                    position: i,
                });
            }
        }

        Ok(Self(trimmed.to_string()))
    }
}

impl Display for CsValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum CsValueError {
    #[error("空値は許容されません")]
    Empty,

    #[error("文字列の長さが16バイトを超えています (文字列=\"{string}\", バイト数={byte_length})")]
    InvalidLength { string: String, byte_length: usize },

    #[error(
        "文字列に不正な文字が含まれています (文字列=\"{string}\", 文字='{character}', 位置={position})"
    )]
    InvalidCharacter {
        string: String,
        character: char,
        position: usize,
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
            let result = CsValue::from_bytes(bytes);

            // Assert
            match result.unwrap_err() {
                CsValueError::InvalidUtf8(_) => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_from_string() {
        // 正常系: 通常
        {
            // Arrange
            let input = "ORIGINAL";
            let expected = CsValue("ORIGINAL".to_string());

            // Act
            let actual = CsValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 前後に空白を含む
        {
            // Arrange
            let input = " DERIVED  ";
            let expected = CsValue("DERIVED".to_string());

            // Act
            let actual = CsValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最大長
        {
            // Arrange
            let input = "1234567890ABCDEF";
            assert_eq!(input.len(), 16);
            let expected = CsValue(input.to_string());

            // Act
            let actual = CsValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 空文字列(Empty)
        {
            // Arrange
            let input = "";

            // Act
            let result = CsValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                CsValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: スペースのみ(Empty)
        {
            // Arrange
            let input = "  ";

            // Act
            let result = CsValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                CsValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 17文字の文字列(InvalidLength)
        {
            // Arrange
            let input = "1234567890ABCDEFG";
            assert_eq!(input.len(), 17);

            // Act
            let result = CsValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                CsValueError::InvalidLength {
                    string,
                    byte_length,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(byte_length, 17);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: バックスラッシュを含む(InvalidCharacter)
        {
            // Arrange
            let input = r"CODE\STRING";

            // Act
            let result = CsValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                CsValueError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, '\\');
                    assert_eq!(position, 4);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 小文字を含む(InvalidCharacter)
        {
            // Arrange
            let input = "primary";

            // Act
            let result = CsValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                CsValueError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, 'p');
                    assert_eq!(position, 0);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 記号を含む(InvalidCharacter)
        {
            // Arrange
            let input = "CODE-STRING";

            // Act
            let result = CsValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                CsValueError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, '-');
                    assert_eq!(position, 4);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let cs = CsValue("ORIGINAL".to_string());
        let expected = "ORIGINAL";

        // Act
        let actual = cs.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
