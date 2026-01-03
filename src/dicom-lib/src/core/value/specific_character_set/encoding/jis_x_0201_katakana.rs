//! JIS X 0201（半角カタカナ）文字セットのエンコーディング

pub(crate) const ESCAPE_SEQUENCE: u32 = 0x1b2949;

const MIN_KANA_CHAR_CODE_JIS_X_0201: u8 = 0xa1;
const MAX_KANA_CHAR_CODE_JIS_X_0201: u8 = 0xdf;
const MIN_KANA_CODE_POINT_UNICODE: u32 = 0xff61;

/// バイト配列におけるエスケープシーケンスを含まない指定範囲をデコードし、文字列を生成する。
/// 外字は`'�'`に置き換えられる。
pub(crate) fn generate_string_lossy(bytes: &[u8], index: usize, length: usize) -> String {
    debug_assert!(index < bytes.len());
    debug_assert!(length <= bytes.len());
    debug_assert!(index + length <= bytes.len());

    bytes[index..index + length]
        .iter()
        .map(|&code| match is_valid_character_code(code) {
            true => unsafe {
                char::from_u32_unchecked(
                    MIN_KANA_CODE_POINT_UNICODE - MIN_KANA_CHAR_CODE_JIS_X_0201 as u32
                        + code as u32,
                )
            },
            false => char::REPLACEMENT_CHARACTER,
        })
        .collect()
}

pub(crate) fn is_valid_character_code(code: u8) -> bool {
    (MIN_KANA_CHAR_CODE_JIS_X_0201..=MAX_KANA_CHAR_CODE_JIS_X_0201).contains(&code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_string_lossy() {
        // 正常系
        {
            let expected = "｡｢｣､･ｦｧｨｩｪｫｬｭｮｯｰｱｲｳｴｵｶｷｸｹｺｻｼｽｾｿﾀﾁﾂﾃﾄﾅﾆﾇﾈﾉﾊﾋﾌﾍﾎﾏﾐﾑﾒﾓﾔﾕﾖﾗﾘﾙﾚﾛﾜﾝﾞﾟ";

            let actual = {
                let bytes = [
                    0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad,
                    0xae, 0xaf, 0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba,
                    0xbb, 0xbc, 0xbd, 0xbe, 0xbf, 0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7,
                    0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf, 0xd0, 0xd1, 0xd2, 0xd3, 0xd4,
                    0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf,
                ];
                generate_string_lossy(&bytes, 0, bytes.len())
            };

            assert_eq!(expected, actual);
        }

        // 準正常系
        {
            let expected = "�ｽ��ｽ��ｽ��ｽ��ｽ�";

            let actual = {
                let bytes = [
                    // UTF-8のバイト列
                    0xef, 0xbd, 0xe1, // 'ｱ'
                    0xef, 0xbd, 0xe2, // 'ｲ'
                    0xef, 0xbd, 0xe3, // 'ｳ'
                    0xef, 0xbd, 0xe4, // 'ｴ'
                    0xef, 0xbd, 0xe5, // 'ｵ'
                ];
                generate_string_lossy(&bytes, 0, bytes.len())
            };

            assert_eq!(expected, actual);
        }
    }
}
