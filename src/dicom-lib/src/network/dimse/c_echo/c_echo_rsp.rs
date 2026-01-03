use crate::{
    constants::sop_class_uids::VERIFICATION,
    core::Tag,
    network::{CommandSet, command_set::Command},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// 成功 ... 操作が成功したことを示す
    Success = 0x0000,
    /// 拒否：未対応のSOP Class ... Verification SOP Classとは異なるSOP Classが指定され、それがサポートされていないことを示す
    Refused = 0x0122,
    /// 重複呼び出し ... 指定されたメッセージIDが別の通知もしくは操作に割り当てられていることを示す
    DuplicateInvocation = 0x0210,
    /// 引数の型が不正 ... 指定されたパラメータの1つが、DIMSEサービスユーザ間のアソシエーションでの使用が合意されていないことを示す
    MistypedArgument = 0x0212,
    /// 認識されていない操作 ... Verification SOP Classとは異なるSOP Classが指定され、そのSOP ClassがC-ECHO操作を認識しないことを示す
    UnrecognizedOperation = 0x0211,
}

pub struct CEchoRsp {
    message_id: u16,
    status: Status,
}

impl CEchoRsp {
    pub fn message_id(&self) -> u16 {
        self.message_id
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn new(message_id: u16, status: Status) -> Self {
        Self { message_id, status }
    }
}

impl From<CEchoRsp> for CommandSet {
    fn from(val: CEchoRsp) -> Self {
        let affected_sop_class_uid = Command {
            tag: Tag(0x0000, 0x0002),
            value_field: format!("{}{}", VERIFICATION, '\0').into_bytes(),
        };
        let command_field = Command {
            tag: Tag(0x0000, 0x0100),
            value_field: 0x8030u16.to_le_bytes().to_vec(),
        };
        let message_id_being_responded_to = Command {
            tag: Tag(0x0000, 0x0120),
            value_field: val.message_id.to_le_bytes().to_vec(),
        };
        let command_data_set_type = Command {
            tag: Tag(0x0000, 0x0800),
            value_field: 0x0101u16.to_le_bytes().to_vec(),
        };
        let status = Command {
            tag: Tag(0x0000, 0x0900),
            value_field: (val.status as u16).to_le_bytes().to_vec(),
        };
        let group_length = affected_sop_class_uid.size()
            + command_field.size()
            + message_id_being_responded_to.size()
            + command_data_set_type.size()
            + status.size();
        let command_group_length = Command {
            tag: Tag(0x0000, 0x0000),
            value_field: (group_length as u32).to_le_bytes().to_vec(),
        };
        let size = group_length + command_group_length.size();

        CommandSet {
            size,
            commands: vec![
                command_group_length,
                affected_sop_class_uid,
                command_field,
                message_id_being_responded_to,
                command_data_set_type,
                status,
            ],
        }
    }
}
