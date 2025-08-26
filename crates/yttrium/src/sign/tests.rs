#![cfg(feature = "test_depends_on_env_REOWN_PROJECT_ID")]
use {
    crate::sign::{
        client::{generate_client_id_key, Client},
        client_types::{Session, Storage},
        protocol_types::{Metadata, SettleNamespace},
    },
    relay_rpc::domain::Topic,
    std::{collections::HashMap, sync::Arc},
};

struct MySessionStore;
impl Storage for MySessionStore {
    fn add_session(&self, session: Session) {
        println!("add_session: {session:?}");
    }

    fn delete_session(&self, topic: Topic) -> Option<Session> {
        println!("delete_session: {topic:?}");
        None
    }

    fn get_session(&self, topic: Topic) -> Option<Session> {
        println!("get_session: {topic:?}");
        None
    }

    fn get_all_sessions(&self) -> Vec<Session> {
        println!("get_all_sessions");
        vec![]
    }

    fn get_decryption_key_for_topic(&self, topic: Topic) -> Option<[u8; 32]> {
        println!("get_decryption_key_for_topic: {topic:?}");
        None
    }

    fn save_pairing_key(&self, topic: Topic, _sym_key: [u8; 32]) {
        println!("save_pairing_key: {topic:?}");
    }
}

#[tokio::test]
#[ignore]
async fn test_sign() {
    let (mut client, mut session_request_rx) = Client::new(
        std::env::var("REOWN_PROJECT_ID").unwrap().into(),
        generate_client_id_key(),
        Arc::new(MySessionStore),
    );
    let uri = std::env::var("PAIRING_URI").unwrap();
    let pairing = client.pair(&uri).await.unwrap();

    let mut namespaces = HashMap::new();
    for (namespace, namespace_proposal) in pairing.required_namespaces.clone() {
        let accounts = namespace_proposal
            .chains
            .iter()
            .map(|chain| {
                format!(
                    "{}:{}",
                    chain, "0x0000000000000000000000000000000000000000"
                )
            })
            .collect();
        let namespace_settle = SettleNamespace {
            accounts,
            methods: namespace_proposal.methods,
            events: namespace_proposal.events,
            chains: namespace_proposal.chains,
        };
        namespaces.insert(namespace, namespace_settle);
    }
    tracing::debug!("namespaces: {:?}", namespaces);

    let metadata = Metadata {
        name: "Reown Swift Sample Wallet".to_string(),
        description: "Reown Swift Sample Wallet".to_string(),
        url: "https://reown.com".to_string(),
        icons: vec![],
        verify_url: None,
        redirect: None,
    };

    client.approve(pairing, namespaces, metadata).await.unwrap();
    let message = session_request_rx.recv().await.unwrap();
    println!("message: {message:?}");
    let message = session_request_rx.recv().await.unwrap();
    println!("message: {message:?}");

    // TODO where to decrypt message?
    // - in the websocket handler
    //   - provide mechanism to access session sym_key shared storage
}
