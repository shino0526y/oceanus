use crate::dicom::network::CommandSet;

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
                    let str = std::str::from_utf8(value_field).map_err(|_| "Affected SOP Class UID コマンドの値フィールドを UTF-8 の文字列として解釈できません")?;
                    affected_sop_class_uid = Some(str.trim_end_matches('\0'));
                }
                (0x0000, 0x0100) => {
                    if value_length != 2 {
                        return Err("Command Field コマンドの値長さが不正です");
                    }
                    command_field = Some(u16::from_le_bytes([value_field[0], value_field[1]]));
                }
                (0x0000, 0x0110) => {
                    if value_length != 2 {
                        return Err("Message ID コマンドの値長さが不正です");
                    }
                    message_id = Some(u16::from_le_bytes([value_field[0], value_field[1]]));
                }
                (0x0000, 0x0800) => {
                    if value_length != 2 {
                        return Err("Command Data Set Type コマンドの値長さが不正です");
                    }
                    command_data_set_type =
                        Some(u16::from_le_bytes([value_field[0], value_field[1]]));
                }
                _ => {}
            }
        }

        if affected_sop_class_uid.is_none() {
            return Err("Affected SOP Class UID コマンドが存在しません");
        }
        if affected_sop_class_uid.unwrap() != "1.2.840.10008.1.1" {
            return Err("Unsupported Affected SOP Class UID が不正です");
        }

        if command_field.is_none() {
            return Err("Command Field コマンドが存在しません");
        }
        if command_field.unwrap() != 0x30 {
            return Err("Command Field が不正です");
        }

        if message_id.is_none() {
            return Err("Message ID コマンドが存在しません");
        }
        let message_id = message_id.unwrap();

        if command_data_set_type.is_none() {
            return Err("Command Data Set Type コマンドが存在しません");
        }
        if command_data_set_type.unwrap() != 0x0101 {
            return Err("Command Data Set Type が不正です");
        }

        Ok(CEchoRq { message_id })
    }
}
