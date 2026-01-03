mod constants;
mod element_in_data_set;
mod reader;

use crate::core::{
    DataElement,
    data_element::vr::VrParseError,
    data_set::{constants::ITEM_TAG, element_in_data_set::ElementInDataSet},
    encoding::Encoding,
};
use std::{io::Cursor, ops::Index, vec::IntoIter};

pub struct DataSet {
    pub(super) encoding: Encoding,
    pub(crate) data_elements: Vec<ElementInDataSet>,
    size: usize,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("I/Oエラーが発生しました: {0}")]
    IoError(#[from] std::io::Error),

    #[error("不明なVRが存在します: {0}")]
    UnknownVr(#[from] VrParseError),
}

impl DataSet {
    pub fn encoding(&self) -> Encoding {
        self.encoding
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn get_position(&self, index: usize) -> u64 {
        self.data_elements[index].position
    }

    pub fn get_parent_index(&self, index: usize) -> Option<usize> {
        self.data_elements[index].parent_index
    }

    pub fn get_descendants_count(&self, index: usize) -> usize {
        let mut ret = 0;

        for i in index + 1..self.data_elements.len() {
            if self.data_elements[i].parent_index.is_some()
                && self.data_elements[i].parent_index.unwrap() >= index
            {
                ret += 1;
            } else {
                break;
            }
        }

        ret
    }

    pub fn get_sequence_depth(&self, index: usize) -> usize {
        let mut ret = 0;
        let mut parent_index = self.data_elements[index].parent_index;

        while parent_index.is_some() {
            if self.data_elements[parent_index.unwrap()].element.tag() != ITEM_TAG {
                ret += 1;
            }
            parent_index = self.data_elements[parent_index.unwrap()].parent_index;
        }

        ret
    }

    pub fn len(&self) -> usize {
        self.data_elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data_elements.is_empty()
    }

    pub fn read_from_cur(cur: &mut Cursor<&[u8]>, encoding: Encoding) -> Result<Self, ParseError> {
        let len = cur.get_ref().len() as u64 - cur.position();
        let data_elements = match encoding {
            Encoding::ExplicitVrLittleEndian => {
                reader::read_explicit_vr_le(cur, cur.position(), len, 0)?
            }
            Encoding::ImplicitVrLittleEndian => {
                reader::read_implicit_vr_le(cur, cur.position(), len, 0)?
            }
            Encoding::ExplicitVrBigEndian => {
                unimplemented!("Explicit VR Big Endianの読み込みは未実装です")
            }
        };

        Ok(DataSet {
            encoding,
            data_elements,
            size: len as usize,
        })
    }
}

impl Index<usize> for DataSet {
    type Output = DataElement;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data_elements[index].element
    }
}

impl From<DataSet> for Vec<u8> {
    fn from(v: DataSet) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(v.size);

        for element_in_data_set in v.data_elements {
            bytes.append(&mut element_in_data_set.into());
        }

        bytes
    }
}

impl<'a> IntoIterator for &'a DataSet {
    type Item = &'a DataElement;
    type IntoIter = IntoIter<&'a DataElement>;

    fn into_iter(self) -> Self::IntoIter {
        self.data_elements
            .iter()
            .map(|e| &e.element)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Tag, data_element::Vr};
    use std::io::SeekFrom;
    use tokio::{fs, io::AsyncSeekExt};

    #[tokio::test]
    async fn test_read_from_cur() {
        #[rustfmt::skip]
        let expected = vec![
            (0, 0x00000160, Tag(0x0008, 0x0005), Some(Vr::Cs), 10, 10, 18, None, 0, 0),
            (1, 0x00000172, Tag(0x0008, 0x0016), Some(Vr::Ui), 30, 30, 38, None, 0, 0),
            (2, 0x00000198, Tag(0x0008, 0x0018), Some(Vr::Ui), 56, 56, 64, None, 0, 0),
            (3, 0x000001d8, Tag(0x0008, 0x0020), Some(Vr::Da), 8, 8, 16, None, 0, 0),
            (4, 0x000001e8, Tag(0x0008, 0x0021), Some(Vr::Da), 8, 8, 16, None, 0, 0),
            (5, 0x000001f8, Tag(0x0008, 0x0022), Some(Vr::Da), 8, 8, 16, None, 0, 0),
            (6, 0x00000208, Tag(0x0008, 0x0023), Some(Vr::Da), 8, 8, 16, None, 0, 0),
            (7, 0x00000218, Tag(0x0008, 0x002a), Some(Vr::Dt), 22, 22, 30, None, 0, 0),
            (8, 0x00000236, Tag(0x0008, 0x0030), Some(Vr::Tm), 14, 14, 22, None, 0, 0),
            (9, 0x0000024c, Tag(0x0008, 0x0031), Some(Vr::Tm), 6, 6, 14, None, 0, 0),
            (10, 0x0000025a, Tag(0x0008, 0x0032), Some(Vr::Tm), 14, 14, 22, None, 0, 0),
            (11, 0x00000270, Tag(0x0008, 0x0033), Some(Vr::Tm), 14, 14, 22, None, 0, 0),
            (12, 0x00000286, Tag(0x0008, 0x0050), Some(Vr::Sh), 16, 16, 24, None, 0, 0),
            (13, 0x0000029e, Tag(0x0008, 0x0060), Some(Vr::Cs), 4, 4, 12, None, 0, 0),
            (14, 0x000002aa, Tag(0x0008, 0x0070), Some(Vr::Lo), 18, 18, 26, None, 0, 0),
            (15, 0x000002c4, Tag(0x0008, 0x0080), Some(Vr::Lo), 28, 28, 36, None, 0, 0),
            (16, 0x000002e8, Tag(0x0008, 0x0090), Some(Vr::Pn), 24, 24, 32, None, 0, 0),
            (17, 0x00000308, Tag(0x0008, 0x1010), Some(Vr::Sh), 14, 14, 22, None, 0, 0),
            (18, 0x0000031e, Tag(0x0008, 0x1030), Some(Vr::Lo), 60, 60, 68, None, 0, 0),
            (19, 0x00000362, Tag(0x0008, 0x103e), Some(Vr::Lo), 12, 12, 20, None, 0, 0),
            (20, 0x00000376, Tag(0x0008, 0x1050), Some(Vr::Pn), 26, 26, 34, None, 0, 0),
            (21, 0x00000398, Tag(0x0008, 0x1070), Some(Vr::Pn), 32, 32, 40, None, 0, 0),
            (22, 0x000003c0, Tag(0x0008, 0x1090), Some(Vr::Lo), 20, 20, 28, None, 0, 0),
            (23, 0x000003dc, Tag(0x0008, 0x114a), Some(Vr::Sq), 110, 0, 12, None, 3, 0),
            (24, 0x000003e8, Tag(0xfffe, 0xe000), None, 102, 0, 8, Some(23), 2, 1),
            (25, 0x000003f0, Tag(0x0008, 0x1150), Some(Vr::Ui), 30, 30, 38, Some(24), 0, 1),
            (26, 0x00000416, Tag(0x0008, 0x1155), Some(Vr::Ui), 56, 56, 64, Some(24), 0, 1),
            (27, 0x00000456, Tag(0x0008, 0x1250), Some(Vr::Sq), 214, 0, 12, None, 8, 0),
            (28, 0x00000462, Tag(0xfffe, 0xe000), None, 206, 0, 8, Some(27), 7, 1),
            (29, 0x0000046a, Tag(0x0020, 0x000d), Some(Vr::Ui), 56, 56, 64, Some(28), 0, 1),
            (30, 0x000004aa, Tag(0x0020, 0x000e), Some(Vr::Ui), 56, 56, 64, Some(28), 0, 1),
            (31, 0x000004ea, Tag(0x0040, 0xa170), Some(Vr::Sq), 66, 0, 12, Some(28), 4, 1),
            (32, 0x000004f6, Tag(0xfffe, 0xe000), None, 58, 0, 8, Some(31), 3, 2),
            (33, 0x000004fe, Tag(0x0008, 0x0100), Some(Vr::Sh), 6, 6, 14, Some(32), 0, 2),
            (34, 0x0000050c, Tag(0x0008, 0x0102), Some(Vr::Sh), 4, 4, 12, Some(32), 0, 2),
            (35, 0x00000518, Tag(0x0008, 0x0104), Some(Vr::Lo), 24, 24, 32, Some(32), 0, 2),
            (36, 0x00000538, Tag(0x0010, 0x0010), Some(Vr::Pn), 26, 26, 34, None, 0, 0),
            (37, 0x0000055a, Tag(0x0010, 0x0020), Some(Vr::Lo), 18, 18, 26, None, 0, 0),
            (38, 0x00000574, Tag(0x0010, 0x0030), Some(Vr::Da), 8, 8, 16, None, 0, 0),
            (39, 0x00000584, Tag(0x0010, 0x0040), Some(Vr::Cs), 2, 2, 10, None, 0, 0),
            (40, 0x0000058e, Tag(0x0010, 0x1020), Some(Vr::Ds), 4, 4, 12, None, 0, 0),
            (41, 0x0000059a, Tag(0x0010, 0x1030), Some(Vr::Ds), 4, 4, 12, None, 0, 0),
            (42, 0x000005a6, Tag(0x0010, 0x4000), Some(Vr::Lt), 58, 58, 66, None, 0, 0),
            (43, 0x000005e8, Tag(0x0018, 0x1000), Some(Vr::Lo), 6, 6, 14, None, 0, 0),
            (44, 0x000005f6, Tag(0x0018, 0x1020), Some(Vr::Lo), 16, 16, 24, None, 0, 0),
            (45, 0x0000060e, Tag(0x0018, 0x1030), Some(Vr::Lo), 18, 18, 26, None, 0, 0),
            (46, 0x00000628, Tag(0x0018, 0x106a), Some(Vr::Cs), 10, 10, 18, None, 0, 0),
            (47, 0x0000063a, Tag(0x0018, 0x1800), Some(Vr::Cs), 2, 2, 10, None, 0, 0),
            (48, 0x00000644, Tag(0x0020, 0x000d), Some(Vr::Ui), 56, 56, 64, None, 0, 0),
            (49, 0x00000684, Tag(0x0020, 0x000e), Some(Vr::Ui), 56, 56, 64, None, 0, 0),
            (50, 0x000006c4, Tag(0x0020, 0x0010), Some(Vr::Sh), 16, 16, 24, None, 0, 0),
            (51, 0x000006dc, Tag(0x0020, 0x0011), Some(Vr::Is), 2, 2, 10, None, 0, 0),
            (52, 0x000006e6, Tag(0x0020, 0x0012), Some(Vr::Is), 2, 2, 10, None, 0, 0),
            (53, 0x000006f0, Tag(0x0020, 0x0013), Some(Vr::Is), 2, 2, 10, None, 0, 0),
            (54, 0x000006fa, Tag(0x0020, 0x0200), Some(Vr::Ui), 38, 38, 46, None, 0, 0),
            (55, 0x00000728, Tag(0x0040, 0x0244), Some(Vr::Da), 8, 8, 16, None, 0, 0),
            (56, 0x00000738, Tag(0x0040, 0x0245), Some(Vr::Tm), 14, 14, 22, None, 0, 0),
            (57, 0x0000074e, Tag(0x0040, 0x0555), Some(Vr::Sq), 0, 0, 12, None, 0, 0),
            (58, 0x0000075a, Tag(0x5400, 0x0100), Some(Vr::Sq), 4486, 0, 12, None, 22, 0),
            (59, 0x00000766, Tag(0xfffe, 0xe000), None, 4294967295, 0, 8, Some(58), 21, 1),
            (60, 0x0000076e, Tag(0x0018, 0x1068), Some(Vr::Ds), 2, 2, 10, Some(59), 0, 1),
            (61, 0x00000778, Tag(0x003a, 0x0004), Some(Vr::Cs), 8, 8, 16, Some(59), 0, 1),
            (62, 0x00000788, Tag(0x003a, 0x0005), Some(Vr::Us), 2, 2, 10, Some(59), 0, 1),
            (63, 0x00000792, Tag(0x003a, 0x0010), Some(Vr::Ul), 4, 4, 12, Some(59), 0, 1),
            (64, 0x0000079e, Tag(0x003a, 0x001a), Some(Vr::Ds), 4, 4, 12, Some(59), 0, 1),
            (65, 0x000007aa, Tag(0x003a, 0x0200), Some(Vr::Sq), 148, 0, 12, Some(59), 11, 1),
            (66, 0x000007b6, Tag(0xfffe, 0xe000), None, 140, 0, 8, Some(65), 10, 2),
            (67, 0x000007be, Tag(0x003a, 0x0202), Some(Vr::Is), 2, 2, 10, Some(66), 0, 2),
            (68, 0x000007c8, Tag(0x003a, 0x0203), Some(Vr::Sh), 12, 12, 20, Some(66), 0, 2),
            (69, 0x000007dc, Tag(0x003a, 0x0208), Some(Vr::Sq), 76, 0, 12, Some(66), 5, 2),
            (70, 0x000007e8, Tag(0xfffe, 0xe000), None, 68, 0, 8, Some(69), 4, 3),
            (71, 0x000007f0, Tag(0x0008, 0x0100), Some(Vr::Sh), 10, 10, 18, Some(70), 0, 3),
            (72, 0x00000802, Tag(0x0008, 0x0102), Some(Vr::Sh), 6, 6, 14, Some(70), 0, 3),
            (73, 0x00000810, Tag(0x0008, 0x0103), Some(Vr::Sh), 4, 4, 12, Some(70), 0, 3),
            (74, 0x0000081c, Tag(0x0008, 0x0104), Some(Vr::Lo), 16, 16, 24, Some(70), 0, 3),
            (75, 0x00000834, Tag(0x003a, 0x0214), Some(Vr::Ds), 4, 4, 12, Some(66), 0, 2),
            (76, 0x00000840, Tag(0x003a, 0x021a), Some(Vr::Us), 2, 2, 10, Some(66), 0, 2),
            (77, 0x0000084a, Tag(0x5400, 0x1004), Some(Vr::Us), 2, 2, 10, Some(59), 0, 1),
            (78, 0x00000854, Tag(0x5400, 0x1006), Some(Vr::Cs), 2, 2, 10, Some(59), 0, 1),
            (79, 0x0000085e, Tag(0x5400, 0x1010), Some(Vr::Ob), 4218, 4218, 4230, Some(59), 0, 1),
            (80, 0x000018e4, Tag(0xfffe, 0xe00d), None, 0, 0, 8, Some(59), 0, 1),
        ];

        let data_set = {
            let buf = fs::read("../../data/dicom/GENECG").await.unwrap();
            let mut cur = Cursor::new(buf.as_ref());
            cur.seek(SeekFrom::Current(0x00000160)).await.unwrap();
            DataSet::read_from_cur(&mut cur, Encoding::ExplicitVrLittleEndian).unwrap()
        };
        let actual = {
            data_set
                .into_iter()
                .enumerate()
                .map(|(i, e)| {
                    (
                        i,
                        data_set.get_position(i),
                        e.tag(),
                        e.vr(),
                        e.value_length(),
                        e.value_field().len(),
                        e.size(),
                        data_set.get_parent_index(i),
                        data_set.get_descendants_count(i),
                        data_set.get_sequence_depth(i),
                    )
                })
                .collect::<Vec<_>>()
        };

        assert_eq!(expected, actual);
    }
}
