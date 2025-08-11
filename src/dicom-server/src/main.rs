use dicom_server::dicom::network::{
    CommandSet,
    upper_layer_protocol::pdu::{
        AAssociateAc, AAssociateRq, PDataTf,
        a_associate_ac::{
            ApplicationContext, PresentationContext, UserInformation,
            presentation_context::{ResultReason, TransferSyntax},
            user_information::{ImplementationClassUid, ImplementationVersionName, MaximumLength},
        },
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

const BUFFER_SIZE: usize = 4096;
const SERVER_AE_TITLE: &str = "SERVER";
const IMPLEMENTATION_CLASS_UID: &str = "1.2.826.0.1.3680043.2.1396.999";
const IMPLEMENTATION_VERSION_NAME: &str = "Oceanus";
const MAXIMUM_LENGTH: u32 = 0;

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
            eprintln!("A-ASSOCIATE-RQ PDU のパースに失敗しました: {message}");
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "A-ASSOCIATE-RQ PDU のパースに失敗しました",
            ));
        }
    };

    let called_ae_title = a_associate_rq.called_ae_title();
    let calling_ae_title = a_associate_rq.calling_ae_title();
    let application_context = a_associate_rq.application_context();
    let presentation_contexts = a_associate_rq.presentation_contexts();
    let user_information = a_associate_rq.user_information();

    println!("Calling AE Title: {calling_ae_title}");
    println!("Called AE Title: {called_ae_title}");
    println!("Version: {}", a_associate_rq.version());
    println!("--------------------");
    println!("Application Context: {}", application_context.name());
    println!("Presentation Contexts: {}", presentation_contexts.len());
    for pc in presentation_contexts {
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
    if user_information.maximum_length().is_some() {
        println!(
            "  Maximum PDU Length: {}",
            user_information.maximum_length().unwrap().maximum_length()
        );
    }
    println!(
        "  Implementation Class UID: {}",
        user_information.implementation_class_uid().uid()
    );
    println!("--------------------");
    if called_ae_title != SERVER_AE_TITLE {
        // TODO: A_ASSOCIATE_RJを送信する
        panic!("サーバーの AE タイトルとクライアントの AE タイトルが一致しません");
    }

    let application_context = ApplicationContext::new("1.2.840.10008.3.1.1.1.1");
    let presentation_contexts = presentation_contexts
        .iter()
        .map(|presentation_context| {
            PresentationContext::new(
                presentation_context.context_id(),
                if presentation_context.abstract_syntax().name() == "1.2.840.10008.1.1" {
                    ResultReason::Acceptance
                } else {
                    ResultReason::AbstractSyntaxNotSupported
                },
                TransferSyntax::new("1.2.840.10008.1.2").unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let user_information = UserInformation::new(
        Some(MaximumLength::new(MAXIMUM_LENGTH)),
        ImplementationClassUid::new(IMPLEMENTATION_CLASS_UID).unwrap(),
        Some(ImplementationVersionName::new(IMPLEMENTATION_VERSION_NAME).unwrap()),
    );
    let a_associate_ac = AAssociateAc::new(
        1,
        called_ae_title,
        calling_ae_title,
        application_context,
        presentation_contexts,
        user_information,
    )
    .unwrap();

    println!("A-ASSOCIATE-AC PDU を送信します");
    {
        let bytes: Vec<u8> = a_associate_ac.into();
        socket.write_all(&bytes).await?;
    }

    let mut buf = [0u8; BUFFER_SIZE];
    let n = socket.read(&mut buf).await?;

    let p_data_tf = match PDataTf::try_from(&buf[0..n]) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("P-DATA-TF PDU のパースに失敗しました: {e}");
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "P-DATA-TF PDU のパースに失敗しました",
            ));
        }
    };
    println!("P-DATA-TF PDU を受信しました");
    let buffer = p_data_tf
        .presentation_data_values()
        .iter()
        .flat_map(|pdv| pdv.data().to_vec())
        .collect::<Vec<_>>();
    let command_set = CommandSet::try_from(buffer.as_ref()).map_err(|e| {
        eprintln!("コマンドセットのパースに失敗しました: {e}");
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "コマンドセットのパースに失敗しました",
        )
    })?;
    println!("  コマンド:");
    command_set.iter().for_each(|command| {
        let tag = command.tag().to_string();
        let value_field = command
            .value_field()
            .iter()
            .map(|b| format!("0x{b:02X}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!("    {tag} [{value_field}]");
    });

    println!("コネクションを切断します");
    socket.shutdown().await?;

    Ok(())
}
