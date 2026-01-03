use super::encoding::{jis_x_0201_katakana, jis_x_0201_romaji};

#[derive(Clone, Copy, PartialEq, Eq)]
enum CharSet {
    IsoIr13,
    IsoIr14,
}

pub(crate) fn generate_string_lossy(bytes: &[u8]) -> String {
    generate_string_lossy_with_range(bytes, 0, bytes.len())
}

pub(crate) fn generate_person_name_strings_lossy(bytes: &[u8]) -> Vec<String> {
    generate_string_lossy(bytes)
        .trim_end_matches(' ')
        .split('\\')
        .map(|s| s.to_string())
        .collect()
}

pub(super) fn generate_string_lossy_with_range(
    bytes: &[u8],
    index: usize,
    length: usize,
) -> String {
    let mut str = String::new();

    let mut char_set = CharSet::IsoIr13;
    let mut start_index = index;

    let mut i = index;
    while i < index + length {
        let char_code = bytes[i];

        let temp_char_set;
        if jis_x_0201_katakana::is_valid_character_code(char_code) {
            temp_char_set = CharSet::IsoIr13;
        } else if jis_x_0201_romaji::is_valid_character_code(char_code) {
            temp_char_set = CharSet::IsoIr14;
        } else {
            temp_char_set = char_set;
        }

        if temp_char_set == char_set {
            i += 1;
            continue;
        }

        let temp_str = generate_string_for_char_set(bytes, start_index, i - start_index, char_set);
        str.push_str(&temp_str);

        char_set = temp_char_set;
        start_index = i;

        i += 1;
    }

    let temp_str =
        generate_string_for_char_set(bytes, start_index, index + length - start_index, char_set);
    str.push_str(&temp_str);

    str
}

fn generate_string_for_char_set(
    bytes: &[u8],
    index: usize,
    length: usize,
    char_set: CharSet,
) -> String {
    if length == 0 {
        return String::new();
    }

    match char_set {
        CharSet::IsoIr13 => jis_x_0201_katakana::generate_string_lossy(bytes, index, length),
        CharSet::IsoIr14 => jis_x_0201_romaji::generate_string_lossy(bytes, index, length),
    }
}
