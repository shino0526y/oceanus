mod specific_character_set;
pub mod values;

pub use specific_character_set::SpecificCharacterSet;
use specific_character_set::{
    iso_2022_ir_6_and_iso_2022_ir_13_and_iso_2022_ir_87, iso_2022_ir_6_and_iso_2022_ir_87,
    iso_2022_ir_13_and_iso_2022_ir_87, iso_ir_13, iso_ir_192, none,
};

fn generate_string_lossy(bytes: &[u8], char_set: SpecificCharacterSet) -> String {
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

fn generate_person_name_strings_lossy(bytes: &[u8], char_set: SpecificCharacterSet) -> Vec<String> {
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
