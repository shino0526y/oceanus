mod status;

pub use status::Status;

use crate::{
    core::Tag,
    network::{CommandSet, command_set::Command},
};

pub struct CStoreRsp {
    message_id: u16,
    status: Status,
    affected_sop_class_uid: String,
    affected_sop_instance_uid: String,
}

impl CStoreRsp {
    pub fn message_id(&self) -> u16 {
        self.message_id
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn affected_sop_class_uid(&self) -> &str {
        &self.affected_sop_class_uid
    }

    pub fn affected_sop_instance_uid(&self) -> &str {
        &self.affected_sop_instance_uid
    }

    pub fn new(
        message_id: u16,
        status: Status,
        affected_sop_class_uid: impl Into<String>,
        affected_sop_instance_uid: impl Into<String>,
    ) -> Self {
        Self {
            message_id,
            status,
            affected_sop_class_uid: affected_sop_class_uid.into(),
            affected_sop_instance_uid: affected_sop_instance_uid.into(),
        }
    }
}

impl From<CStoreRsp> for CommandSet {
    fn from(val: CStoreRsp) -> Self {
        let affected_sop_class_uid = {
            let mut uid = val.affected_sop_class_uid;
            if uid.len() % 2 != 0 {
                uid.push('\0');
            };
            Command {
                tag: Tag(0x0000, 0x0002),
                value_field: uid.into_bytes(),
            }
        };
        let command_field = Command {
            tag: Tag(0x0000, 0x0100),
            value_field: 0x8001u16.to_le_bytes().to_vec(),
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
            value_field: Into::<u16>::into(val.status).to_le_bytes().to_vec(),
        };
        let affected_sop_instance_uid = {
            let mut uid = val.affected_sop_instance_uid;
            if uid.len() % 2 != 0 {
                uid.push('\0');
            };
            Command {
                tag: Tag(0x0000, 0x1000),
                value_field: uid.into_bytes(),
            }
        };
        let group_length = affected_sop_class_uid.size()
            + command_field.size()
            + message_id_being_responded_to.size()
            + command_data_set_type.size()
            + status.size()
            + affected_sop_instance_uid.size();
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
