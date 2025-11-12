mod specific_character_set;

pub use specific_character_set::SpecificCharacterSet;
use specific_character_set::{
    iso_2022_ir_6_and_iso_2022_ir_13_and_iso_2022_ir_87, iso_2022_ir_6_and_iso_2022_ir_87,
    iso_2022_ir_13_and_iso_2022_ir_87, iso_ir_13, iso_ir_192, none,
};

const BYTES_LEN_ERR_MESSAGE: &str = "引数に指定されたバイト列の長さが不正です";

macro_rules! fn_bytes_into_numbers {
    ($type:ty, $fn_name:ident) => {
        pub fn $fn_name(bytes: &[u8]) -> Result<Vec<$type>, &'static str> {
            if bytes.is_empty() {
                return Ok(vec![]);
            }

            const TYPE_SIZE: usize = std::mem::size_of::<$type>();
            if bytes.len() % TYPE_SIZE != 0 {
                return Err(BYTES_LEN_ERR_MESSAGE);
            }

            let value_count = bytes.len() / TYPE_SIZE;
            let mut values = Vec::with_capacity(value_count);
            for i in 0..value_count {
                unsafe {
                    let value = <$type>::from_le_bytes(
                        *(bytes.as_ptr().add(i * TYPE_SIZE) as *const [_; TYPE_SIZE]),
                    );
                    values.push(value);
                }
            }

            Ok(values)
        }
    };
}

fn_bytes_into_numbers!(u16, generate_u16_values);
fn_bytes_into_numbers!(u32, generate_u32_values);
fn_bytes_into_numbers!(u64, generate_u64_values);
fn_bytes_into_numbers!(i16, generate_i16_values);
fn_bytes_into_numbers!(i32, generate_i32_values);
fn_bytes_into_numbers!(i64, generate_i64_values);
fn_bytes_into_numbers!(f32, generate_f32_values);
fn_bytes_into_numbers!(f64, generate_f64_values);

pub fn generate_string_lossy(bytes: &[u8], char_set: SpecificCharacterSet) -> String {
    match char_set {
        SpecificCharacterSet::None => none::generate_string_lossy(bytes),
        SpecificCharacterSet::IsoIr13 => iso_ir_13::generate_string_lossy(bytes),
        SpecificCharacterSet::IsoIr192 => iso_ir_192::generate_string_lossy(bytes),
        SpecificCharacterSet::Iso2022Ir6AndIso2022Ir87 => {
            iso_2022_ir_6_and_iso_2022_ir_87::generate_string_lossy(bytes)
        }
        SpecificCharacterSet::Iso2022Ir6AndIso2022Ir13AndIso2022Ir87 => {
            iso_2022_ir_6_and_iso_2022_ir_13_and_iso_2022_ir_87::generate_string_lossy(bytes)
        }
        SpecificCharacterSet::Iso2022Ir13AndIso2022Ir87 => {
            iso_2022_ir_13_and_iso_2022_ir_87::generate_string_lossy(bytes)
        }
    }
}

pub fn generate_person_name_strings_lossy(
    bytes: &[u8],
    char_set: SpecificCharacterSet,
) -> Vec<String> {
    if bytes.is_empty() {
        return vec!["".to_string()];
    }

    match char_set {
        SpecificCharacterSet::None => none::generate_person_name_strings_lossy(bytes),
        SpecificCharacterSet::IsoIr13 => iso_ir_13::generate_person_name_strings_lossy(bytes),
        SpecificCharacterSet::IsoIr192 => iso_ir_192::generate_person_name_strings_lossy(bytes),
        SpecificCharacterSet::Iso2022Ir6AndIso2022Ir87 => {
            iso_2022_ir_6_and_iso_2022_ir_87::generate_person_name_strings_lossy(bytes)
        }
        SpecificCharacterSet::Iso2022Ir6AndIso2022Ir13AndIso2022Ir87 => {
            iso_2022_ir_6_and_iso_2022_ir_13_and_iso_2022_ir_87::generate_person_name_strings_lossy(
                bytes,
            )
        }
        SpecificCharacterSet::Iso2022Ir13AndIso2022Ir87 => {
            iso_2022_ir_13_and_iso_2022_ir_87::generate_person_name_strings_lossy(bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_person_name_strings_lossy() {
        // 正常系
        {
            // シングルバイト文字領域ではASCIIが、それ以外の領域ではJIS X 0208が使用される、一般的なケース
            // https://dicom.nema.org/medical/dicom/2020e/output/chtml/part05/sect_H.3.html
            {
                let source = [
                    0x59, 0x61, 0x6d, 0x61, 0x64, 0x61, 0x5e, 0x54, 0x61, 0x72, 0x6f, 0x75, 0x3d,
                    0x1b, 0x24, 0x42, 0x3b, 0x33, 0x45, 0x44, 0x1b, 0x28, 0x42, 0x5e, 0x1b, 0x24,
                    0x42, 0x42, 0x40, 0x4f, 0x3a, 0x1b, 0x28, 0x42, 0x3d, 0x1b, 0x24, 0x42, 0x24,
                    0x64, 0x24, 0x5e, 0x24, 0x40, 0x1b, 0x28, 0x42, 0x5e, 0x1b, 0x24, 0x42, 0x24,
                    0x3f, 0x24, 0x6d, 0x24, 0x26, 0x1b, 0x28, 0x42,
                ];
                let expected = vec!["Yamada^Tarou=山田^太郎=やまだ^たろう"];

                let actual = generate_person_name_strings_lossy(
                    &source,
                    SpecificCharacterSet::Iso2022Ir6AndIso2022Ir87,
                );

                assert_eq!(expected, actual);
            }

            // シングルバイト文字領域ではJIS X 0201（半角カタカナ＆ローマ字）が、それ以外の領域ではJIS X 0208が使用されるケース
            // https://dicom.nema.org/medical/dicom/2020e/output/chtml/part05/sect_H.3.2.html
            {
                let source = [
                    0xd4, 0xcf, 0xc0, 0xde, 0x5e, 0xc0, 0xdb, 0xb3, 0x3d, 0x1b, 0x24, 0x42, 0x3b,
                    0x33, 0x45, 0x44, 0x1b, 0x28, 0x4a, 0x5e, 0x1b, 0x24, 0x42, 0x42, 0x40, 0x4f,
                    0x3a, 0x1b, 0x28, 0x4a, 0x3d, 0x1b, 0x24, 0x42, 0x24, 0x64, 0x24, 0x5e, 0x24,
                    0x40, 0x1b, 0x28, 0x4a, 0x5e, 0x1b, 0x24, 0x42, 0x24, 0x3f, 0x24, 0x6d, 0x24,
                    0x26, 0x1b, 0x28, 0x4a,
                ];
                let expected = vec!["ﾔﾏﾀﾞ^ﾀﾛｳ=山田^太郎=やまだ^たろう"];

                let actual = generate_person_name_strings_lossy(
                    &source,
                    SpecificCharacterSet::Iso2022Ir13AndIso2022Ir87,
                );

                assert_eq!(expected, actual);
            }

            // シングルバイト文字領域ではASCIIが、表意文字領域ではJIS X 0208が、表音文字領域ではJIS X 0201（半角カタカナ＆ローマ字）が使用されるケース
            {
                let source = [
                    0x59, 0x61, 0x6d, 0x61, 0x64, 0x61, 0x5e, 0x54, 0x61, 0x72, 0x6f, 0x75, 0x3d,
                    0x1b, 0x24, 0x42, 0x3b, 0x33, 0x45, 0x44, 0x1b, 0x28, 0x42, 0x5e, 0x1b, 0x24,
                    0x42, 0x42, 0x40, 0x4f, 0x3a, 0x1b, 0x28, 0x42, 0x3d, 0x1b, 0x29, 0x49, 0xd4,
                    0xcf, 0xc0, 0xde, 0x1b, 0x28, 0x42, 0x5e, 0x1b, 0x29, 0x49, 0xc0, 0xdb, 0xb3,
                    0x1b, 0x28, 0x42, 0x20,
                ];
                let expected = vec!["Yamada^Tarou=山田^太郎=ﾔﾏﾀﾞ^ﾀﾛｳ"];

                let actual = generate_person_name_strings_lossy(
                    &source,
                    SpecificCharacterSet::Iso2022Ir6AndIso2022Ir13AndIso2022Ir87,
                );

                assert_eq!(expected, actual);
            }

            // UTF-8が使用されるケース
            {
                let source = [
                    0x59, 0x61, 0x6d, 0x61, 0x64, 0x61, 0x5e, 0x54, 0x61, 0x72, 0x6f, 0x75, 0x3d,
                    0xe5, 0xb1, 0xb1, 0xe7, 0x94, 0xb0, 0x5e, 0xe5, 0xa4, 0xaa, 0xe9, 0x83, 0x8e,
                    0x3d, 0xe3, 0x82, 0x84, 0xe3, 0x81, 0xbe, 0xe3, 0x81, 0xa0, 0x5e, 0xe3, 0x81,
                    0x9f, 0xe3, 0x82, 0x8d, 0xe3, 0x81, 0x86,
                ];
                let expected = vec!["Yamada^Tarou=山田^太郎=やまだ^たろう"];

                let actual =
                    generate_person_name_strings_lossy(&source, SpecificCharacterSet::IsoIr192);

                assert_eq!(expected, actual);
            }
        }

        // 準正常系
        {
            // JIS X 0208の外字であるShift_JISエンコーディングの「髙」が含まれているケース
            {
                let source = [
                    0x54, 0x61, 0x6b, 0x61, 0x68, 0x61, 0x73, 0x68, 0x69, 0x5e, 0x44, 0x61, 0x69,
                    0x73, 0x75, 0x6b, 0x65, 0x3d, 0x1b, 0x24, 0x42, 0xfb, 0xfc, 0x36, 0x36, 0x1b,
                    0x28, 0x42, 0x5e, 0x1b, 0x24, 0x42, 0x42, 0x67, 0x4a, 0x65, 0x1b, 0x28, 0x42,
                    0x3d, 0x1b, 0x24, 0x42, 0x24, 0x3f, 0x24, 0x2b, 0x24, 0x4f, 0x24, 0x37, 0x1b,
                    0x28, 0x42, 0x5e, 0x1b, 0x24, 0x42, 0x24, 0x40, 0x24, 0x24, 0x24, 0x39, 0x24,
                    0x31, 0x1b, 0x28, 0x42, 0x20,
                ];
                let expected = vec!["Takahashi^Daisuke=�橋^大輔=たかはし^だいすけ"];

                let actual = generate_person_name_strings_lossy(
                    &source,
                    SpecificCharacterSet::Iso2022Ir6AndIso2022Ir87,
                );

                assert_eq!(expected, actual);
            }
        }
    }
}
