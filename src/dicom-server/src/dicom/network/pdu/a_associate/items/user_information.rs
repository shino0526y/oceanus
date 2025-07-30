pub mod sub_items;

use crate::dicom::network::pdu::a_associate::items::{INVALID_ITEM_TYPE_ERROR_MESSAGE, Item};

pub const ITEM_TYPE: u8 = 0x50;

pub struct UserInformation {
    length: u16,
    maximum_length: Option<sub_items::MaximumLength>,
    implementation_class_uid: Option<sub_items::ImplementationClassUid>,
    asynchronous_operations_window: Option<sub_items::AsynchronousOperationsWindow>,
    implementation_version_name: Option<sub_items::ImplementationVersionName>,
}

impl UserInformation {
    pub fn size(&self) -> usize {
        4 + self.length as usize
    }

    pub fn length(&self) -> u16 {
        self.length
    }

    pub fn maximum_length(&self) -> Option<&sub_items::MaximumLength> {
        self.maximum_length.as_ref()
    }

    pub fn implementation_class_uid(&self) -> Option<&sub_items::ImplementationClassUid> {
        self.implementation_class_uid.as_ref()
    }

    pub fn asynchronous_operations_window(
        &self,
    ) -> Option<&sub_items::AsynchronousOperationsWindow> {
        self.asynchronous_operations_window.as_ref()
    }

    pub fn implementation_version_name(&self) -> Option<&sub_items::ImplementationVersionName> {
        self.implementation_version_name.as_ref()
    }
}

impl TryFrom<&[u8]> for UserInformation {
    type Error = String;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let item = Item::try_from(bytes)?;
        if item.item_type != ITEM_TYPE {
            return Err(INVALID_ITEM_TYPE_ERROR_MESSAGE.to_string());
        }

        let mut offset = 0;
        let mut maximum_length = Option::None;
        let mut implementation_class_uid = Option::None;
        let mut asynchronous_operations_window = Option::None;
        let mut implementation_version_name = Option::None;
        while offset < item.data.len() {
            let sub_item_type = item.data[offset];
            match sub_item_type {
                sub_items::maximum_length::ITEM_TYPE => {
                    maximum_length = {
                        let maximum_length = sub_items::MaximumLength::try_from(
                            &item.data[offset..],
                        )
                        .map_err(|message| {
                            format!(
                                "Maximum Length Sub-Item のパースに失敗しました: {}",
                                message
                            )
                        })?;
                        offset += maximum_length.size();
                        Some(maximum_length)
                    }
                }
                sub_items::implementation_class_uid::ITEM_TYPE => {
                    implementation_class_uid = {
                        let implementation_class_uid = sub_items::ImplementationClassUid::try_from(
                            &item.data[offset..],
                        )
                        .map_err(|message| {
                            format!(
                                "Implementation Class UID Sub-Item のパースに失敗しました: {}",
                                message
                            )
                        })?;
                        offset += implementation_class_uid.size();
                        Some(implementation_class_uid)
                    }
                }
                sub_items::asynchronous_operations_window::ITEM_TYPE => {
                    asynchronous_operations_window = {
                        let asynchronous_operations_window =
                            sub_items::AsynchronousOperationsWindow::try_from(&item.data[offset..])
                                .map_err(|message| {
                                    format!(
                                        "Asynchronous Operations Window Sub-Item のパースに失敗しました: {}",
                                        message
                                    )
                                })?;
                        offset += asynchronous_operations_window.size();
                        Some(asynchronous_operations_window)
                    }
                }
                sub_items::implementation_version_name::ITEM_TYPE => {
                    implementation_version_name = {
                        let implementation_version_name =
                            sub_items::ImplementationVersionName::try_from(&item.data[offset..])
                                .map_err(|message| {
                                    format!(
                                        "Implementation Version Name Sub-Item のパースに失敗しました: {}",
                                        message
                                    )
                                })?;
                        offset += implementation_version_name.size();
                        Some(implementation_version_name)
                    }
                }
                _ => {
                    // TODO: Maximum Length Sub-Item以外のSub-Itemのパース
                    //     : 以下は暫定の実装。バイト列をそのまま出力する。
                    println!("未対応の Sub-Item (0x{:02X}):", sub_item_type);
                    let sub_item = Item::try_from(&item.data[offset..])?;
                    for i in 0..sub_item.data.len() {
                        print!("0x{:02X} ", sub_item.data[i]);
                    }
                    println!();
                    offset += 4 + sub_item.data.len();
                }
            }
        }

        Ok(UserInformation {
            length: item.length,
            maximum_length,
            implementation_class_uid,
            asynchronous_operations_window,
            implementation_version_name,
        })
    }
}
