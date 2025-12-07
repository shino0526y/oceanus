use std::{
    fmt::{Display, Formatter},
    str::Utf8Error,
};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct UiValue(String);

impl UiValue {
    const MAX_BYTE_LENGTH: usize = 64;

    pub fn uid(&self) -> &str {
        &self.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, UiValueError> {
        let str = str::from_utf8(bytes).map_err(UiValueError::InvalidUtf8)?;
        Self::from_string(str)
    }

    pub fn from_string(str: &str) -> Result<Self, UiValueError> {
        if str.len() > Self::MAX_BYTE_LENGTH {
            return Err(UiValueError::InvalidLength {
                string: str.to_string(),
                byte_length: str.len(),
            });
        }

        if str.is_empty() {
            return Err(UiValueError::Empty);
        }

        // パディングのNULL文字を削除
        let mut str = str;
        if str.len().is_multiple_of(2) && str.ends_with('\0') {
            str = &str[..str.len() - 1];
        }

        // 各文字が許可された文字(0-9、.)であることを確認
        for (i, c) in str.chars().enumerate() {
            if !matches!(c, '0'..='9' | '.') {
                return Err(UiValueError::InvalidCharacter {
                    string: str.to_string(),
                    character: c,
                    position: i,
                });
            }
        }

        // 先頭と末尾が.でないことを確認
        if str.starts_with('.') {
            return Err(UiValueError::StartsWithDot {
                string: str.to_string(),
            });
        }
        if str.ends_with('.') {
            return Err(UiValueError::EndsWithDot {
                string: str.to_string(),
            });
        }

        // 連続する.がないことを確認
        if str.contains("..") {
            return Err(UiValueError::ConsecutiveDots {
                string: str.to_string(),
            });
        }

        // 各コンポーネントの検証
        for component in str.split('.') {
            // 各コンポーネントの最初の桁が0でないことを確認（ただし単一の0は許可）
            // https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/chapter_9.html#sect_9.1
            if component.len() > 1 && component.starts_with('0') {
                return Err(UiValueError::ComponentStartsWithZero {
                    string: str.to_string(),
                    component: component.to_string(),
                });
            }
        }

        Ok(Self(str.to_string()))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.to_bytes_without_padding();
        if !bytes.len().is_multiple_of(2) {
            bytes.push(b'\0');
        }
        bytes
    }

    fn to_bytes_without_padding(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

impl Display for UiValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum UiValueError {
    #[error("空値は許容されません")]
    Empty,

    #[error("文字列の長さが64バイトを超えています (文字列=\"{string}\", バイト数={byte_length})")]
    InvalidLength { string: String, byte_length: usize },

    #[error(
        "文字列に不正な文字が含まれています (文字列=\"{string}\", 文字='{character}', 位置={position})"
    )]
    InvalidCharacter {
        string: String,
        character: char,
        position: usize,
    },

    #[error("文字列が'.'で始まっています (文字列=\"{string}\")")]
    StartsWithDot { string: String },

    #[error("文字列が'.'で終わっています (文字列=\"{string}\")")]
    EndsWithDot { string: String },

    #[error("文字列に連続した'.'が含まれています (文字列=\"{string}\")")]
    ConsecutiveDots { string: String },

    #[error(
        "コンポーネントが0で始まっています (文字列=\"{string}\", コンポーネント=\"{component}\")"
    )]
    ComponentStartsWithZero { string: String, component: String },

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
            let result = UiValue::from_bytes(bytes);

            // Assert
            match result.unwrap_err() {
                UiValueError::InvalidUtf8(_) => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_from_string() {
        // 正常系: 通常
        {
            // Arrange
            let input = "1.2.840.10008.1.1";
            let expected = UiValue("1.2.840.10008.1.1".to_string());

            // Act
            let actual = UiValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最大長
        {
            // Arrange
            let input = "1234567890.1234567890.1234567890.1234567890.1234567890.123456789";
            assert_eq!(input.len(), 64);
            let expected = UiValue(input.to_string());

            // Act
            let actual = UiValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: コンポーネントに単一の0を含む
        //         https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/chapter_9.html#sect_9.1
        {
            // Arrange
            let input = "1.2.840.10008.1.1.0";
            let expected = UiValue("1.2.840.10008.1.1.0".to_string());

            // Act
            let actual = UiValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 空文字列(Empty)
        {
            // Arrange
            let input = "";

            // Act
            let result = UiValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                UiValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 65バイトのUID(InvalidLength)
        {
            // Arrange
            let input = "1234567890.1234567890.1234567890.1234567890.1234567890.1234567890";
            assert_eq!(input.len(), 65);

            // Act
            let result = UiValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                UiValueError::InvalidLength {
                    string,
                    byte_length,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(byte_length, 65);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: アルファベットを含む(InvalidCharacter)
        {
            // Arrange
            let input = "1.2.840.10008.1.a";

            // Act
            let result = UiValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                UiValueError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, 'a');
                    assert_eq!(position, 16);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: '.'で始まる(StartsWithDot)
        {
            // Arrange
            let input = ".1.2.840.10008.1.1";

            // Act
            let result = UiValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                UiValueError::StartsWithDot { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: '.'で終わる(EndsWithDot)
        {
            // Arrange
            let input = "1.2.840.10008.1.1.";

            // Act
            let result = UiValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                UiValueError::EndsWithDot { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 連続した'.'を含む(ConsecutiveDots)
        {
            // Arrange
            let input = "1.2..840.10008.1.1";

            // Act
            let result = UiValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                UiValueError::ConsecutiveDots { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: コンポーネントが0で始まる(ComponentStartsWithZero)
        //           https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/chapter_9.html#sect_9.1
        {
            // Arrange
            let input = "1.2.840.10008.1.1.00";

            // Act
            let result = UiValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                UiValueError::ComponentStartsWithZero { string, component } => {
                    assert_eq!(string, input);
                    assert_eq!(component, "00");
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let ui = UiValue("1.2.840.10008.1.1".to_string());
        let expected = "1.2.840.10008.1.1";

        // Act
        let actual = ui.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
