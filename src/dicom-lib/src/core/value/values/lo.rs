use crate::core::value::{self, SpecificCharacterSet};
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// 長い文字列（Long String）
#[derive(Debug, PartialEq, Clone)]
pub struct Lo {
    string: String,
}

impl Lo {
    const MAX_CHAR_COUNT: usize = 64;

    pub fn string(&self) -> &str {
        &self.string
    }

    /// 文字列から長い文字列（LO）の値をパースします。
    ///
    /// この関数は、DICOM規格に準拠したLO型の文字列をパースし、
    /// バックスラッシュ(`\`)で区切られた複数の値を処理します。
    /// 各値は最大64文字までで、制御文字（`\`を除く）以外の文字が許可されます。
    /// マルチバイト文字（日本語など）も使用可能です。
    ///
    /// # 引数
    ///
    /// * `str` - パースする文字列。複数の値はバックスラッシュ(`\`)で区切られます。
    ///
    /// # 戻り値
    ///
    /// * `Ok(Vec<Option<Self>>)` - パースに成功した場合、LO値のベクタを返します。
    ///   空白のみの値や空文字列は`None`として表現されます。
    /// * `Err(LoError)` - パースに失敗した場合、エラーを返します。
    ///   64文字を超える値がある場合にエラーとなります。
    ///
    /// # パディング処理
    ///
    /// DICOM規格に従い、文字列が偶数バイト長で末尾が空白の場合、
    /// その空白は自動的に除去されます。また、各値の前後の空白もトリミングされます。
    ///
    /// # 例
    ///
    /// ```
    /// use dicom_lib::core::value::values::Lo;
    ///
    /// // 単一の文字列をパース
    /// let result = Lo::from_string("LONG STRING VALUE").unwrap();
    /// assert_eq!(result.len(), 1);
    /// assert_eq!(result[0].as_ref().unwrap().string(), "LONG STRING VALUE");
    ///
    /// // 複数の値をパース（バックスラッシュ区切り）
    /// let result = Lo::from_string(r"VALUE1\VALUE2").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert_eq!(result[0].as_ref().unwrap().string(), "VALUE1");
    /// assert_eq!(result[1].as_ref().unwrap().string(), "VALUE2");
    ///
    /// // 空値を含むケース
    /// let result = Lo::from_string(r"\VALUE1").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert!(result[0].is_none());
    /// assert_eq!(result[1].as_ref().unwrap().string(), "VALUE1");
    ///
    /// // 前後の空白は自動的にトリミングされる
    /// let result = Lo::from_string(" LONG STRING VALUE  ").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().string(), "LONG STRING VALUE");
    ///
    /// // 日本語を含む文字列
    /// let result = Lo::from_string("山田太郎").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().string(), "山田太郎");
    ///
    /// // 最大長（64文字）
    /// let input = "1234567890123456789012345678901234567890123456789012345678901234";
    /// let result = Lo::from_string(input).unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().string(), input);
    /// ```
    ///
    /// # エラー
    ///
    /// ```
    /// use dicom_lib::core::value::values::{Lo, lo::LoError};
    ///
    /// // 65文字を超える文字列
    /// let input = "12345678901234567890123456789012345678901234567890123456789012345";
    /// let result = Lo::from_string(input);
    /// assert!(matches!(result, Err(LoError::InvalidLength { .. })));
    /// ```
    pub fn from_string(str: &str) -> Result<Vec<Option<Self>>, LoError> {
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

    pub fn from_buf_lossy(
        buf: &[u8],
        char_set: SpecificCharacterSet,
    ) -> Result<Vec<Option<Self>>, LoError> {
        let str = value::generate_string_lossy(buf, char_set);
        Self::from_string(&str)
    }

    fn from_string_single(str: &str) -> Result<Option<Self>, LoError> {
        if str.chars().count() > Self::MAX_CHAR_COUNT {
            return Err(LoError::InvalidLength {
                string: str.to_string(),
                char_count: str.chars().count(),
            });
        }

        let trimmed = str.trim_matches(' ');
        if trimmed.is_empty() {
            return Ok(None);
        }

        Ok(Some(Self {
            string: trimmed.to_string(),
        }))
    }
}

impl Display for Lo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}

#[derive(Error, Debug)]
pub enum LoError {
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
            let input = "LONG STRING VALUE";
            let expected = vec![Some(Lo {
                string: "LONG STRING VALUE".to_string(),
            })];

            // Act
            let actual = Lo::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値（バックスラッシュ区切り）
        {
            // Arrange
            let input = r"VALUE1\VALUE2";
            let expected = vec![
                Some(Lo {
                    string: "VALUE1".to_string(),
                }),
                Some(Lo {
                    string: "VALUE2".to_string(),
                }),
            ];

            // Act
            let actual = Lo::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let input = r"\VALUE1";
            let expected = vec![
                None,
                Some(Lo {
                    string: "VALUE1".to_string(),
                }),
            ];

            // Act
            let actual = Lo::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 前後に空白を含む
        {
            // Arrange
            let input = " LONG STRING VALUE  ";
            let expected = vec![Some(Lo {
                string: "LONG STRING VALUE".to_string(),
            })];

            // Act
            let actual = Lo::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空文字列
        {
            // Arrange
            let input = "";
            let expected = vec![None];

            // Act
            let actual = Lo::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最大長（64文字）
        {
            // Arrange
            let input = "1234567890123456789012345678901234567890123456789012345678901234";
            assert_eq!(input.chars().count(), 64);
            let expected = vec![Some(Lo {
                string: input.to_string(),
            })];

            // Act
            let actual = Lo::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 日本語を含む文字列
        {
            // Arrange
            let input = "山田太郎";
            let expected = vec![Some(Lo {
                string: "山田太郎".to_string(),
            })];

            // Act
            let actual = Lo::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値の最後の値が空白パディングされ、その値の長さが65になるケース
        //        詳しくは以下を参照。
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.4.html
        {
            // Arrange
            let input = r"1234567890123456789012345678901234567890123456789012345678901234\1234567890123456789012345678901234567890123456789012345678901234 ";
            let expected = vec![
                Some(Lo {
                    string: "1234567890123456789012345678901234567890123456789012345678901234"
                        .to_string(),
                }),
                Some(Lo {
                    string: "1234567890123456789012345678901234567890123456789012345678901234"
                        .to_string(),
                }),
            ];

            // Act
            let actual = Lo::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 65文字の文字列（長すぎる）
        {
            // Arrange
            let input = "12345678901234567890123456789012345678901234567890123456789012345";
            assert_eq!(input.chars().count(), 65);

            // Act
            let result = Lo::from_string(input);

            // Assert
            match result.unwrap_err() {
                LoError::InvalidLength { string, char_count } => {
                    assert_eq!(string, input);
                    assert_eq!(char_count, 65);
                }
            }
        }
    }

    #[test]
    fn test_from_buf_lossy() {
        // 正常系: 通常
        {
            // Arrange
            let buf = b"LONG STRING VALUE";
            let expected = vec![Some(Lo {
                string: "LONG STRING VALUE".to_string(),
            })];

            // Act
            let actual = Lo::from_buf_lossy(buf, SpecificCharacterSet::None).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空白パディングを含む
        {
            // Arrange
            let buf = b" LONG STRING VALUE  ";
            let expected = vec![Some(Lo {
                string: "LONG STRING VALUE".to_string(),
            })];

            // Act
            let actual = Lo::from_buf_lossy(buf, SpecificCharacterSet::None).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 日本語を含む文字列（ISO 2022 IR 6＆ISO 2022 IR 87）
        {
            // Arrange
            let buf = [
                0x1b, 0x24, 0x42, 0x3b, 0x33, 0x45, 0x44, 0x42, 0x40, 0x4f, 0x3a, 0x1b, 0x28, 0x42,
            ];
            let expected = vec![Some(Lo {
                string: "山田太郎".to_string(),
            })];

            // Act
            let actual =
                Lo::from_buf_lossy(&buf, SpecificCharacterSet::Iso2022Ir6AndIso2022Ir87).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: UTF-8の日本語文字列
        {
            // Arrange
            let buf = "山田太郎".as_bytes();
            let expected = vec![Some(Lo {
                string: "山田太郎".to_string(),
            })];

            // Act
            let actual = Lo::from_buf_lossy(buf, SpecificCharacterSet::IsoIr192).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値
        {
            // Arrange
            let buf = b"VALUE1\\VALUE2";
            let expected = vec![
                Some(Lo {
                    string: "VALUE1".to_string(),
                }),
                Some(Lo {
                    string: "VALUE2".to_string(),
                }),
            ];

            // Act
            let actual = Lo::from_buf_lossy(buf, SpecificCharacterSet::None).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let buf = b"\\VALUE1";
            let expected = vec![
                None,
                Some(Lo {
                    string: "VALUE1".to_string(),
                }),
            ];

            // Act
            let actual = Lo::from_buf_lossy(buf, SpecificCharacterSet::None).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let lo = Lo {
            string: "LONG STRING VALUE".to_string(),
        };
        let expected = "LONG STRING VALUE";

        // Act
        let actual = lo.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
