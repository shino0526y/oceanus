pub mod file_meta_information;

use crate::{core::DataSet, file::file_meta_information::FileMetaInformation};

pub struct File {
    meta_information: FileMetaInformation,
    data_set: DataSet,
}

impl File {
    pub fn meta_information(&self) -> &FileMetaInformation {
        &self.meta_information
    }

    pub fn data_set(&self) -> &DataSet {
        &self.data_set
    }

    pub fn new(meta_information: FileMetaInformation, mut data_set: DataSet) -> Self {
        // データセット内の各データ要素の位置情報を更新
        let mut position = 128 // Preamble
                            + 4 // Prefix
                            + meta_information.size() as u64;
        data_set.data_elements.iter_mut().for_each(|e| {
            e.position = position;
            position += e.size() as u64;
        });

        Self {
            meta_information,
            data_set,
        }
    }

    pub fn size(&self) -> usize {
        128 // Preamble
        + 4 // Prefix
        + self.meta_information.size()
        + self.data_set.size()
    }
}

impl Into<Vec<u8>> for File {
    fn into(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.size());
        buf.extend_from_slice(&[0u8; 128]); // Preamble
        buf.extend_from_slice(b"DICM"); // Prefix
        buf.append(&mut self.meta_information.into()); // File Meta Information
        buf.append(&mut self.data_set.into()); // Data Set
        buf
    }
}
