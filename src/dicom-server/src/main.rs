use dicom_server::dicom::network::{
    CommandSet,
    dimse::c_echo::{
        c_echo_rq::CEchoRq,
        c_echo_rsp::{CEchoRsp, Status},
    },
    upper_layer_protocol::{
        pdu::{
            AAssociateAc, AAssociateRq, PDataTf,
            a_associate_ac::{
                ApplicationContext, PresentationContext, UserInformation,
                presentation_context::{ResultReason, TransferSyntax},
                user_information::{
                    ImplementationClassUid, ImplementationVersionName, MaximumLength,
                },
            },
            a_associate_rq, p_data_tf,
        },
        utils::command_set_converter::{
            command_set_to_p_data_tf_pdus, p_data_tf_pdus_to_command_set,
        },
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

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

    let a_associate_rq = {
        let mut buf_reader = tokio::io::BufReader::new(&mut socket);

        let pdu_type = buf_reader.read_u8().await?;
        if pdu_type != a_associate_rq::PDU_TYPE {
            // TODO: A-ASSOCIATE-RQ以外のPDUいきなり来た時のエラー処理を実装
            panic!("A-ASSOCIATE-RQ 以外の PDU が受信されました");
        }
        buf_reader.read_u8().await?; // Reserved
        let pdu_length = buf_reader.read_u32().await?;

        match AAssociateRq::read_from_stream(&mut buf_reader, pdu_length).await {
            Ok(req) => req,
            Err(e) => {
                panic!("A-ASSOCIATE-RQ PDU のパースに失敗しました: {:?}", e);
            }
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
    let mut maximum_length = 0;
    if user_information.maximum_length().is_some() {
        maximum_length = user_information.maximum_length().unwrap().maximum_length();
        println!("  Maximum PDU Length: {maximum_length}");
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

    let p_data_tf = {
        let mut buf_reader = tokio::io::BufReader::new(&mut socket);

        let pdu_type = buf_reader.read_u8().await?;
        if pdu_type != p_data_tf::PDU_TYPE {
            // TODO: P-DATA-TF以外のPDUがいきなり来た時のエラー処理を実装
            panic!("P-DATA-TF 以外の PDU が受信されました");
        }
        buf_reader.read_u8().await?; // Reserved
        let pdu_length = buf_reader.read_u32().await?;

        match PDataTf::read_from_stream(&mut buf_reader, pdu_length).await {
            Ok(req) => req,
            Err(e) => {
                panic!("P-DATA-TF PDU のパースに失敗しました: {:?}", e);
            }
        }
    };
    println!("P-DATA-TF PDU を受信しました");

    let presentation_context_id = p_data_tf.presentation_data_values()[0].presentation_context_id();
    let c_echo_rq = {
        let command_set = p_data_tf_pdus_to_command_set(&[p_data_tf]).unwrap_or_else(|e| {
            panic!("コマンドセットのパースに失敗しました: {e}");
        });

        CEchoRq::try_from(command_set).unwrap_or_else(|e| {
            panic!("C-ECHO-RQ のパースに失敗しました: {e}");
        })
    };

    // TODO: エラー処理
    //     : エラー内容に応じて適切なステータスでC-ECHO-RSPを生成し、クライアントに送信する
    let c_echo_rsp = CEchoRsp::new(c_echo_rq.message_id(), Status::Success);
    println!("C-ECHO-RSP を送信します");
    {
        let command_set: CommandSet = c_echo_rsp.into();
        let p_data_tf_pdus =
            command_set_to_p_data_tf_pdus(&command_set, presentation_context_id, maximum_length);
        for p_data_tf in p_data_tf_pdus {
            let bytes: Vec<u8> = (&p_data_tf).into();
            socket.write_all(&bytes).await?;
        }
    }

    println!("コネクションを切断します");
    socket.shutdown().await?;

    Ok(())
}
