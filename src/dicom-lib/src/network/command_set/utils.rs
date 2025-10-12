use crate::network::upper_layer_protocol::pdu::{PDataTf, p_data_tf::PresentationDataValue};

pub fn generate_p_data_tf_pdus(
    context_id: u8,
    command_set_buf: Vec<u8>,
    data_set_buf: Vec<u8>,
    maximum_length: u32,
) -> Vec<PDataTf> {
    if maximum_length == 0 {
        let pdvs = vec![PresentationDataValue::new(
            context_id,
            true,
            true,
            command_set_buf,
        )];

        if !data_set_buf.is_empty() {
            unimplemented!("データセットの変換処理は未実装");
        }

        vec![PDataTf::new(pdvs)]
    } else {
        let max_chunk_size = maximum_length as usize - 6; // Presentation Data Value Itemの実データ以外のサイズが6バイト
        let mut p_data_tf_pdus = vec![];

        {
            let mut offset = 0;
            while offset < command_set_buf.len() {
                let chunk = &command_set_buf
                    [offset..offset + max_chunk_size.min(command_set_buf.len() - offset)];
                offset += chunk.len();
                let is_last = offset >= command_set_buf.len();
                let pdv = PresentationDataValue::new(context_id, true, is_last, chunk);

                let p_data_tf = PDataTf::new(vec![pdv]);
                p_data_tf_pdus.push(p_data_tf);
            }
        }

        if !data_set_buf.is_empty() {
            unimplemented!("データセットの変換処理は未実装");
        }

        p_data_tf_pdus
    }
}
