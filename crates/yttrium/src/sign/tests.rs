#![cfg(feature = "test_depends_on_env_REOWN_PROJECT_ID")]
use {
    crate::sign::{
        client::{generate_client_id_key, Client},
        client_types::{ConnectParams, Session},
        protocol_types::{Metadata, ProposalNamespace, SettleNamespace},
        storage::Storage,
        IncomingSessionMessage,
    },
    relay_rpc::domain::Topic,
    std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    },
};

struct MySessionStoreInner {
    sessions: Vec<Session>,
    pairing_keys: HashMap<Topic, (u64, [u8; 32], [u8; 32])>,
    partial_sessions: HashMap<Topic, [u8; 32]>,
}

struct MySessionStore(Arc<Mutex<MySessionStoreInner>>);

impl Storage for MySessionStore {
    fn add_session(&self, session: Session) {
        let mut inner = self.0.lock().unwrap();
        inner.sessions.push(session);
    }

    fn delete_session(&self, topic: Topic) {
        let mut inner = self.0.lock().unwrap();
        inner.sessions.retain(|s| s.topic != topic);
    }

    fn get_session(&self, topic: Topic) -> Option<Session> {
        let inner = self.0.lock().unwrap();
        inner.sessions.iter().find(|s| s.topic == topic).cloned()
    }

    fn get_all_sessions(&self) -> Vec<Session> {
        let inner = self.0.lock().unwrap();
        inner.sessions.clone()
    }

    fn get_all_topics(&self) -> Vec<Topic> {
        let inner = self.0.lock().unwrap();
        inner
            .sessions
            .iter()
            .map(|s| s.topic.clone())
            .chain(inner.pairing_keys.keys().cloned())
            .chain(inner.partial_sessions.keys().cloned())
            .collect()
    }

    fn get_decryption_key_for_topic(&self, topic: Topic) -> Option<[u8; 32]> {
        let inner = self.0.lock().unwrap();
        inner
            .sessions
            .iter()
            .find(|session| session.topic == topic)
            .map(|session| session.session_sym_key)
            .or_else(|| {
                inner.pairing_keys.get(&topic).map(|(_, sym_key, _)| *sym_key)
            })
            .or_else(|| inner.partial_sessions.get(&topic).copied())
    }

    fn save_pairing(
        &self,
        topic: Topic,
        rpc_id: u64,
        sym_key: [u8; 32],
        self_key: [u8; 32],
    ) {
        let mut inner = self.0.lock().unwrap();
        inner.pairing_keys.insert(topic, (rpc_id, sym_key, self_key));
    }

    fn get_pairing(
        &self,
        topic: Topic,
        _rpc_id: u64,
    ) -> Option<([u8; 32], [u8; 32])> {
        let inner = self.0.lock().unwrap();
        inner
            .pairing_keys
            .get(&topic)
            .map(|(_, sym_key, self_key)| (*sym_key, *self_key))
    }

    fn save_partial_session(&self, topic: Topic, sym_key: [u8; 32]) {
        let mut inner = self.0.lock().unwrap();
        inner.partial_sessions.insert(topic, sym_key);
    }
}

#[tokio::test]
async fn test_sign() {
    let (mut app_client, mut app_session_request_rx) = Client::new(
        std::env::var("REOWN_PROJECT_ID").unwrap().into(),
        generate_client_id_key(),
        Arc::new(MySessionStore(Arc::new(Mutex::new(MySessionStoreInner {
            sessions: vec![],
            pairing_keys: HashMap::new(),
            partial_sessions: HashMap::new(),
        })))),
    );
    let connect_result = app_client
        .connect(
            ConnectParams {
                optional_namespaces: HashMap::from([(
                    "eip155".to_string(),
                    ProposalNamespace {
                        chains: vec!["eip155:11155111".to_string()],
                        methods: vec!["personal_sign".to_string()],
                        events: vec![],
                    },
                )]),
                session_properties: None,
                scoped_properties: None,
            },
            Metadata {
                name: "Reown Rust Tests App".to_string(),
                description: "Reown Rust Tests App".to_string(),
                url: "https://reown.com".to_string(),
                icons: vec![],
                verify_url: None,
                redirect: None,
            },
        )
        .await
        .unwrap();

    let (mut wallet_client, mut wallet_session_request_rx) = Client::new(
        std::env::var("REOWN_PROJECT_ID").unwrap().into(),
        generate_client_id_key(),
        Arc::new(MySessionStore(Arc::new(Mutex::new(MySessionStoreInner {
            sessions: vec![],
            pairing_keys: HashMap::new(),
            partial_sessions: HashMap::new(),
        })))),
    );
    let pairing = wallet_client.pair(&connect_result.uri).await.unwrap();

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
        name: "Reown Rust Tests Wallet".to_string(),
        description: "Reown Rust Tests Wallet".to_string(),
        url: "https://reown.com".to_string(),
        icons: vec![],
        verify_url: None,
        redirect: None,
    };

    wallet_client.approve(pairing, namespaces, metadata).await.unwrap();

    let message = wallet_session_request_rx.recv().await.unwrap();
    assert!(matches!(message.1, IncomingSessionMessage::SessionConnect(_)));

    let message = app_session_request_rx.recv().await.unwrap();
    assert!(matches!(message.1, IncomingSessionMessage::SessionConnect(_)));

    // TODO send session request (e.g. personal_sign)
    // TODO assert session request is received on wallet-side
}
