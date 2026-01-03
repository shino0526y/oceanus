//! ASCII文字セットのエンコーディング

pub(crate) const ESCAPE_SEQUENCE: u32 = 0x1b2842;

/// バイト配列におけるエスケープシーケンスを含まない指定範囲をデコードし、文字列を生成する。
/// 外字は`'�'`に置き換えられる。
pub(crate) fn generate_string_lossy(bytes: &[u8], index: usize, length: usize) -> String {
    debug_assert!(index < bytes.len());
    debug_assert!(length <= bytes.len());
    debug_assert!(index + length <= bytes.len());

    bytes[index..index + length]
        .iter()
        .map(|&code| match is_valid_character_code(code) {
            true => code as char,
            false => char::REPLACEMENT_CHARACTER,
        })
        .collect()
}

pub(super) fn is_valid_character_code(code: u8) -> bool {
    // DICOMではASCII文字セット内の一部の制御文字しか使用しない。
    // 具体的には以下の制御文字が該当する。
    //   - LF (Line Feed) ... 0x0a
    //   - FF (Form Feed) ... 0x0c
    //   - CR (Carriage Return) ... 0x0d
    //   - ESC (Escape) ... 0x1b
    //   - TAB (Horizontal Tab) ... 0x09
    //
    // 上記以外の制御文字については外字として扱う。
    //
    // https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/chapter_e.html
    // https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/chapter_6.html#sect_6.1.3

    matches!(code, 0x09 | 0x0a | 0x0c | 0x0d | 0x1b | 0x20..=0x7f)
}
