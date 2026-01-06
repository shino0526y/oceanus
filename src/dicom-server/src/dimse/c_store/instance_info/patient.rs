use chrono::NaiveDate;
use dicom_lib::{
    core::value::{
        SpecificCharacterSet,
        value_representations::{cs::CsValue, da::DaValue, lo::LoValue, pn::PnValue},
    },
    network::service_class::storage::{Status, status::code::CannotUnderstand},
};

pub struct Patient {
    id: String,
    name_alphabet: String,
    name_kanji: String,
    name_hiragana: String,
    birth_date: Option<DaValue>,
    sex: Option<Sex>,
}

pub enum Sex {
    M,
    F,
    O,
}

impl Sex {
    pub fn to_iso_5218(&self) -> i16 {
        match self {
            Sex::M => 1, // male
            Sex::F => 2, // female
            Sex::O => 9, // not applicable
        }
    }
}

impl Patient {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name_alphabet(&self) -> &str {
        &self.name_alphabet
    }

    pub fn name_kanji(&self) -> &str {
        &self.name_kanji
    }

    pub fn name_hiragana(&self) -> &str {
        &self.name_hiragana
    }

    pub fn birth_date(&self) -> Option<&NaiveDate> {
        self.birth_date.as_ref().map(|da_value| da_value.date())
    }

    pub fn sex(&self) -> Option<&Sex> {
        self.sex.as_ref()
    }

    pub fn new(
        char_set: SpecificCharacterSet,
        patients_name: Option<PnValue>,
        patient_id: Option<LoValue>,
        patients_birth_date: Option<DaValue>,
        patients_sex: Option<CsValue>,
    ) -> Result<Self, (String, Status)> {
        let id = match patient_id {
            Some(id) => id.string().to_string(),
            None => String::new(),
        };
        let name_alphabet;
        let name_kanji;
        let name_hiragana;
        {
            if let Some(patients_name) = &patients_name {
                match char_set {
                    SpecificCharacterSet::None => {
                        // 例: "YAMADA^TARO"
                        name_alphabet = if let Some(single_byte) = patients_name.single_byte_name()
                        {
                            single_byte.to_string()
                        } else {
                            String::new()
                        };
                        name_kanji = String::new();
                        name_hiragana = String::new();
                    }
                    SpecificCharacterSet::Iso2022Ir6AndIso2022Ir87
                    | SpecificCharacterSet::IsoIr192 => {
                        // 例: "YAMADA^TARO=山田^太郎=やまだ^たろう"
                        name_alphabet = if let Some(single_byte) = patients_name.single_byte_name()
                        {
                            single_byte.to_string()
                        } else {
                            String::new()
                        };
                        name_kanji = if let Some(ideographic) = patients_name.ideographic_name() {
                            ideographic.to_string()
                        } else {
                            String::new()
                        };
                        name_hiragana = if let Some(phonetic) = patients_name.phonetic_name() {
                            phonetic.to_string()
                        } else {
                            String::new()
                        };
                    }
                    SpecificCharacterSet::Iso2022Ir6AndIso2022Ir13AndIso2022Ir87 => {
                        // 例: "YAMADA^TARO=山田^太郎=ﾔﾏﾀﾞ^ﾀﾛｳ"
                        name_alphabet = if let Some(single_byte) = patients_name.single_byte_name()
                        {
                            single_byte.to_string()
                        } else {
                            String::new()
                        };
                        name_kanji = if let Some(ideographic) = patients_name.ideographic_name() {
                            ideographic.to_string()
                        } else {
                            String::new()
                        };
                        name_hiragana = String::new(); // TODO: 半角カタカナをひらがなに変換する処理の実装
                    }
                    SpecificCharacterSet::Iso2022Ir13AndIso2022Ir87 => {
                        // 例: "ﾔﾏﾀﾞ^ﾀﾛｳ=山田^太郎=やまだ^たろう"
                        name_alphabet = String::new();
                        name_kanji = if let Some(ideographic) = patients_name.ideographic_name() {
                            ideographic.to_string()
                        } else {
                            String::new()
                        };
                        name_hiragana = if let Some(phonetic) = patients_name.phonetic_name() {
                            phonetic.to_string()
                        } else {
                            String::new()
                        };
                    }
                    SpecificCharacterSet::IsoIr13 => {
                        // 例: "ﾔﾏﾀﾞ^ﾀﾛｳ"
                        name_alphabet = String::new();
                        name_kanji = String::new();
                        name_hiragana = String::new(); // TODO: 半角カタカナをひらがなに変換する処理の実装
                    }
                }
            } else {
                name_alphabet = String::new();
                name_kanji = String::new();
                name_hiragana = String::new();
            }
        }
        let birth_date = patients_birth_date;
        let sex = match patients_sex {
            Some(sex) => match sex.code() {
                "M" => Some(Sex::M),
                "F" => Some(Sex::F),
                "O" => Some(Sex::O),
                invalid => {
                    return Err((
                        format!("Patient's Sexの値が不正です: {}", invalid),
                        Status::CannotUnderstand(CannotUnderstand::new(0xc000).unwrap()),
                    ));
                }
            },
            None => None,
        };

        Ok(Patient {
            id,
            name_alphabet,
            name_kanji,
            name_hiragana,
            birth_date,
            sex,
        })
    }
}
