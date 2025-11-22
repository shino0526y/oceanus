use std::{
    fmt::{Display, Formatter},
    str::Utf8Error,
};
use thiserror::Error;

/// UID（Unique Identifier）
#[derive(Debug, PartialEq, Clone)]
pub struct Ui {
    uid: String,
}

impl Ui {
    const MAX_BYTE_LENGTH: usize = 64;

    pub fn uid(&self) -> &str {
        &self.uid
    }

    /// 文字列からUID（UI）の値をパースします。
    ///
    /// この関数は、DICOM規格に準拠したUI型の文字列をパースし、
    /// バックスラッシュ(`\`)で区切られた複数の値を処理します。
    /// UIDは最大64バイトまでで、数字（0-9）とピリオド（`.`）のみが許可されます。
    /// UIDは先頭または末尾がピリオドで始まったり終わったりすることはできず、
    /// 連続したピリオドも許可されません。
    ///
    /// # 引数
    ///
    /// * `str` - パースする文字列。複数の値はバックスラッシュ(`\`)で区切られます。
    ///
    /// # 戻り値
    ///
    /// * `Ok(Vec<Option<Self>>)` - パースに成功した場合、UI値のベクタを返します。
    ///   空文字列は`None`として表現されます。
    /// * `Err(UiError)` - パースに失敗した場合、エラーを返します。
    ///   不正な文字が含まれている場合、64バイトを超える値がある場合、
    ///   またはピリオドの使用が不適切な場合にエラーとなります。
    ///
    /// # パディング処理
    ///
    /// DICOM規格に従い、文字列が偶数バイト長で末尾がNULL文字（`\0`）の場合、
    /// そのNULL文字は自動的に除去されます。
    ///
    /// # UID形式の制約
    ///
    /// - 数字（0-9）とピリオド（`.`）のみ使用可能
    /// - 先頭がピリオドで始まらない
    /// - 末尾がピリオドで終わらない
    /// - 連続したピリオド（`..`）を含まない
    /// - 最大64バイト
    ///
    /// # 例
    ///
    /// ```
    /// use dicom_lib::core::value::values::Ui;
    ///
    /// // 標準的なDICOM UID
    /// let result = Ui::from_string("1.2.840.10008.1.1").unwrap();
    /// assert_eq!(result.len(), 1);
    /// assert_eq!(result[0].as_ref().unwrap().uid(), "1.2.840.10008.1.1");
    ///
    /// // 複数のUIDをパース（バックスラッシュ区切り）
    /// let result = Ui::from_string(r"1.2.840.10008.1.1\1.2.840.10008.1.2").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert_eq!(result[0].as_ref().unwrap().uid(), "1.2.840.10008.1.1");
    /// assert_eq!(result[1].as_ref().unwrap().uid(), "1.2.840.10008.1.2");
    ///
    /// // 空値を含むケース
    /// let result = Ui::from_string(r"\1.2.840.10008.1.1").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert!(result[0].is_none());
    /// assert_eq!(result[1].as_ref().unwrap().uid(), "1.2.840.10008.1.1");
    ///
    /// // 最大長（64バイト）
    /// let input = "1234567890.1234567890.1234567890.1234567890.1234567890.123456789";
    /// let result = Ui::from_string(input).unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().uid(), input);
    /// ```
    ///
    /// # エラー
    ///
    /// ```
    /// use dicom_lib::core::value::values::{Ui, ui::UiError};
    ///
    /// // 65バイトを超えるUID
    /// let input = "1234567890.1234567890.1234567890.1234567890.1234567890.1234567890";
    /// let result = Ui::from_string(input);
    /// assert!(matches!(result, Err(UiError::InvalidLength { .. })));
    ///
    /// // 不正な文字を含む（アルファベット）
    /// let result = Ui::from_string("1.2.840.10008.1.a");
    /// assert!(matches!(result, Err(UiError::InvalidCharacter { .. })));
    ///
    /// // ピリオドで始まる
    /// let result = Ui::from_string(".1.2.840.10008.1.1");
    /// assert!(matches!(result, Err(UiError::StartsWithDot { .. })));
    ///
    /// // ピリオドで終わる
    /// let result = Ui::from_string("1.2.840.10008.1.1.");
    /// assert!(matches!(result, Err(UiError::EndsWithDot { .. })));
    ///
    /// // 連続したピリオド
    /// let result = Ui::from_string("1.2..840.10008.1.1");
    /// assert!(matches!(result, Err(UiError::ConsecutiveDots { .. })));
    /// ```
    pub fn from_string(str: &str) -> Result<Vec<Option<Self>>, UiError> {
        let str = if str.len().is_multiple_of(2) && str.ends_with('\0') {
            &str[..str.len() - 1]
        } else {
            str
        };
        let strings = str.split('\\').collect::<Vec<_>>();

        let mut values = Vec::with_capacity(strings.len());
        for str in strings {
            values.push(Self::from_string_single(str)?);
        }

        Ok(values)
    }

    pub fn from_buf(buf: &[u8]) -> Result<Vec<Option<Self>>, UiError> {
        let str = str::from_utf8(buf).map_err(UiError::InvalidUtf8)?;
        Self::from_string(str)
    }

    fn from_string_single(str: &str) -> Result<Option<Self>, UiError> {
        if str.len() > Self::MAX_BYTE_LENGTH {
            return Err(UiError::InvalidLength {
                string: str.to_string(),
                byte_length: str.len(),
            });
        }

        if str.is_empty() {
            return Ok(None);
        }

        // 各文字が許可された文字(0-9、.)であることを確認
        for (i, c) in str.chars().enumerate() {
            if !matches!(c, '0'..='9' | '.') {
                return Err(UiError::InvalidCharacter {
                    string: str.to_string(),
                    character: c,
                    position: i,
                });
            }
        }

        // 先頭と末尾が.でないことを確認
        if str.starts_with('.') {
            return Err(UiError::StartsWithDot {
                string: str.to_string(),
            });
        }
        if str.ends_with('.') {
            return Err(UiError::EndsWithDot {
                string: str.to_string(),
            });
        }

        // 連続する.がないことを確認
        if str.contains("..") {
            return Err(UiError::ConsecutiveDots {
                string: str.to_string(),
            });
        }

        Ok(Some(Self {
            uid: str.to_string(),
        }))
    }
}

impl Display for Ui {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uid)
    }
}

#[derive(Error, Debug)]
pub enum UiError {
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

    #[error("バイト列をUTF-8として解釈できません: {0}")]
    InvalidUtf8(#[from] Utf8Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        // 正常系: 通常
        {
            // Arrange
            let input = "1.2.840.10008.1.1";
            let expected = vec![Some(Ui {
                uid: "1.2.840.10008.1.1".to_string(),
            })];

            // Act
            let actual = Ui::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値（バックスラッシュ区切り）
        {
            // Arrange
            let input = r"1.2.840.10008.1.1\1.2.840.10008.1.2";
            let expected = vec![
                Some(Ui {
                    uid: "1.2.840.10008.1.1".to_string(),
                }),
                Some(Ui {
                    uid: "1.2.840.10008.1.2".to_string(),
                }),
            ];

            // Act
            let actual = Ui::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let input = r"\1.2.840.10008.1.1";
            let expected = vec![
                None,
                Some(Ui {
                    uid: "1.2.840.10008.1.1".to_string(),
                }),
            ];

            // Act
            let actual = Ui::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空文字列
        {
            // Arrange
            let input = "";
            let expected = vec![None];

            // Act
            let actual = Ui::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最大長（64バイト）
        {
            // Arrange
            let input = "1234567890.1234567890.1234567890.1234567890.1234567890.123456789";
            assert_eq!(input.len(), 64);
            let expected = vec![Some(Ui {
                uid: input.to_string(),
            })];

            // Act
            let actual = Ui::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値の最後の値がNULLパディングされ、その値の長さが65になるケース
        //        詳しくは以下を参照。
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.4.html
        {
            // Arrange
            let input = "1234567890.1234567890.1234567890.1234567890.1234567890.123456789\\1234567890.1234567890.1234567890.1234567890.1234567890.123456789\0";
            let expected = vec![
                Some(Ui {
                    uid: "1234567890.1234567890.1234567890.1234567890.1234567890.123456789"
                        .to_string(),
                }),
                Some(Ui {
                    uid: "1234567890.1234567890.1234567890.1234567890.1234567890.123456789"
                        .to_string(),
                }),
            ];

            // Act
            let actual = Ui::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 65バイトのUID（長すぎる）
        {
            // Arrange
            let input = "1234567890.1234567890.1234567890.1234567890.1234567890.1234567890";
            assert_eq!(input.len(), 65);

            // Act
            let result = Ui::from_string(input);

            // Assert
            match result.unwrap_err() {
                UiError::InvalidLength {
                    string,
                    byte_length,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(byte_length, 65);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 不正な文字を含むUID
        {
            // Arrange
            let input = "1.2.840.10008.1.a";

            // Act
            let result = Ui::from_string(input);

            // Assert
            match result.unwrap_err() {
                UiError::InvalidCharacter {
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

        // 準正常系: '.'で始まるUID
        {
            // Arrange
            let input = ".1.2.840.10008.1.1";

            // Act
            let result = Ui::from_string(input);

            // Assert
            match result.unwrap_err() {
                UiError::StartsWithDot { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: '.'で終わるUID
        {
            // Arrange
            let input = "1.2.840.10008.1.1.";

            // Act
            let result = Ui::from_string(input);

            // Assert
            match result.unwrap_err() {
                UiError::EndsWithDot { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 連続した'.'を含むUID
        {
            // Arrange
            let input = "1.2..840.10008.1.1";

            // Act
            let result = Ui::from_string(input);

            // Assert
            match result.unwrap_err() {
                UiError::ConsecutiveDots { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_from_buf() {
        // 正常系: 通常
        {
            // Arrange
            let buf = b"1.2.840.10008.1.1";
            let expected = vec![Some(Ui {
                uid: "1.2.840.10008.1.1".to_string(),
            })];

            // Act
            let actual = Ui::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: NULL文字パディングを含む
        {
            // Arrange
            let buf = b"1.2.840.10008.1.1\0";
            let expected = vec![Some(Ui {
                uid: "1.2.840.10008.1.1".to_string(),
            })];

            // Act
            let actual = Ui::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値
        {
            // Arrange
            let buf = b"1.2.840.10008.1.1\\1.2.840.10008.1.2";
            let expected = vec![
                Some(Ui {
                    uid: "1.2.840.10008.1.1".to_string(),
                }),
                Some(Ui {
                    uid: "1.2.840.10008.1.2".to_string(),
                }),
            ];

            // Act
            let actual = Ui::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let buf = b"\\1.2.840.10008.1.1";
            let expected = vec![
                None,
                Some(Ui {
                    uid: "1.2.840.10008.1.1".to_string(),
                }),
            ];

            // Act
            let actual = Ui::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 不正なUTF-8バイト列
        {
            // Arrange
            let buf = b"\xff\xfe";

            // Act
            let result = Ui::from_buf(buf);

            // Assert
            match result.unwrap_err() {
                UiError::InvalidUtf8(_) => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let ui = Ui {
            uid: "1.2.840.10008.1.1".to_string(),
        };
        let expected = "1.2.840.10008.1.1";

        // Act
        let actual = ui.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
