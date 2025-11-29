use std::fmt::{Display, Formatter};
use thiserror::Error;

/// 倍精度浮動小数点数（Floating Point Double）
#[derive(Debug, PartialEq, Clone)]
pub struct Fd {
    value: f64,
}

impl Fd {
    const BYTES_PER_VALUE: usize = 8;

    pub fn value(&self) -> f64 {
        self.value
    }

    /// バイト列から倍精度浮動小数点数（FD）の値をパースします。
    ///
    /// この関数は、DICOM規格に準拠したFD型のバイナリデータをパースします。
    /// FDはIEEE 754 binary64形式の倍精度浮動小数点数で、
    /// NaN（非数）や無限大を含む全ての値が許可されます。
    /// 各値は8バイトで、複数の値が連続して格納されている場合があります。
    /// バイト列はリトルエンディアン形式である必要があります。
    ///
    /// # 引数
    ///
    /// * `buf` - パースするバイト列。8バイトの倍数である必要があります。
    ///
    /// # 戻り値
    ///
    /// * `Ok(Vec<Self>)` - パースに成功した場合、FD値のベクターを返します。
    ///   空のバッファの場合は、空のベクターを返します。
    /// * `Err(FdError)` - パースに失敗した場合、エラーを返します。
    ///   バイト数が8の倍数でない場合にエラーとなります。
    ///
    /// # 例
    ///
    /// ```
    /// use dicom_lib::core::value::values::Fd;
    ///
    /// // 単一の値をパース
    /// let buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f]; // 1.0
    /// let result = Fd::from_buf(&buf).unwrap();
    /// assert_eq!(result.len(), 1);
    /// assert_eq!(result[0].value(), 1.0);
    ///
    /// // 複数の値をパース
    /// let buf = [
    ///     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f, // 1.0
    ///     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, // 2.0
    /// ];
    /// let result = Fd::from_buf(&buf).unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert_eq!(result[0].value(), 1.0);
    /// assert_eq!(result[1].value(), 2.0);
    ///
    /// // ゼロ
    /// let buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    /// let result = Fd::from_buf(&buf).unwrap();
    /// assert_eq!(result.len(), 1);
    /// assert_eq!(result[0].value(), 0.0);
    ///
    /// // 空のバッファ
    /// let buf = [];
    /// let result = Fd::from_buf(&buf).unwrap();
    /// assert_eq!(result.len(), 0);
    /// ```
    ///
    /// # エラー
    ///
    /// ```
    /// use dicom_lib::core::value::values::{Fd, fd::FdError};
    ///
    /// // 8バイトの倍数でないバッファ
    /// let buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    /// let result = Fd::from_buf(&buf);
    /// assert!(matches!(result, Err(FdError::InvalidLength { .. })));
    /// ```
    pub fn from_buf(buf: &[u8]) -> Result<Vec<Self>, FdError> {
        if !buf.len().is_multiple_of(Self::BYTES_PER_VALUE) {
            return Err(FdError::InvalidLength {
                byte_length: buf.len(),
            });
        }

        if buf.is_empty() {
            return Ok(Vec::new());
        }

        let mut values = Vec::with_capacity(buf.len() / Self::BYTES_PER_VALUE);
        for chunk in buf.chunks_exact(Self::BYTES_PER_VALUE) {
            let bytes: [u8; 8] = chunk.try_into().unwrap(); // chunks_exactなので必ず8バイト
            let value = f64::from_le_bytes(bytes);
            values.push(Self { value });
        }

        Ok(values)
    }
}

impl Display for Fd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Error, Debug)]
pub enum FdError {
    #[error("バイト列の長さが8の倍数ではありません (バイト数={byte_length})")]
    InvalidLength { byte_length: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_buf() {
        // 正常系: 単一の値
        {
            // Arrange
            let buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f]; // 1.0

            // Act
            let actual = Fd::from_buf(&buf).unwrap();

            // Assert
            assert_eq!(actual.len(), 1);
            assert_eq!(actual[0].value(), 1.0);
        }

        // 正常系: 複数の値
        {
            // Arrange
            let buf = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f, // 1.0
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, // 2.0
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x40, // 3.0
            ];

            // Act
            let actual = Fd::from_buf(&buf).unwrap();

            // Assert
            assert_eq!(actual.len(), 3);
            assert_eq!(actual[0].value(), 1.0);
            assert_eq!(actual[1].value(), 2.0);
            assert_eq!(actual[2].value(), 3.0);
        }

        // 正常系: ゼロ
        {
            // Arrange
            let buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

            // Act
            let actual = Fd::from_buf(&buf).unwrap();

            // Assert
            assert_eq!(actual.len(), 1);
            assert_eq!(actual[0].value(), 0.0);
        }

        // 正常系: 負の値
        {
            // Arrange
            let buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0xbf]; // -1.0

            // Act
            let actual = Fd::from_buf(&buf).unwrap();

            // Assert
            assert_eq!(actual.len(), 1);
            assert_eq!(actual[0].value(), -1.0);
        }

        // 正常系: 空のバッファ
        {
            // Arrange
            let buf = [];

            // Act
            let actual = Fd::from_buf(&buf).unwrap();

            // Assert
            assert_eq!(actual.len(), 0);
        }

        // 正常系: 正の無限大
        {
            // Arrange
            let buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x7f];

            // Act
            let actual = Fd::from_buf(&buf).unwrap();

            // Assert
            assert_eq!(actual.len(), 1);
            assert_eq!(actual[0].value(), f64::INFINITY);
        }

        // 正常系: 負の無限大
        {
            // Arrange
            let buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0xff];

            // Act
            let actual = Fd::from_buf(&buf).unwrap();

            // Assert
            assert_eq!(actual.len(), 1);
            assert_eq!(actual[0].value(), f64::NEG_INFINITY);
        }

        // 正常系: NaN
        {
            // Arrange
            let buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0x7f];

            // Act
            let actual = Fd::from_buf(&buf).unwrap();

            // Assert
            assert_eq!(actual.len(), 1);
            assert!(actual[0].value().is_nan());
        }

        // 正常系: 非常に小さな値
        {
            // Arrange
            let buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb0, 0x3c];

            // Act
            let actual = Fd::from_buf(&buf).unwrap();

            // Assert
            // リトルエンディアンの[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb0, 0x3c]は
            // f64の約2.22e-16に相当
            assert_eq!(actual.len(), 1);
            assert!((actual[0].value() - 2.220446049250313e-16).abs() < 1e-30);
        }

        // 正常系: 非常に大きな値
        {
            // Arrange
            let buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0xe0, 0x7e, 0x54];

            // Act
            let actual = Fd::from_buf(&buf).unwrap();

            // Assert
            // リトルエンディアンの[0x00, 0x00, 0x00, 0x00, 0x00, 0xe0, 0x7e, 0x54]は
            // f64の約1.055e99に相当
            assert_eq!(actual.len(), 1);
            assert!((actual[0].value() - 1.0551775957449296e99).abs() < 1e85);
        }

        // 準正常系: 1バイト（8の倍数でない）
        {
            // Arrange
            let buf = [0x01];

            // Act
            let result = Fd::from_buf(&buf);

            // Assert
            match result.unwrap_err() {
                FdError::InvalidLength { byte_length } => {
                    assert_eq!(byte_length, 1);
                }
            }
        }

        // 準正常系: 7バイト（8の倍数でない）
        {
            // Arrange
            let buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

            // Act
            let result = Fd::from_buf(&buf);

            // Assert
            match result.unwrap_err() {
                FdError::InvalidLength { byte_length } => {
                    assert_eq!(byte_length, 7);
                }
            }
        }

        // 準正常系: 9バイト（8の倍数でない）
        {
            // Arrange
            let buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

            // Act
            let result = Fd::from_buf(&buf);

            // Assert
            match result.unwrap_err() {
                FdError::InvalidLength { byte_length } => {
                    assert_eq!(byte_length, 9);
                }
            }
        }
    }

    #[test]
    fn test_to_string() {
        // 正常系: 通常の値
        {
            // Arrange
            let fd = Fd { value: 1.5 };
            let expected = "1.5";

            // Act
            let actual = fd.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: ゼロ
        {
            // Arrange
            let fd = Fd { value: 0.0 };
            let expected = "0";

            // Act
            let actual = fd.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 負の値
        {
            // Arrange
            let fd = Fd { value: -1.5 };
            let expected = "-1.5";

            // Act
            let actual = fd.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 整数値
        {
            // Arrange
            let fd = Fd { value: 2.0 };
            let expected = "2";

            // Act
            let actual = fd.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 無限大
        {
            // Arrange
            let fd = Fd {
                value: f64::INFINITY,
            };
            let expected = "inf";

            // Act
            let actual = fd.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 負の無限大
        {
            // Arrange
            let fd = Fd {
                value: f64::NEG_INFINITY,
            };
            let expected = "-inf";

            // Act
            let actual = fd.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: NaN
        {
            // Arrange
            let fd = Fd { value: f64::NAN };
            let expected = "NaN";

            // Act
            let actual = fd.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 小さな値
        {
            // Arrange
            let fd = Fd { value: 0.001 };
            let expected = "0.001";

            // Act
            let actual = fd.to_string();

            // Assert
            assert_eq!(expected, actual);
        }
    }
}
