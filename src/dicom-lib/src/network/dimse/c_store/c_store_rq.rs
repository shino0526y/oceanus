use crate::network::{CommandSet, dimse::enums::Priority};
use std::str::from_utf8;

#[derive(Debug, PartialEq, Eq)]
pub struct CStoreRq {
    affected_sop_class_uid: String,
    message_id: u16,
    priority: Priority,
    affected_sop_instance_uid: String,
    move_originator_ae_title: Option<String>,
    move_originator_message_id: Option<u16>,
}

impl CStoreRq {
    pub fn affected_sop_class_uid(&self) -> &str {
        &self.affected_sop_class_uid
    }

    pub fn message_id(&self) -> u16 {
        self.message_id
    }

    pub fn priority(&self) -> Priority {
        self.priority
    }

    pub fn affected_sop_instance_uid(&self) -> &str {
        &self.affected_sop_instance_uid
    }

    pub fn move_originator_ae_title(&self) -> Option<&str> {
        self.move_originator_ae_title.as_deref()
    }

    pub fn move_originator_message_id(&self) -> Option<u16> {
        self.move_originator_message_id
    }
}

impl TryFrom<CommandSet> for CStoreRq {
    type Error = String;

    fn try_from(val: CommandSet) -> Result<Self, Self::Error> {
        let mut affected_sop_class_uid = None;
        let mut command_field = None;
        let mut message_id = None;
        let mut priority = None;
        let mut command_data_set_type = None;
        let mut affected_sop_instance_uid = None;
        let mut move_originator_ae_title = None;
        let mut move_originator_message_id = None;

        for command in val.iter() {
            let tag = command.tag();
            let value_length = command.value_length();
            let value_field = command.value_field();
            match (tag.group(), tag.element()) {
                (0x0000, 0x0002) => {
                    let uid = from_utf8(value_field).map_err(|_| "Affected SOP Class UIDコマンドの値フィールドをUTF-8の文字列として解釈できません")?.trim_end_matches('\0');
                    if !uid.starts_with(
                        "1.2.840.10008.5.1.4.1.1.", // Storage関連のSOP Class UIDは1.2.840.10008.5.1.4.1.1.*の形式
                    ) {
                        return Err("Affected SOP Class UIDが不正です".to_string());
                    }
                    affected_sop_class_uid = Some(uid.to_string());
                }
                (0x0000, 0x0100) => {
                    if value_length != 2 {
                        return Err("Command Fieldコマンドの値長さが不正です".to_string());
                    }
                    let f = u16::from_le_bytes([value_field[0], value_field[1]]);
                    if f != 0x0001 {
                        return Err("Command Fieldが不正です".to_string());
                    }
                    command_field = Some(f);
                }
                (0x0000, 0x0110) => {
                    if value_length != 2 {
                        return Err("Message IDコマンドの値長さが不正です".to_string());
                    }
                    message_id = Some(u16::from_le_bytes([value_field[0], value_field[1]]));
                }
                (0x0000, 0x0700) => {
                    if value_length != 2 {
                        return Err("Priorityコマンドの値長さが不正です".to_string());
                    }
                    priority = Some(
                        Priority::try_from(u16::from_le_bytes([value_field[0], value_field[1]]))
                            .map_err(|e| format!("Priorityが不正です: {e}"))?,
                    );
                }
                (0x0000, 0x0800) => {
                    if value_length != 2 {
                        return Err("Command Data Set Typeコマンドの値長さが不正です".to_string());
                    }
                    let t = u16::from_le_bytes([value_field[0], value_field[1]]);
                    if t == 0x0101 {
                        return Err("Command Data Set Typeが不正です".to_string());
                    }
                    command_data_set_type = Some(t);
                }
                (0x0000, 0x1000) => {
                    let uid = from_utf8(value_field).map_err(|_| "Affected SOP Instance UIDコマンドの値フィールドをUTF-8の文字列として解釈できません")?.trim_end_matches('\0');
                    if uid.is_empty() {
                        return Err("Affected SOP Instance UIDが空です".to_string());
                    }
                    affected_sop_instance_uid = Some(uid.to_string());
                }
                (0x0000, 0x1030) => {
                    let title = from_utf8(value_field).map_err(|_| "Move Originator Application Entity Titleコマンドの値フィールドをUTF-8の文字列として解釈できません")?.trim_end_matches(' ').trim_start_matches(' ');
                    if title.is_empty() || title.len() > 16 {
                        return Err(
                            "Move Originator Application Entity Titleは1文字以上16文字以下でなければなりません".to_string(),
                        );
                    }
                    if !title.is_ascii() {
                        return Err(
                            "Move Originator Application Entity TitleはISO 646:1990 (basic G0 set)でエンコーディングされている必要があります".to_string(),
                        );
                    }
                    move_originator_ae_title = Some(title.to_string());
                }
                (0x0000, 0x1031) => {
                    if value_length != 2 {
                        return Err(
                            "Move Originator Message IDコマンドの値長さが不正です".to_string()
                        );
                    }
                    move_originator_message_id =
                        Some(u16::from_le_bytes([value_field[0], value_field[1]]));
                }
                _ => {}
            }
        }

        if affected_sop_class_uid.is_none() {
            return Err("Affected SOP Class UIDコマンドが存在しません".to_string());
        }
        if command_field.is_none() {
            return Err("Command Fieldコマンドが存在しません".to_string());
        }
        if message_id.is_none() {
            return Err("Message IDコマンドが存在しません".to_string());
        }
        if priority.is_none() {
            return Err("Priorityコマンドが存在しません".to_string());
        }
        if command_data_set_type.is_none() {
            return Err("Command Data Set Typeコマンドが存在しません".to_string());
        }
        if affected_sop_instance_uid.is_none() {
            return Err("Affected SOP Instance UIDコマンドが存在しません".to_string());
        }

        Ok(CStoreRq {
            affected_sop_class_uid: affected_sop_class_uid.unwrap(),
            message_id: message_id.unwrap(),
            priority: priority.unwrap(),
            affected_sop_instance_uid: affected_sop_instance_uid.unwrap(),
            move_originator_ae_title,
            move_originator_message_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{core::Tag, network::command_set::Command};

    #[test]
    fn test_c_store_rq_try_from() {
        let expected = CStoreRq {
            affected_sop_class_uid: "1.2.840.10008.5.1.4.1.1.4".to_string(),
            message_id: 1,
            priority: Priority::Medium,
            affected_sop_instance_uid: "41.2.392.200036.8120.100.20041012.1123100.2001002010"
                .to_string(),
            move_originator_ae_title: None,
            move_originator_message_id: None,
        };

        let actual = {
            let command_set = CommandSet::new(vec![
                Command::new(Tag(0x0000, 0x0000), 134u32.to_le_bytes().to_vec()),
                Command::new(
                    Tag(0x0000, 0x0002),
                    "1.2.840.10008.5.1.4.1.1.4\0".as_bytes().to_vec(),
                ),
                Command::new(Tag(0x0000, 0x0100), 0x0001u16.to_le_bytes().to_vec()),
                Command::new(Tag(0x0000, 0x0110), 1u16.to_le_bytes().to_vec()),
                Command::new(Tag(0x0000, 0x0700), 0u16.to_le_bytes().to_vec()),
                Command::new(Tag(0x0000, 0x0800), 1u16.to_le_bytes().to_vec()),
                Command::new(
                    Tag(0x0000, 0x1000),
                    "41.2.392.200036.8120.100.20041012.1123100.2001002010"
                        .as_bytes()
                        .to_vec(),
                ),
            ])
            .unwrap();

            CStoreRq::try_from(command_set).unwrap()
        };

        assert_eq!(expected, actual);
    }
}
