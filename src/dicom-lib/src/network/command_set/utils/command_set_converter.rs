use crate::network::{
    CommandSet,
    upper_layer_protocol::pdu::{PDataTf, p_data_tf::PresentationDataValue},
};

pub fn p_data_tf_pdus_to_command_set(p_data_tf_pdus: &[PDataTf]) -> Result<CommandSet, String> {
    let buffer = p_data_tf_pdus
        .iter()
        .flat_map(|p_data_tf| p_data_tf.presentation_data_values())
        .filter(|pdv| pdv.is_command())
        .flat_map(|pdv| pdv.data().to_vec())
        .collect::<Vec<_>>();

    let command_set = CommandSet::try_from(buffer.as_ref())
        .map_err(|e| format!("コマンドセットのパースに失敗しました: {e}"))?;

    Ok(command_set)
}

pub fn command_set_to_p_data_tf_pdus(
    command_set: CommandSet,
    presentation_context_id: u8,
    maximum_length: u32,
) -> Vec<PDataTf> {
    let mut p_data_tf_pdus = vec![];
    let data: Vec<u8> = command_set.into();

    let max_chunk_size = if maximum_length == 0 {
        data.len()
    } else {
        maximum_length as usize - 6 // Presentation Data Value Itemの実データ以外のサイズが6バイト
    };
    let mut offset = 0;
    while offset < data.len() {
        let chunk = &data[offset..offset + max_chunk_size.min(data.len() - offset)];
        offset += chunk.len();
        let is_last = offset >= data.len();
        let pdv = PresentationDataValue::new(presentation_context_id, true, is_last, chunk);

        let p_data_tf = PDataTf::new(vec![pdv]);
        p_data_tf_pdus.push(p_data_tf);
    }

    p_data_tf_pdus
}
