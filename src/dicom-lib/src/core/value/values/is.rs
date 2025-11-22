use std::{
    fmt::{Display, Formatter},
    num::ParseIntError,
    str::Utf8Error,
};
use thiserror::Error;

/// 整数文字列（Integer String）
#[derive(Debug, PartialEq, Clone)]
pub struct Is {
    value: i32,
}

impl Is {
    const MAX_BYTE_LENGTH: usize = 12;

    pub fn value(&self) -> i32 {
        self.value
    }

    /// 文字列から整数文字列（IS）の値をパースします。
    ///
    /// この関数は、DICOM規格に準拠したIS型の文字列をパースし、
    /// バックスラッシュ(`\`)で区切られた複数の値を処理します。
    /// 各値は最大12バイトまでで、数字（0-9）と符号（+、-）のみが許可されます。
    /// パースされる数値はi32の範囲内（-2,147,483,648 ～ 2,147,483,647）である必要があります。
    ///
    /// # 引数
    ///
    /// * `str` - パースする文字列。複数の値はバックスラッシュ(`\`)で区切られます。
    ///
    /// # 戻り値
    ///
    /// * `Ok(Vec<Option<Self>>)` - パースに成功した場合、IS値のベクタを返します。
    ///   空白のみの値や空文字列は`None`として表現されます。
    /// * `Err(IsError)` - パースに失敗した場合、エラーを返します。
    ///   不正な文字が含まれている場合、12バイトを超える値がある場合、
    ///   またはi32の範囲外の値の場合にエラーとなります。
    ///
    /// # パディング処理
    ///
    /// DICOM規格に従い、文字列が偶数バイト長で末尾が空白の場合、
    /// その空白は自動的に除去されます。また、各値の前後の空白もトリミングされます。
    ///
    /// # 例
    ///
    /// ```
    /// use dicom_lib::core::value::values::Is;
    ///
    /// // 正の整数をパース
    /// let result = Is::from_string("12345").unwrap();
    /// assert_eq!(result.len(), 1);
    /// assert_eq!(result[0].as_ref().unwrap().value(), 12345);
    ///
    /// // 負の整数をパース
    /// let result = Is::from_string("-12345").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().value(), -12345);
    ///
    /// // プラス記号付きの整数をパース
    /// let result = Is::from_string("+12345").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().value(), 12345);
    ///
    /// // 複数の値をパース（バックスラッシュ区切り）
    /// let result = Is::from_string(r"100\200\300").unwrap();
    /// assert_eq!(result.len(), 3);
    /// assert_eq!(result[0].as_ref().unwrap().value(), 100);
    /// assert_eq!(result[1].as_ref().unwrap().value(), 200);
    /// assert_eq!(result[2].as_ref().unwrap().value(), 300);
    ///
    /// // 空値を含むケース
    /// let result = Is::from_string(r"\123").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert!(result[0].is_none());
    /// assert_eq!(result[1].as_ref().unwrap().value(), 123);
    ///
    /// // 前後の空白は自動的にトリミングされる
    /// let result = Is::from_string(" 123  ").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().value(), 123);
    ///
    /// // i32の最大値・最小値
    /// let result = Is::from_string("2147483647").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().value(), 2147483647);
    ///
    /// let result = Is::from_string("-2147483648").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().value(), -2147483648);
    /// ```
    ///
    /// # エラー
    ///
    /// ```
    /// use dicom_lib::core::value::values::{Is, is::IsError};
    ///
    /// // 13バイトを超える文字列
    /// let result = Is::from_string("1234567890123");
    /// assert!(matches!(result, Err(IsError::InvalidLength { .. })));
    ///
    /// // 不正な文字を含む（小数点）
    /// let result = Is::from_string("123.45");
    /// assert!(matches!(result, Err(IsError::InvalidCharacter { .. })));
    ///
    /// // アルファベットを含む
    /// let result = Is::from_string("123A");
    /// assert!(matches!(result, Err(IsError::InvalidCharacter { .. })));
    ///
    /// // i32の範囲外（2^31以上）
    /// let result = Is::from_string("2147483648");
    /// assert!(matches!(result, Err(IsError::ParseError { .. })));
    ///
    /// // i32の範囲外（-2^31未満）
    /// let result = Is::from_string("-2147483649");
    /// assert!(matches!(result, Err(IsError::ParseError { .. })));
    ///
    /// // 符号が途中にある
    /// let result = Is::from_string("12-34");
    /// assert!(matches!(result, Err(IsError::ParseError { .. })));
    /// ```
    pub fn from_string(str: &str) -> Result<Vec<Option<Self>>, IsError> {
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

    pub fn from_buf(buf: &[u8]) -> Result<Vec<Option<Self>>, IsError> {
        let str = str::from_utf8(buf).map_err(IsError::InvalidUtf8)?;
        Self::from_string(str)
    }

    fn from_string_single(str: &str) -> Result<Option<Self>, IsError> {
        if str.len() > Self::MAX_BYTE_LENGTH {
            return Err(IsError::InvalidLength {
                string: str.to_string(),
                byte_length: str.len(),
            });
        }

        let trimmed = str.trim_matches(' ');
        if trimmed.is_empty() {
            return Ok(None);
        }

        // 各文字が許可された文字（数字、+、-）であることを確認
        for (i, c) in trimmed.chars().enumerate() {
            if !matches!(c, '0'..='9' | '+' | '-') {
                return Err(IsError::InvalidCharacter {
                    string: trimmed.to_string(),
                    character: c,
                    position: i,
                });
            }
        }

        let value = trimmed.parse::<i32>().map_err(|e| IsError::ParseError {
            string: trimmed.to_string(),
            error: e,
        })?;

        Ok(Some(Self { value }))
    }
}

impl Display for Is {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Error, Debug)]
pub enum IsError {
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
    fn test_from_string() {
        // 正常系: 通常の正の整数
        {
            // Arrange
            let input = "12345";
            let expected = vec![Some(Is { value: 12345 })];

            // Act
            let actual = Is::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 負の整数
        {
            // Arrange
            let input = "-12345";
            let expected = vec![Some(Is { value: -12345 })];

            // Act
            let actual = Is::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: プラス記号付きの整数
        {
            // Arrange
            let input = "+12345";
            let expected = vec![Some(Is { value: 12345 })];

            // Act
            let actual = Is::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: ゼロ
        {
            // Arrange
            let input = "0";
            let expected = vec![Some(Is { value: 0 })];

            // Act
            let actual = Is::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最大値 (2^31 - 1)
        {
            // Arrange
            let input = "2147483647";
            let expected = vec![Some(Is { value: 2147483647 })];

            // Act
            let actual = Is::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最小値 (-2^31)
        {
            // Arrange
            let input = "-2147483648";
            let expected = vec![Some(Is { value: -2147483648 })];

            // Act
            let actual = Is::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値（バックスラッシュ区切り）
        {
            // Arrange
            let input = r"100\200\300";
            let expected = vec![
                Some(Is { value: 100 }),
                Some(Is { value: 200 }),
                Some(Is { value: 300 }),
            ];

            // Act
            let actual = Is::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let input = r"\123";
            let expected = vec![None, Some(Is { value: 123 })];

            // Act
            let actual = Is::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 前後に空白を含む文字列（空白は削除される）
        {
            // Arrange
            let input = " 123  ";
            let expected = vec![Some(Is { value: 123 })];

            // Act
            let actual = Is::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空文字列
        {
            // Arrange
            let input = "";
            let expected = vec![None];

            // Act
            let actual = Is::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最大長（12バイト）
        {
            // Arrange
            let input = "-1234567890 ";
            assert_eq!(input.len(), 12);
            let expected = vec![Some(Is { value: -1234567890 })];

            // Act
            let actual = Is::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 13バイトの文字列（長すぎる）
        {
            // Arrange
            let input = "1234567890123";
            assert_eq!(input.len(), 13);

            // Act
            let result = Is::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsError::InvalidLength {
                    string,
                    byte_length,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(byte_length, 13);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 不正な文字を含む文字列（ドット）
        {
            // Arrange
            let input = "123.45";
            let expected_char = '.';
            let expected_position = 3;

            // Act
            let result = Is::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, expected_char);
                    assert_eq!(position, expected_position);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: アルファベットを含む文字列
        {
            // Arrange
            let input = "123A";
            let expected_char = 'A';
            let expected_position = 3;

            // Act
            let result = Is::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, expected_char);
                    assert_eq!(position, expected_position);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 符号が途中にある文字列
        {
            // Arrange
            let input = "12-34";

            // Act
            let result = Is::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 埋め込みスペースを含む（DICOM仕様では禁止）
        {
            // Arrange
            let input = "12 34";

            // Act
            let result = Is::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsError::InvalidCharacter {
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

        // 準正常系: 範囲外の値（2^31以上）
        {
            // Arrange
            let input = "2147483648";

            // Act
            let result = Is::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 範囲外の値（-2^31未満）
        {
            // Arrange
            let input = "-2147483649";

            // Act
            let result = Is::from_string(input);

            // Assert
            match result.unwrap_err() {
                IsError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_from_buf() {
        // 正常系: 通常の整数
        {
            // Arrange
            let buf = b"12345";
            let expected = vec![Some(Is { value: 12345 })];

            // Act
            let actual = Is::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 負の整数
        {
            // Arrange
            let buf = b"-12345";
            let expected = vec![Some(Is { value: -12345 })];

            // Act
            let actual = Is::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空白パディングを含む整数
        {
            // Arrange
            let buf = b"123  ";
            let expected = vec![Some(Is { value: 123 })];

            // Act
            let actual = Is::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値
        {
            // Arrange
            let buf = b"100\\200";
            let expected = vec![Some(Is { value: 100 }), Some(Is { value: 200 })];

            // Act
            let actual = Is::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let buf = b"\\123";
            let expected = vec![None, Some(Is { value: 123 })];

            // Act
            let actual = Is::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 不正なUTF-8バイト列
        {
            // Arrange
            let buf = b"\xff\xfe";

            // Act
            let result = Is::from_buf(buf);

            // Assert
            match result.unwrap_err() {
                IsError::InvalidUtf8(_) => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // 正常系: 正の整数
        {
            // Arrange
            let is = Is { value: 12345 };
            let expected = "12345";

            // Act
            let actual = is.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 負の整数
        {
            // Arrange
            let is = Is { value: -12345 };
            let expected = "-12345";

            // Act
            let actual = is.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: ゼロ
        {
            // Arrange
            let is = Is { value: 0 };
            let expected = "0";

            // Act
            let actual = is.to_string();

            // Assert
            assert_eq!(expected, actual);
        }
    }
}
