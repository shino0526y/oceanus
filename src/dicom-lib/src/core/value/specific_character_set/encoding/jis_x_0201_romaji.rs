//! JIS X 0201（ローマ字）文字セットのエンコーディング

use super::ascii;

pub(crate) const ESCAPE_SEQUENCE: u32 = 0x1b284a;

/// バイト配列におけるエスケープシーケンスを含まない指定範囲をデコードし、文字列を生成する。
/// 外字は`'�'`に置き換えられる。
pub(crate) fn generate_string_lossy(bytes: &[u8], index: usize, length: usize) -> String {
    debug_assert!(index < bytes.len());
    debug_assert!(length <= bytes.len());
    debug_assert!(index + length <= bytes.len());

    bytes[index..index + length]
        .iter()
        .map(|&code| match is_valid_character_code(code) {
            true => match code {
                0x5c => '¥',
                0x7e => '‾',
                _ => code as char,
            },
            false => char::REPLACEMENT_CHARACTER,
        })
        .collect()
}

pub(crate) fn is_valid_character_code(code: u8) -> bool {
    // ここではASCII文字セットの外字判定処理を流用する。
    ascii::is_valid_character_code(code)
}
