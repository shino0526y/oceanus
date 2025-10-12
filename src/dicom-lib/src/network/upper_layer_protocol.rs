pub mod pdu;
pub mod states;

pub use states::*;

use crate::network::upper_layer_protocol::pdu::{AAbort, a_abort};
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub async fn send_a_abort(
    socket: &mut (impl AsyncWrite + Unpin),
    source: a_abort::Source,
    reason: a_abort::Reason,
) -> std::io::Result<()> {
    let a_abort = AAbort::new(source, reason);

    let bytes: Vec<u8> = a_abort.into();
    socket.write_all(&bytes).await?;

    Ok(())
}
