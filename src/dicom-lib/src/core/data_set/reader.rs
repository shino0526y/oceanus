use crate::{
    core::{
        DataElement, Tag,
        data_set::{
            constants::{
                ITEM_DELIMITATION_TAG, ITEM_TAG, PIXEL_DATA_TAG, SEQUENCE_DELIMITATION_TAG,
            },
            element_in_data_set::ElementInDataSet,
        },
        encoding::Encoding,
    },
    dictionaries::tag_dictionary,
};
use std::io::{Cursor, Error, Read, Seek, SeekFrom};

pub fn read_explicit_vr_le(
    cur: &mut Cursor<&[u8]>,
    position: u64,
    length: u64,
    index_base: usize,
) -> Result<Vec<ElementInDataSet>, Error> {
    cur.seek(SeekFrom::Start(position))?;

    let mut elements = vec![];
    let mut current_position = position;
    let end_position = position + length;
    while current_position < end_position {
        let element = read_element_explicit_vr_le(cur)?;
        let tag = element.tag();
        let value_length = element.value_length();
        let is_un = element.vr() == Some("UN");
        elements.push(element);

        current_position = cur.position();

        // データ要素がカプセル化されたPixel Dataである場合、もしくは値長さが不定かつ"UN"のVRを持つデータ要素である場合、
        // そのデータ要素の子孫要素を通常とは異なる方法で読み取る必要がある。
        if value_length == 0xffffffff {
            if tag == PIXEL_DATA_TAG {
                // データ要素が値長さが0xffffffffのPixel Dataである場合、すなわちカプセル化されたPixel Dataである場合、それはシーケンスとみなされる。
                // このとき、アイテム要素がシーケンスの子要素として存在し、そのアイテム要素の値には画像のピクセルデータが記録される。
                // Pixel Dataに対応するシーケンス区切り要素が出現した時点でこのループを抜ける。
                while current_position < end_position {
                    let child_element_in_encapsulated_pixel_data =
                        read_child_element_in_encapsulated_pixel_data_explicit_vr_le(cur)?;
                    let tag = child_element_in_encapsulated_pixel_data.tag();
                    elements.push(child_element_in_encapsulated_pixel_data);

                    current_position = cur.position();

                    if tag == SEQUENCE_DELIMITATION_TAG {
                        break;
                    }
                }
            } else if is_un {
                // データ要素が、値長さが0xffffffffかつ"UN"のVRを持っている場合、それはシーケンスとみなされ、その子孫要素を暗黙的値表現リトルエンディアンとして読み取る必要がある。
                // この際、子孫要素として値長さが0xffffffffであるシーケンス要素が含まれていることを考慮する。
                //
                // なお、厳密には規格違反と思われるが、明示的値表現リトルエンディアンとしてデータ要素が保存されている可能性もある。
                // そのため、暗黙的値表現リトルエンディアンとして読み取りを試み、整合性が取れない場合は明示的値表現リトルエンディアンとして読み取る。

                let mut sequence_count = 1;
                let position_before_reading = current_position;
                let mut descendant_elements = vec![];

                while current_position < end_position {
                    let descendant_element_in_sequence = match read_element_implicit_vr_le(cur) {
                        Ok(element) => element,
                        Err(_) => {
                            // Errが戻されたということは、バッファーからデータ要素を暗黙的値表現リトルエンディアンで読み取ろうとしたものの、不整合があったということを意味する。
                            // この後の処理で明示的値表現リトルエンディアンとして再度読み取りを試みるため、シーケンスカウントに無効な値である-1をセットし、ループを抜ける。
                            sequence_count = -1;
                            break;
                        }
                    };

                    let tag = descendant_element_in_sequence.tag();
                    let value_length = descendant_element_in_sequence.value_length();
                    let is_un_or_sq = descendant_element_in_sequence.vr() == Some("UN")
                        || descendant_element_in_sequence.vr() == Some("SQ");
                    descendant_elements.push(descendant_element_in_sequence);

                    if value_length == 0xffffffff && is_un_or_sq {
                        sequence_count += 1;
                    } else if tag == SEQUENCE_DELIMITATION_TAG {
                        sequence_count -= 1;
                        if sequence_count == 0 {
                            break;
                        }
                    }

                    current_position = cur.position();
                }

                if sequence_count == 0 {
                    // シーケンスカウントが0であるということは、暗黙的値表現リトルエンディアンとしてデータを読み取った結果に不整合がなかったということ。
                    elements.append(&mut descendant_elements);
                    continue;
                }

                // ここに到達したということは、シーケンスカウントが0でなかった、すなわち、暗黙的値表現リトルエンディアンとして読み取ったデータに不整合があったということを意味する。
                // 読み取った結果をリセットし、明示的値表現リトルエンディアンとして読み取りを試みる。
                descendant_elements.clear();
                cur.seek(SeekFrom::Start(position_before_reading))?;

                current_position = position_before_reading;
                sequence_count = 1;
                while current_position < end_position {
                    let descendant_element_in_sequence = read_element_explicit_vr_le(cur)?;
                    let tag = descendant_element_in_sequence.tag();
                    let value_length = descendant_element_in_sequence.value_length();
                    let is_un_or_sq = descendant_element_in_sequence.vr() == Some("UN")
                        || descendant_element_in_sequence.vr() == Some("SQ");
                    descendant_elements.push(descendant_element_in_sequence);

                    current_position = cur.position();

                    if value_length == 0xffffffff && is_un_or_sq {
                        sequence_count += 1;
                    } else if tag == SEQUENCE_DELIMITATION_TAG {
                        sequence_count -= 1;
                        if sequence_count == 0 {
                            break;
                        }
                    }
                }

                elements.append(&mut descendant_elements);
            }
        }
    }

    let elements_length = elements.len();
    update_parent_index_for_each_element(&mut elements, 0, elements_length, index_base, None);

    Ok(elements)
}

pub fn read_implicit_vr_le(
    cur: &mut Cursor<&[u8]>,
    position: u64,
    length: u64,
    index_base: usize,
) -> Result<Vec<ElementInDataSet>, Error> {
    cur.seek(SeekFrom::Start(position))?;

    let mut elements = vec![];
    let mut current_position = position;
    let end_position = position + length;
    while current_position < end_position {
        let element = read_element_implicit_vr_le(cur)?;
        elements.push(element);

        current_position = cur.position();
    }

    let elements_length = elements.len();
    update_parent_index_for_each_element(&mut elements, 0, elements_length, index_base, None);

    Ok(elements)
}

fn read_element_implicit_vr_le(cur: &mut Cursor<&[u8]>) -> Result<ElementInDataSet, Error> {
    let position = cur.position();
    let tag = read_tag(cur)?;
    let value_length = read_value_length_implicit_vr_le(cur)?;
    let value_field = read_value_field(cur, tag, None, value_length)?;
    let encoding = Encoding::ImplicitVrLittleEndian;
    let size = cur.position() - position;

    let element = ElementInDataSet {
        element: DataElement::new(tag, None, value_length, value_field, encoding, size),
        position,
        parent_index: None, // 現時点では意味のない値
    };
    Ok(element)
}

fn read_element_explicit_vr_le(cur: &mut Cursor<&[u8]>) -> Result<ElementInDataSet, Error> {
    let position = cur.position();
    let tag = read_tag(cur)?;
    let vr = read_vr(cur, tag)?;
    let value_length = read_value_length_explicit_vr_le(cur, tag, &vr)?;
    let value_field = read_value_field(cur, tag, Some(&vr), value_length)?;
    let encoding = Encoding::ExplicitVrLittleEndian;
    let size = cur.position() - position;

    let element = ElementInDataSet {
        element: DataElement::new(tag, Some(vr), value_length, value_field, encoding, size),
        position,
        parent_index: None, // 現時点では意味のない値
    };
    Ok(element)
}

fn read_child_element_in_encapsulated_pixel_data_explicit_vr_le(
    cur: &mut Cursor<&[u8]>,
) -> Result<ElementInDataSet, Error> {
    let position = cur.position();
    let tag = read_tag(cur)?;
    let vr = "".to_string();
    let value_length = {
        let mut buf = [0; 4];
        cur.read_exact(&mut buf)?;
        u32::from_le_bytes(buf)
    };
    let value_field = {
        let mut buf = vec![0; value_length as usize];
        cur.read_exact(&mut buf)?;
        buf
    };
    let encoding = Encoding::ExplicitVrLittleEndian;
    let size = cur.position() - position;

    let element = ElementInDataSet {
        element: DataElement::new(tag, Some(vr), value_length, value_field, encoding, size),
        position,
        parent_index: None, // 現時点では意味のない値
    };
    Ok(element)
}

fn read_tag(cur: &mut Cursor<&[u8]>) -> Result<Tag, Error> {
    let mut tag_group_buf = [0; 2];
    cur.read_exact(&mut tag_group_buf)?;

    let mut tag_element_buf = [0; 2];
    cur.read_exact(&mut tag_element_buf)?;

    let tag_group = u16::from_le_bytes(tag_group_buf);
    let tag_element = u16::from_le_bytes(tag_element_buf);
    Ok(Tag(tag_group, tag_element))
}

fn read_vr(cur: &mut Cursor<&[u8]>, tag: Tag) -> Result<String, Error> {
    match tag {
        ITEM_TAG | ITEM_DELIMITATION_TAG | SEQUENCE_DELIMITATION_TAG => Ok("".to_string()),
        _ => {
            let mut buf = [0; 2];
            cur.read_exact(&mut buf)?;
            let vr_string = String::from_utf8_lossy(&buf);
            Ok(vr_string.to_string())
        }
    }
}

fn read_value_length_implicit_vr_le(cur: &mut Cursor<&[u8]>) -> Result<u32, Error> {
    let mut buf = [0; 4];
    cur.read_exact(&mut buf)?;

    let value_length = u32::from_le_bytes(buf);
    Ok(value_length)
}

fn read_value_length_explicit_vr_le(
    cur: &mut Cursor<&[u8]>,
    tag: Tag,
    vr: &str,
) -> Result<u32, Error> {
    match tag {
        ITEM_TAG | ITEM_DELIMITATION_TAG | SEQUENCE_DELIMITATION_TAG => {
            let mut buf = [0; 4];
            cur.read_exact(&mut buf)?;

            let value_length = u32::from_le_bytes(buf);
            Ok(value_length)
        }
        _ => {
            match vr {
                "AE" | "AS" | "AT" | "CS" | "DA" | "DS" | "DT" | "FD" | "FL" | "IS" | "LO"
                | "LT" | "PN" | "SH" | "SL" | "SS" | "ST" | "TM" | "UI" | "UL" | "US" => {
                    // 上記のVRである場合、読み取るサイズは2バイト。
                    let mut buf = [0; 2];
                    cur.read_exact(&mut buf)?;

                    let value_length = u16::from_le_bytes(buf) as u32;
                    Ok(value_length)
                }
                "OB" | "OD" | "OF" | "OL" | "OV" | "OW" | "SQ" | "SV" | "UC" | "UN" | "UR"
                | "UT" | "UV" => {
                    // 上記のVRである場合、読み取るサイズは4バイト。
                    // なお、ストリームには2バイトの予約済み領域が含まれるため、読み取り位置を2バイトずらす。
                    cur.seek_relative(2)?;
                    let mut buf = [0; 4];
                    cur.read_exact(&mut buf)?;

                    let value_length = u32::from_le_bytes(buf);
                    Ok(value_length)
                }
                _ => {
                    // タグ辞書からVRを取得する。
                    // VRを取得できない場合（プライベートもしくは不明なタグ）、そのVRを"UN"とみなす。
                    let estimated_vr = match tag_dictionary::search(tag) {
                        Some(item) => item.vr,
                        None => "UN",
                    };
                    // 取得されたVRは"SS or US"といった複数のVRの文字列である可能性がある。
                    // そのため、VR文字列の先頭2文字を切り出し、それをVRとして採用する。
                    // 採用されたVRをこのメソッド自身に渡し、再帰呼び出しした結果を返す。
                    read_value_length_explicit_vr_le(cur, tag, &estimated_vr[0..2])
                }
            }
        }
    }
}

fn read_value_field(
    cur: &mut Cursor<&[u8]>,
    tag: Tag,
    vr: Option<&str>,
    value_length: u32,
) -> Result<Vec<u8>, Error> {
    if tag == ITEM_TAG || tag == ITEM_DELIMITATION_TAG || tag == SEQUENCE_DELIMITATION_TAG {
        return Ok(vec![]);
    }

    let vr = match vr {
        Some(vr) => vr,
        None => tag_dictionary::search(tag).map_or("UN", |item| &item.vr[0..2]),
    };

    if vr == "SQ" || value_length == 0xffffffff {
        Ok(vec![])
    } else {
        // 上記の条件に合致しなかった場合、データ要素は何かしらの値フィールドを持つ。
        let mut buf = vec![0; value_length as usize];
        cur.read_exact(&mut buf)?;
        Ok(buf)
    }
}

fn update_parent_index_for_each_element(
    elements: &mut Vec<ElementInDataSet>,
    index: usize,
    length: usize,
    index_base: usize,
    top_parent_index: Option<usize>,
) {
    let mut i: usize = index;
    while i < index + length {
        elements[i].parent_index = top_parent_index;

        let descendants = calculate_descendants_count(elements, i);
        if descendants != 0 {
            update_parent_index_for_each_element(
                elements,
                i + 1,
                descendants,
                index_base,
                Some(i + index_base),
            );
            i += descendants;
        }

        i += 1;
    }
}

fn calculate_descendants_count(elements: &[ElementInDataSet], index: usize) -> usize {
    if !elements[index].value_field().is_empty() {
        // 子孫要素の有無に関わらず、親要素自身は値フィールドを持たない。
        // そのため、値フィールドの長さが0でないデータ要素の子孫は存在しないことになる。
        return 0;
    } else if elements[index].vr() != Some("SQ")
        && elements[index].tag() != ITEM_TAG
        && elements[index].value_length() != 0xffffffff
    {
        // シーケンス要素 / アイテム要素 / カプセル化されたPixel Data要素 / 不定な値長さを持つVRが"UN"のデータ要素 は子孫要素を持つ可能性がある。
        // その条件に合致しない場合は、そのデータ要素の子孫が存在しないことになる。
        // なお、カプセル化されたPixel Data要素の値長さは0xffffffffであるため、不定な値長さを持つVRが"UN"のデータ要素と同じ条件で判定を行っている。
        return 0;
    } else if elements[index].value_length() == 0 {
        // データ要素の値長さが0の場合は子孫の個数は0となる。
        return 0;
    } else if index + 1 == elements.len() {
        // インデックスがvectorの末尾を指し示す場合、それ以降にデータ要素が存在しないため、子孫要素も存在しない。
        return 0;
    }

    if elements[index].value_length() == 0xffffffff {
        // データ要素の値長さが0xffffffffである場合、そのデータ要素は子孫を持つ。
        // ここでは、データ要素がアイテム要素である場合と、シーケンス要素（もしくはそれと同等であるとみなせる要素）である場合で分岐する。

        if elements[index].tag() == ITEM_TAG {
            // データ要素が値長さが0xffffffffのアイテム要素である場合、それに対応するアイテム区切り要素が出現するまでのデータ要素の個数をカウントする。
            // このとき、カウントされたデータ要素の個数が子孫要素の個数である。
            // アイテム要素の子孫として値長さが0xffffffffのアイテム要素が入れ子で存在する場合は、それに対応するアイテム区切り要素も存在することを考慮する。
            let mut descendants_count = 0;
            let mut delimitation_count = 1;
            for element in elements.iter().skip(index + 1) {
                descendants_count += 1;

                if element.tag() == ITEM_DELIMITATION_TAG {
                    delimitation_count -= 1;
                    if delimitation_count == 0 {
                        break;
                    }
                } else if element.value_length() == 0xffffffff && element.tag() == ITEM_TAG {
                    delimitation_count += 1;
                }
            }
            descendants_count
        } else {
            // データ要素が値長さが0xffffffffのシーケンス要素、もしくはそれと同等であるとみなせる要素
            // （値長さが0xffffffffであり、UNのVRを持つ要素もしくはカプセル化されたPixel Data要素）である場合、
            // それに対応するシーケンス区切り要素が出現するまでのデータ要素の個数をカウントする。
            // このとき、カウントされたデータ要素の個数が子孫要素の個数である。
            // シーケンス内に値長さが0xffffffffのシーケンス要素、もしくはそれと同等であるとみなせる要素が入れ子で存在する場合は、
            // それに対応するシーケンス区切り要素が出現することを考慮する。
            let mut descendants_count = 0;
            let mut delimitation_count = 1;
            for element in elements.iter().skip(index + 1) {
                descendants_count += 1;

                if element.tag() == SEQUENCE_DELIMITATION_TAG {
                    delimitation_count -= 1;
                    if delimitation_count == 0 {
                        break;
                    }
                } else if element.value_length() == 0xffffffff && element.tag() != ITEM_TAG {
                    delimitation_count += 1;
                }
            }
            descendants_count
        }
    } else {
        // データ要素の値長さが不明でない場合、子孫要素の合計サイズがこの値と同じになる。
        // そのため、値長さの値を、出現するデータ要素のサイズで減算しつつ、データ要素の出現回数をカウントし、
        // 減算の結果値が0以下になった場合に処理を抜ける。
        // このとき、カウントされたデータ要素の個数が子孫要素の個数である。
        let mut descendants_count = 0;
        let mut size = elements[index].value_length() as i32;
        for element in elements.iter().skip(index + 1) {
            descendants_count += 1;

            size -= element.size() as i32;
            if size <= 0 {
                break;
            }
        }
        descendants_count
    }
}
