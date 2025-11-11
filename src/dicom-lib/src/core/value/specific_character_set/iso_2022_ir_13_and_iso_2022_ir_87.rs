use super::{
    encoding::{jis_x_0201_katakana, jis_x_0201_romaji, jis_x_0208},
    iso_ir_13,
};

pub(crate) fn generate_string_lossy(bytes: &[u8]) -> String {
    generate_string_lossy_with_range(bytes, 0, bytes.len())
}

pub(crate) fn generate_patient_name_strings_lossy(bytes: &[u8]) -> Vec<String> {
    let mut values = vec![];

    let mut escape_sequence = jis_x_0201_romaji::ESCAPE_SEQUENCE;
    let mut start_index = 0;

    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && escape_sequence == jis_x_0201_romaji::ESCAPE_SEQUENCE {
            // ここに到達したということは、VMが2以上、つまり値が複数あり、そのうえで、値の区切りが出現したということ。
            // 区切りの直前の値を取得し、Vecに追加する。
            values.push(generate_patient_name_string_lossy(
                bytes,
                start_index,
                i - start_index,
            ));

            start_index = i + 1;
            i += 2;
            continue;
        } else if bytes[i] != 0x1b {
            // ESCが現れない場合は次のループに進む。
            i += 1;
            continue;
        }

        // ここに到達したということはESCが現れたということ。
        // エスケープシーケンスを取得する。
        if i + 2 >= bytes.len() {
            break;
        }
        let temp_escape_sequence = u32::from_be_bytes([0, bytes[i], bytes[i + 1], bytes[i + 2]]);
        match temp_escape_sequence {
            jis_x_0201_katakana::ESCAPE_SEQUENCE
            | jis_x_0201_romaji::ESCAPE_SEQUENCE
            | jis_x_0208::ESCAPE_SEQUENCE => {
                escape_sequence = temp_escape_sequence;
                i += 3;
            }
            _ => i += 1,
        }
    }

    values.push(generate_patient_name_string_lossy(
        bytes,
        start_index,
        bytes.len() - start_index,
    ));

    values
}

fn generate_string_lossy_with_range(bytes: &[u8], index: usize, length: usize) -> String {
    let mut str = String::new();

    let mut escape_sequence = jis_x_0201_romaji::ESCAPE_SEQUENCE;
    let mut start_index = index;

    let mut i = index;
    while i < index + length {
        if bytes[i] != 0x1b {
            // ESCが現れない場合は次のループに進む。
            i += 1;
            continue;
        }

        // ここに到達したということはESCが現れたということ。
        // エスケープシーケンスに応じた文字列を生成した上で結合する。
        let temp_str = generate_string_for_escape_sequence(
            bytes,
            start_index,
            i - start_index,
            escape_sequence,
        );
        str.push_str(&temp_str);

        // 次のエスケープシーケンスを取得する。取得できない場合はループを抜ける。
        if i + 2 >= index + length {
            break;
        }
        let temp_escape_sequence = u32::from_be_bytes([0, bytes[i], bytes[i + 1], bytes[i + 2]]);
        match temp_escape_sequence {
            jis_x_0201_katakana::ESCAPE_SEQUENCE
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

    let temp_str = generate_string_for_escape_sequence(
        bytes,
        start_index,
        index + length - start_index,
        escape_sequence,
    );
    str.push_str(&temp_str);

    str
}

fn generate_patient_name_string_lossy(bytes: &[u8], index: usize, length: usize) -> String {
    let mut start_index = index;

    let mut i = index;
    while i < index + length {
        let code = bytes[i];

        if code == b'=' {
            break;
        }

        i += 1;
    }

    let mut end_index = i;

    let mut str = String::new();
    let temp_str = iso_ir_13::generate_string_lossy_with_range(
        bytes,
        start_index,
        end_index + 1 - start_index,
    )
    .trim_end_matches(' ')
    .to_string();
    str.push_str(&temp_str);
    if end_index == index + length - 1 {
        // ここに到達したということは、"ﾔﾏﾀﾞ^ﾀﾛｳ"のように、領域区切り文字'='が含まれていなかったということ。
        // 処理を終了する。
        return str;
    }

    // ここに到達したということは、"ﾔﾏﾀﾞ^ﾀﾛｳ=山田^太郎=やまだ^たろう"のように、領域区切り文字'='が含まれていたということ。
    // シングルバイト領域の後ろの領域区切り文字'='から値の末尾までを範囲指定し、文字列を生成、結合する。
    start_index = end_index + 1;
    end_index = index + length - 1;
    let temp_str =
        generate_string_lossy_with_range(bytes, start_index, end_index + 1 - start_index)
            .trim_end_matches(' ')
            .to_string();
    str.push_str(&temp_str);

    str
}

fn generate_string_for_escape_sequence(
    bytes: &[u8],
    index: usize,
    length: usize,
    escape_sequence: u32,
) -> String {
    if length == 0 {
        return "".to_string();
    }

    match escape_sequence {
        jis_x_0201_katakana::ESCAPE_SEQUENCE => {
            iso_ir_13::generate_string_lossy_with_range(bytes, index, length)
        }
        jis_x_0201_romaji::ESCAPE_SEQUENCE => {
            iso_ir_13::generate_string_lossy_with_range(bytes, index, length)
        }
        jis_x_0208::ESCAPE_SEQUENCE => jis_x_0208::generate_string_lossy(bytes, index, length),
        _ => unreachable!(),
    }
}
