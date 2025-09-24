use crate::{constants::sop_class_uids::VERIFICATION, network::CommandSet};

pub struct CEchoRq {
    message_id: u16,
}

impl CEchoRq {
    pub fn message_id(&self) -> u16 {
        self.message_id
    }
}

impl TryFrom<CommandSet> for CEchoRq {
    type Error = &'static str;

    fn try_from(val: CommandSet) -> Result<Self, Self::Error> {
        let mut affected_sop_class_uid = None;
        let mut command_field = None;
        let mut message_id = None;
        let mut command_data_set_type = None;

        for command in val.iter() {
            let tag = command.tag();
            let value_length = command.value_length();
            let value_field = command.value_field();
            match (tag.group(), tag.element()) {
                (0x0000, 0x0002) => {
                    let str = std::str::from_utf8(value_field).map_err(|_| "Affected SOP Class UIDコマンドの値フィールドをUTF-8の文字列として解釈できません")?;
                    affected_sop_class_uid = Some(str.trim_end_matches('\0'));
                }
                (0x0000, 0x0100) => {
                    if value_length != 2 {
                        return Err("Command Fieldコマンドの値長さが不正です");
                    }
                    command_field = Some(u16::from_le_bytes([value_field[0], value_field[1]]));
                }
                (0x0000, 0x0110) => {
                    if value_length != 2 {
                        return Err("Message IDコマンドの値長さが不正です");
                    }
                    message_id = Some(u16::from_le_bytes([value_field[0], value_field[1]]));
                }
                (0x0000, 0x0800) => {
                    if value_length != 2 {
                        return Err("Command Data Set Typeコマンドの値長さが不正です");
                    }
                    command_data_set_type =
                        Some(u16::from_le_bytes([value_field[0], value_field[1]]));
                }
                _ => {}
            }
        }

        if affected_sop_class_uid.is_none() {
            return Err("Affected SOP Class UIDコマンドが存在しません");
        }
        if affected_sop_class_uid.unwrap() != VERIFICATION {
            return Err("Affected SOP Class UIDが不正です");
        }

        if command_field.is_none() {
            return Err("Command Fieldコマンドが存在しません");
        }
        if command_field.unwrap() != 0x30 {
            return Err("Command Fieldが不正です");
        }

        if message_id.is_none() {
            return Err("Message IDコマンドが存在しません");
        }
        let message_id = message_id.unwrap();

        if command_data_set_type.is_none() {
            return Err("Command Data Set Typeコマンドが存在しません");
        }
        if command_data_set_type.unwrap() != 0x0101 {
            return Err("Command Data Set Typeが不正です");
        }

        Ok(CEchoRq { message_id })
    }
}
