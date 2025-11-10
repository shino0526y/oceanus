use super::encoding::{ascii, jis_x_0201_katakana, jis_x_0201_romaji, jis_x_0208};

pub(crate) fn generate_string_lossy(bytes: &[u8]) -> String {
    let mut str = String::new();

    let mut escape_sequence = ascii::ESCAPE_SEQUENCE;
    let mut start_index = 0;

    let mut i = start_index;
    while i < bytes.len() {
        // ESCが現れない場合は次のループに進む。
        if bytes[i] != 0x1b {
            i += 1;
            continue;
        }

        // ここに到達したということはESCが現れたということ。
        // エスケープシーケンスに応じた文字列を生成した上で結合する。
        let temp_str = generate_string_for_escape_sequence_lossy(
            bytes,
            start_index,
            i - start_index,
            escape_sequence,
        );
        str.push_str(&temp_str);

        // 次のエスケープシーケンスを取得する。取得できない場合はループを抜ける。
        if i + 2 >= bytes.len() {
            break;
        }
        let temp_escape_sequence = u32::from_be_bytes([0, bytes[i], bytes[i + 1], bytes[i + 2]]);
        match temp_escape_sequence {
            ascii::ESCAPE_SEQUENCE
            | jis_x_0201_katakana::ESCAPE_SEQUENCE
            | jis_x_0201_romaji::ESCAPE_SEQUENCE
            | jis_x_0208::ESCAPE_SEQUENCE => {
                escape_sequence = temp_escape_sequence;
                start_index = i + 3;
                i += 3;
            }
            _ => {
                // ESCを先頭に持つが既知のエスケープシーケンスではない値が現れた場合、ESC自体を文字列変換範囲に含め処理を進める。
                start_index = i;
                i += 1;
            }
        }
    }

    let temp_str = generate_string_for_escape_sequence_lossy(
        bytes,
        start_index,
        bytes.len() - start_index,
        escape_sequence,
    );
    str.push_str(&temp_str);

    str
}

pub(crate) fn generate_patient_name_strings_lossy(bytes: &[u8]) -> Vec<String> {
    generate_string_lossy(bytes)
        .trim_end_matches(' ')
        .split('\\')
        .map(|s| s.to_string())
        .collect()
}

fn generate_string_for_escape_sequence_lossy(
    bytes: &[u8],
    index: usize,
    length: usize,
    escape_sequence: u32,
) -> String {
    if length == 0 {
        return "".to_string();
    }

    match escape_sequence {
        ascii::ESCAPE_SEQUENCE => ascii::generate_string_lossy(bytes, index, length),
        jis_x_0201_katakana::ESCAPE_SEQUENCE => {
            jis_x_0201_katakana::generate_string_lossy(bytes, index, length)
        }
        jis_x_0201_romaji::ESCAPE_SEQUENCE => {
            jis_x_0201_romaji::generate_string_lossy(bytes, index, length)
        }
        jis_x_0208::ESCAPE_SEQUENCE => jis_x_0208::generate_string_lossy(bytes, index, length),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_string_lossy() {
        // 正常系
        {
            // ASCII
            {
                let source = [0x61, 0x62, 0x63, 0x64, 0x65];
                let expected = "abcde";

                let actual = generate_string_lossy(&source);

                assert_eq!(expected, actual);
            }

            // JIS X 0201（ローマ字）
            {
                let source = [
                    0x1b, 0x28, 0x4a, 0x61, 0x62, 0x63, 0x64, 0x65, 0x1b, 0x28, 0x42,
                ];
                let expected = "abcde";

                let actual = generate_string_lossy(&source);

                assert_eq!(expected, actual);
            }

            // JIS X 0201（半角カタカナ）
            {
                let source = [
                    0x1b, 0x29, 0x49, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0x1b, 0x28, 0x42,
                ];
                let expected = "ｱｲｳｴｵ";

                let actual = generate_string_lossy(&source);

                assert_eq!(expected, actual);
            }

            // JIS X 0208
            {
                let source = [
                    0x1b, 0x24, 0x42, 0x24, 0x22, 0x24, 0x24, 0x24, 0x26, 0x24, 0x28, 0x24, 0x2a,
                    0x1b, 0x28, 0x42,
                ];
                let expected = "あいうえお";

                let actual = generate_string_lossy(&source);

                assert_eq!(expected, actual);
            }
        }
    }
}
