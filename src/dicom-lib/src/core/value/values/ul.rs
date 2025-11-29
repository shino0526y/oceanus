use std::fmt::{Display, Formatter};
use thiserror::Error;

/// 符号なし32ビット整数（Unsigned Long）
#[derive(Debug, PartialEq, Clone)]
pub struct Ul {
    value: u32,
}

impl Ul {
    const BYTES_PER_VALUE: usize = 4;

    pub fn value(&self) -> u32 {
        self.value
    }

    /// バイト列から符号なし32ビット整数（UL）の値をパースします。
    ///
    /// この関数は、DICOM規格に準拠したUL型のバイナリデータをパースします。
    /// ULは32ビット符号なし整数で、0から2^32-1までの値を表現できます。
    /// 各値は4バイトで、複数の値が連続して格納されている場合があります。
    /// バイト列はリトルエンディアン形式である必要があります。
    ///
    /// # 引数
    ///
    /// * `buf` - パースするバイト列。4バイトの倍数である必要があります。
    ///
    /// # 戻り値
    ///
    /// * `Ok(Vec<Self>)` - パースに成功した場合、UL値のベクターを返します。
    ///   空のバッファの場合は、空のベクターを返します。
    /// * `Err(UlError)` - パースに失敗した場合、エラーを返します。
    ///   バイト数が4の倍数でない場合にエラーとなります。
    ///
    /// # 例
    ///
    /// ```
    /// use dicom_lib::core::value::values::Ul;
    ///
    /// // 単一の値をパース
    /// let buf = [0x01, 0x00, 0x00, 0x00];
    /// let result = Ul::from_buf(&buf).unwrap();
    /// assert_eq!(result.len(), 1);
    /// assert_eq!(result[0].value(), 1);
    ///
    /// // 複数の値をパース
    /// let buf = [0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00];
    /// let result = Ul::from_buf(&buf).unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert_eq!(result[0].value(), 1);
    /// assert_eq!(result[1].value(), 2);
    ///
    /// // 最大値（2^32-1）をパース
    /// let buf = [0xff, 0xff, 0xff, 0xff];
    /// let result = Ul::from_buf(&buf).unwrap();
    /// assert_eq!(result[0].value(), 4294967295);
    ///
    /// // 空のバッファ
    /// let buf = [];
    /// let result = Ul::from_buf(&buf).unwrap();
    /// assert_eq!(result.len(), 0);
    /// ```
    ///
    /// # エラー
    ///
    /// ```
    /// use dicom_lib::core::value::values::{Ul, ul::UlError};
    ///
    /// // 4バイトの倍数でないバッファ
    /// let buf = [0x01, 0x00, 0x00];
    /// let result = Ul::from_buf(&buf);
    /// assert!(matches!(result, Err(UlError::InvalidLength { .. })));
    /// ```
    pub fn from_buf(buf: &[u8]) -> Result<Vec<Self>, UlError> {
        if !buf.len().is_multiple_of(Self::BYTES_PER_VALUE) {
            return Err(UlError::InvalidLength {
                byte_length: buf.len(),
            });
        }

        if buf.is_empty() {
            return Ok(Vec::new());
        }

        let mut values = Vec::with_capacity(buf.len() / Self::BYTES_PER_VALUE);
        for chunk in buf.chunks_exact(Self::BYTES_PER_VALUE) {
            let bytes: [u8; 4] = chunk.try_into().unwrap(); // chunks_exactなので必ず4バイト
            let value = u32::from_le_bytes(bytes);
            values.push(Self { value });
        }

        Ok(values)
    }
}

impl Display for Ul {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Error, Debug)]
pub enum UlError {
    #[error("バイト列の長さが4の倍数ではありません (バイト数={byte_length})")]
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
            let buf = [0x01, 0x00, 0x00, 0x00];

            // Act
            let actual = Ul::from_buf(&buf).unwrap();

            // Assert
            assert_eq!(actual.len(), 1);
            assert_eq!(actual[0].value(), 1);
        }

        // 正常系: 複数の値
        {
            // Arrange
            let buf = [
                0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
            ];

            // Act
            let actual = Ul::from_buf(&buf).unwrap();

            // Assert
            assert_eq!(actual.len(), 3);
            assert_eq!(actual[0].value(), 1);
            assert_eq!(actual[1].value(), 2);
            assert_eq!(actual[2].value(), 3);
        }

        // 正常系: ゼロ
        {
            // Arrange
            let buf = [0x00, 0x00, 0x00, 0x00];

            // Act
            let actual = Ul::from_buf(&buf).unwrap();

            // Assert
            assert_eq!(actual.len(), 1);
            assert_eq!(actual[0].value(), 0);
        }

        // 正常系: 最大値 (2^32 - 1)
        {
            // Arrange
            let buf = [0xff, 0xff, 0xff, 0xff];

            // Act
            let actual = Ul::from_buf(&buf).unwrap();

            // Assert
            assert_eq!(actual.len(), 1);
            assert_eq!(actual[0].value(), 4294967295);
        }

        // 正常系: 空のバッファ
        {
            // Arrange
            let buf = [];

            // Act
            let actual = Ul::from_buf(&buf).unwrap();

            // Assert
            assert_eq!(actual.len(), 0);
        }

        // 正常系: 大きな値
        {
            // Arrange
            let buf = [0x00, 0x00, 0x00, 0x80]; // 2147483648 (2^31)

            // Act
            let actual = Ul::from_buf(&buf).unwrap();

            // Assert
            assert_eq!(actual.len(), 1);
            assert_eq!(actual[0].value(), 2147483648);
        }

        // 準正常系: 1バイト（4の倍数でない）
        {
            // Arrange
            let buf = [0x01];

            // Act
            let result = Ul::from_buf(&buf);

            // Assert
            match result.unwrap_err() {
                UlError::InvalidLength { byte_length } => {
                    assert_eq!(byte_length, 1);
                }
            }
        }

        // 準正常系: 2バイト（4の倍数でない）
        {
            // Arrange
            let buf = [0x01, 0x00];

            // Act
            let result = Ul::from_buf(&buf);

            // Assert
            match result.unwrap_err() {
                UlError::InvalidLength { byte_length } => {
                    assert_eq!(byte_length, 2);
                }
            }
        }

        // 準正常系: 3バイト（4の倍数でない）
        {
            // Arrange
            let buf = [0x01, 0x00, 0x00];

            // Act
            let result = Ul::from_buf(&buf);

            // Assert
            match result.unwrap_err() {
                UlError::InvalidLength { byte_length } => {
                    assert_eq!(byte_length, 3);
                }
            }
        }

        // 準正常系: 5バイト（4の倍数でない）
        {
            // Arrange
            let buf = [0x01, 0x00, 0x00, 0x00, 0x02];

            // Act
            let result = Ul::from_buf(&buf);

            // Assert
            match result.unwrap_err() {
                UlError::InvalidLength { byte_length } => {
                    assert_eq!(byte_length, 5);
                }
            }
        }
    }

    #[test]
    fn test_to_string() {
        // 正常系: 通常の値
        {
            // Arrange
            let ul = Ul { value: 123 };
            let expected = "123";

            // Act
            let actual = ul.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: ゼロ
        {
            // Arrange
            let ul = Ul { value: 0 };
            let expected = "0";

            // Act
            let actual = ul.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最大値
        {
            // Arrange
            let ul = Ul { value: 4294967295 };
            let expected = "4294967295";

            // Act
            let actual = ul.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 小さな値
        {
            // Arrange
            let ul = Ul { value: 1 };
            let expected = "1";

            // Act
            let actual = ul.to_string();

            // Assert
            assert_eq!(expected, actual);
        }
    }
}
