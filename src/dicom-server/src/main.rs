use dicom_server::dicom::network::pdu::a_associate_rq::AAssociateRq;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const BUFFER_SIZE: usize = 4096;
const SERVER_AE_TITLE: &str = "SERVER";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:104").await?;
    println!("104 ポートをリッスンしています...");

    let (mut socket, addr) = listener.accept().await?;
    println!(
        "コネクションを受け入れました ({}:{})",
        addr.ip(),
        addr.port()
    );

    let mut buf = [0u8; BUFFER_SIZE];
    let n = socket.read(&mut buf).await?;

    let a_associate_rq = match AAssociateRq::try_from(&buf[0..n]) {
        Ok(req) => req,
        Err(message) => {
            eprintln!("A-ASSOCIATE-RQ PDU のパースに失敗しました: {}", message);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "A-ASSOCIATE-RQ PDU のパースに失敗しました",
            ));
        }
    };

    println!("Calling AE Title: {}", a_associate_rq.calling_ae_title());
    println!("Called AE Title: {}", a_associate_rq.called_ae_title());
    println!("Version: {}", a_associate_rq.version());
    println!("--------------------");
    println!(
        "Application Context: {}",
        a_associate_rq.application_context().name()
    );
    println!(
        "Presentation Contexts: {}",
        a_associate_rq.presentation_contexts().len()
    );
    for pc in a_associate_rq.presentation_contexts() {
        println!(
            "  Context ID: {}, Abstract Syntax: {}, Transfer Syntaxes: {}",
            pc.context_id(),
            pc.abstract_syntax().name(),
            pc.transfer_syntaxes().len()
        );
        for ts in pc.transfer_syntaxes() {
            println!("    Transfer Syntax: {}", ts.name());
        }
    }
    println!("User Information: ");
    if a_associate_rq.user_information().maximum_length().is_some() {
        println!(
            "  Maximum PDU Length: {}",
            a_associate_rq
                .user_information()
                .maximum_length()
                .unwrap()
                .maximum_length()
        );
    }
    if a_associate_rq
        .user_information()
        .implementation_class_uid()
        .is_some()
    {
        println!(
            "  Implementation Class UID: {}",
            a_associate_rq
                .user_information()
                .implementation_class_uid()
                .unwrap()
                .uid()
        );
    }

    println!("コネクションを切断します");
    socket.shutdown().await?;

    Ok(())
}
