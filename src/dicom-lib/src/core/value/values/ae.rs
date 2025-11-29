use std::{
    fmt::{Display, Formatter},
    str::Utf8Error,
};
use thiserror::Error;

/// アプリケーションエンティティ（Application Entity）
#[derive(Debug, PartialEq, Clone)]
pub struct Ae {
    value: String,
}

impl Ae {
    const MAX_BYTE_LENGTH: usize = 16;

    pub fn value(&self) -> &str {
        &self.value
    }

    /// 文字列からアプリケーションエンティティ（AE）の値をパースします。
    ///
    /// この関数は、DICOM規格に準拠したAE型の文字列をパースし、
    /// バックスラッシュ(`\`)で区切られた複数の値を処理します。
    /// 各値は最大16バイトまでで、バックスラッシュと制御文字以外の文字が許可されます。
    /// 先頭と末尾の空白は意味を持たず、自動的にトリミングされます。
    /// 空白のみの値は使用できません。
    ///
    /// # 引数
    ///
    /// * `str` - パースする文字列。複数の値はバックスラッシュ(`\`)で区切られます。
    ///
    /// # 戻り値
    ///
    /// * `Ok(Vec<Option<Self>>)` - パースに成功した場合、AE値のベクタを返します。
    ///   空文字列は`None`として表現されます。
    /// * `Err(AeError)` - パースに失敗した場合、エラーを返します。
    ///   不正な文字が含まれている場合や、16バイトを超える値がある場合にエラーとなります。
    ///
    /// # パディング処理
    ///
    /// DICOM規格に従い、文字列が偶数バイト長で末尾が空白の場合、
    /// その空白は自動的に除去されます。また、各値の前後の空白もトリミングされます。
    ///
    /// # 例
    ///
    /// ```
    /// use dicom_lib::core::value::values::Ae;
    ///
    /// // 単一のAE titleをパース
    /// let result = Ae::from_string("STORE_SCP").unwrap();
    /// assert_eq!(result.len(), 1);
    /// assert_eq!(result[0].as_ref().unwrap().value(), "STORE_SCP");
    ///
    /// // 複数の値をパース（バックスラッシュ区切り）
    /// let result = Ae::from_string(r"STORE_SCP\QUERY_SCP").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert_eq!(result[0].as_ref().unwrap().value(), "STORE_SCP");
    /// assert_eq!(result[1].as_ref().unwrap().value(), "QUERY_SCP");
    ///
    /// // 空値を含むケース
    /// let result = Ae::from_string(r"\STORE_SCP").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert!(result[0].is_none());
    /// assert_eq!(result[1].as_ref().unwrap().value(), "STORE_SCP");
    ///
    /// // 前後の空白は自動的にトリミングされる
    /// let result = Ae::from_string(" STORE_SCP  ").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().value(), "STORE_SCP");
    ///
    /// // 最大長（16バイト）
    /// let input = "1234567890ABCDEF";
    /// let result = Ae::from_string(input).unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().value(), input);
    /// ```
    ///
    /// # エラー
    ///
    /// ```
    /// use dicom_lib::core::value::values::{Ae, ae::AeError};
    ///
    /// // 17バイトを超える文字列
    /// let result = Ae::from_string("1234567890ABCDEFG");
    /// assert!(matches!(result, Err(AeError::InvalidLength { .. })));
    ///
    /// // 制御文字を含む
    /// let result = Ae::from_string("STORE\nSCP");
    /// assert!(matches!(result, Err(AeError::InvalidCharacter { .. })));
    /// ```
    pub fn from_string(str: &str) -> Result<Vec<Option<Self>>, AeError> {
        let str = if str.len().is_multiple_of(2) && str.ends_with(' ') {
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

    pub fn from_buf(buf: &[u8]) -> Result<Vec<Option<Self>>, AeError> {
        let str = str::from_utf8(buf).map_err(AeError::InvalidUtf8)?;
        Self::from_string(str)
    }

    fn from_string_single(str: &str) -> Result<Option<Self>, AeError> {
        if str.len() > Self::MAX_BYTE_LENGTH {
            return Err(AeError::InvalidLength {
                string: str.to_string(),
                byte_length: str.len(),
            });
        }

        let trimmed = str.trim_matches(' ');
        if trimmed.is_empty() {
            return Ok(None);
        }

        // 各文字が許可された文字（制御文字以外）であることを確認
        for (i, c) in trimmed.chars().enumerate() {
            if c.is_control() {
                return Err(AeError::InvalidCharacter {
                    string: trimmed.to_string(),
                    character: c,
                    position: i,
                });
            }
        }

        Ok(Some(Self {
            value: trimmed.to_string(),
        }))
    }
}

impl Display for Ae {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Error, Debug)]
pub enum AeError {
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
    fn test_from_string() {
        // 正常系: 通常
        {
            // Arrange
            let input = "STORE_SCP";
            let expected = vec![Some(Ae {
                value: "STORE_SCP".to_string(),
            })];

            // Act
            let actual = Ae::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値（バックスラッシュ区切り）
        {
            // Arrange
            let input = r"STORE_SCP\QUERY_SCP";
            let expected = vec![
                Some(Ae {
                    value: "STORE_SCP".to_string(),
                }),
                Some(Ae {
                    value: "QUERY_SCP".to_string(),
                }),
            ];

            // Act
            let actual = Ae::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let input = r"\STORE_SCP";
            let expected = vec![
                None,
                Some(Ae {
                    value: "STORE_SCP".to_string(),
                }),
            ];

            // Act
            let actual = Ae::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 前後に空白を含む
        {
            // Arrange
            let input = " STORE_SCP  ";
            let expected = vec![Some(Ae {
                value: "STORE_SCP".to_string(),
            })];

            // Act
            let actual = Ae::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空文字列
        {
            // Arrange
            let input = "";
            let expected = vec![None];

            // Act
            let actual = Ae::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最大長（16バイト）
        {
            // Arrange
            let input = "1234567890ABCDEF";
            assert_eq!(input.len(), 16);
            let expected = vec![Some(Ae {
                value: input.to_string(),
            })];

            // Act
            let actual = Ae::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 記号を含む（ハイフン、アンダースコア等）
        {
            // Arrange
            let input = "PACS-SERVER_01";
            let expected = vec![Some(Ae {
                value: "PACS-SERVER_01".to_string(),
            })];

            // Act
            let actual = Ae::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 小文字を含む
        {
            // Arrange
            let input = "StoreScp";
            let expected = vec![Some(Ae {
                value: "StoreScp".to_string(),
            })];

            // Act
            let actual = Ae::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値の最後の値が空白パディングされ、その値の長さが17になるケース
        //        詳しくは以下を参照。
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.4.html
        {
            // Arrange
            let input = r"0123456789ABCDEF\0123456789ABCDEF ";
            let expected = vec![
                Some(Ae {
                    value: "0123456789ABCDEF".to_string(),
                }),
                Some(Ae {
                    value: "0123456789ABCDEF".to_string(),
                }),
            ];

            // Act
            let actual = Ae::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 17バイトの文字列（長すぎる）
        {
            // Arrange
            let input = "1234567890ABCDEFG";
            assert_eq!(input.len(), 17);

            // Act
            let result = Ae::from_string(input);

            // Assert
            match result.unwrap_err() {
                AeError::InvalidLength {
                    string,
                    byte_length,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(byte_length, 17);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 改行を含む（制御文字）
        {
            // Arrange
            let input = "STORE\nSCP";

            // Act
            let result = Ae::from_string(input);

            // Assert
            match result.unwrap_err() {
                AeError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, '\n');
                    assert_eq!(position, 5);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: タブを含む（制御文字）
        {
            // Arrange
            let input = "STORE\tSCP";

            // Act
            let result = Ae::from_string(input);

            // Assert
            match result.unwrap_err() {
                AeError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, '\t');
                    assert_eq!(position, 5);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: NULL文字を含む（制御文字）
        {
            // Arrange
            let input = "STORE\0SCP";

            // Act
            let result = Ae::from_string(input);

            // Assert
            match result.unwrap_err() {
                AeError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, '\0');
                    assert_eq!(position, 5);
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
            let buf = b"STORE_SCP";
            let expected = vec![Some(Ae {
                value: "STORE_SCP".to_string(),
            })];

            // Act
            let actual = Ae::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空白パディングを含む
        {
            // Arrange
            let buf = b" STORE_SCP  ";
            let expected = vec![Some(Ae {
                value: "STORE_SCP".to_string(),
            })];

            // Act
            let actual = Ae::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値
        {
            // Arrange
            let buf = b"STORE_SCP\\QUERY_SCP";
            let expected = vec![
                Some(Ae {
                    value: "STORE_SCP".to_string(),
                }),
                Some(Ae {
                    value: "QUERY_SCP".to_string(),
                }),
            ];

            // Act
            let actual = Ae::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let buf = b"\\STORE_SCP";
            let expected = vec![
                None,
                Some(Ae {
                    value: "STORE_SCP".to_string(),
                }),
            ];

            // Act
            let actual = Ae::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 不正なUTF-8バイト列
        {
            // Arrange
            let buf = b"\xff\xfe";

            // Act
            let result = Ae::from_buf(buf);

            // Assert
            match result.unwrap_err() {
                AeError::InvalidUtf8(_) => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let ae = Ae {
            value: "STORE_SCP".to_string(),
        };
        let expected = "STORE_SCP";

        // Act
        let actual = ae.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
