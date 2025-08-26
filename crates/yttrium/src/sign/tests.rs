#![cfg(feature = "test_depends_on_env_REOWN_PROJECT_ID")]
use {
    crate::sign::{
        client::{generate_client_id_key, Client},
        client_types::{Session, SessionStore},
        protocol_types::{Metadata, SettleNamespace},
    },
    std::{collections::HashMap, sync::Arc},
};

struct MySessionStore;
impl SessionStore for MySessionStore {
    fn add_session(&self, session: Session) {
        println!("add_session: {session:?}");
    }

    fn delete_session(&self, topic: String) {
        println!("delete_session: {topic:?}");
    }

    fn get_session(&self, topic: String) -> Option<Session> {
        println!("get_session: {topic:?}");
        None
    }

    fn get_all_sessions(&self) -> Vec<Session> {
        println!("get_all_sessions");
        vec![]
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
