#![cfg(feature = "test_depends_on_env_REOWN_PROJECT_ID")]
use {
    crate::sign::{
        client::{generate_client_id_key, Client},
        client_types::{ConnectParams, Session},
        protocol_types::{
            Metadata, ProposalNamespace, SessionRequest,
            SessionRequestJsonRpcResponse, SessionRequestJsonRpcResultResponse,
            SessionRequestRequest, SettleNamespace,
        },
        storage::{Storage, StorageError, StoragePairing},
        IncomingSessionMessage,
    },
    aws_sdk_cloudwatch::{
        primitives::DateTime,
        types::{Dimension, MetricDatum, StandardUnit},
    },
    relay_rpc::domain::Topic,
    std::{
        collections::HashMap,
        sync::{Arc, Mutex},
        time::SystemTime,
    },
};

struct MySessionStoreInner {
    sessions: Vec<Session>,
    pairing_keys: HashMap<Topic, (u64, StoragePairing)>,
    partial_sessions: HashMap<Topic, [u8; 32]>,
}

struct MySessionStore(Arc<Mutex<MySessionStoreInner>>);

impl Storage for MySessionStore {
    fn add_session(&self, session: Session) -> Result<(), StorageError> {
        let mut inner = self.0.lock().unwrap();
        inner.sessions.push(session);
        Ok(())
    }

    fn delete_session(&self, topic: Topic) -> Result<(), StorageError> {
        let mut inner = self.0.lock().unwrap();
        inner.sessions.retain(|s| s.topic != topic);
        Ok(())
    }

    fn get_session(
        &self,
        topic: Topic,
    ) -> Result<Option<Session>, StorageError> {
        let inner = self.0.lock().unwrap();
        let session = inner.sessions.iter().find(|s| s.topic == topic).cloned();
        Ok(session)
    }

    fn get_all_sessions(&self) -> Result<Vec<Session>, StorageError> {
        let inner = self.0.lock().unwrap();
        let sessions = inner.sessions.clone();
        Ok(sessions)
    }

    fn get_all_topics(&self) -> Result<Vec<Topic>, StorageError> {
        let inner = self.0.lock().unwrap();
        let topics = inner
            .sessions
            .iter()
            .map(|s| s.topic.clone())
            .chain(inner.pairing_keys.keys().cloned())
            .chain(inner.partial_sessions.keys().cloned())
            .collect();
        Ok(topics)
    }

    fn get_decryption_key_for_topic(
        &self,
        topic: Topic,
    ) -> Result<Option<[u8; 32]>, StorageError> {
        let inner = self.0.lock().unwrap();
        let decryption_key = inner
            .sessions
            .iter()
            .find(|session| session.topic == topic)
            .map(|session| session.session_sym_key)
            .or_else(|| {
                inner.pairing_keys.get(&topic).map(
                    |(_, StoragePairing { sym_key, self_key: _ })| *sym_key,
                )
            })
            .or_else(|| inner.partial_sessions.get(&topic).copied());
        Ok(decryption_key)
    }

    fn save_pairing(
        &self,
        topic: Topic,
        rpc_id: u64,
        sym_key: [u8; 32],
        self_key: [u8; 32],
    ) -> Result<(), StorageError> {
        let mut inner = self.0.lock().unwrap();
        inner
            .pairing_keys
            .insert(topic, (rpc_id, StoragePairing { sym_key, self_key }));
        Ok(())
    }

    fn get_pairing(
        &self,
        topic: Topic,
        _rpc_id: u64,
    ) -> Result<Option<StoragePairing>, StorageError> {
        let inner = self.0.lock().unwrap();
        let pairing = inner
            .pairing_keys
            .get(&topic)
            .map(|(_, storage_pairing)| storage_pairing)
            .cloned();
        Ok(pairing)
    }

    fn save_partial_session(
        &self,
        topic: Topic,
        sym_key: [u8; 32],
    ) -> Result<(), StorageError> {
        let mut inner = self.0.lock().unwrap();
        inner.partial_sessions.insert(topic, sym_key);
        Ok(())
    }
}

async fn test_sign_impl() -> Result<(), String> {
    tracing_subscriber::fmt()
        // .with_max_level(tracing::Level::DEBUG)
        .init();
    let (mut app_client, mut app_session_request_rx) = Client::new(
        std::env::var("REOWN_PROJECT_ID").unwrap().into(),
        generate_client_id_key(),
        Arc::new(MySessionStore(Arc::new(Mutex::new(MySessionStoreInner {
            sessions: vec![],
            pairing_keys: HashMap::new(),
            partial_sessions: HashMap::new(),
        })))),
    );
    app_client.start();
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
        .map_err(|e| format!("Failed to connect: {e}"))?;

    let (mut wallet_client, mut wallet_session_request_rx) = Client::new(
        std::env::var("REOWN_PROJECT_ID").unwrap().into(),
        generate_client_id_key(),
        Arc::new(MySessionStore(Arc::new(Mutex::new(MySessionStoreInner {
            sessions: vec![],
            pairing_keys: HashMap::new(),
            partial_sessions: HashMap::new(),
        })))),
    );
    wallet_client.start();
    let pairing = wallet_client
        .pair(&connect_result.uri)
        .await
        .map_err(|e| format!("Failed to pair: {e}"))?;

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

    let metadata = Metadata {
        name: "Reown Rust Tests Wallet".to_string(),
        description: "Reown Rust Tests Wallet".to_string(),
        url: "https://reown.com".to_string(),
        icons: vec![],
        verify_url: None,
        redirect: None,
    };

    wallet_client
        .approve(pairing, namespaces, metadata)
        .await
        .map_err(|e| format!("Failed to approve: {e}"))?;

    let message = wallet_session_request_rx
        .recv()
        .await
        .ok_or_else(|| format!("Failed to receive session connect"))?;

    if !(matches!(message.1, IncomingSessionMessage::SessionConnect(_))) {
        Err(format!("Expected SessionConnect, got {:?}", message.1))?;
    }

    let message = app_session_request_rx
        .recv()
        .await
        .ok_or_else(|| format!("Failed to receive session connect"))?;
    assert!(matches!(message.1, IncomingSessionMessage::SessionConnect(_)));

    tracing::debug!("Requesting personal sign");
    app_client
        .request(
            message.0,
            SessionRequest {
                chain_id: "eip155:11155111".to_string(),
                request: SessionRequestRequest {
                    method: "personal_sign".to_string(),
                    params: serde_json::Value::String("0x0".to_string()),
                    expiry: Some(0),
                },
            },
        )
        .await
        .unwrap();
    tracing::debug!("Receiving session request");
    let message = wallet_session_request_rx.recv().await.unwrap();
    tracing::debug!("Received session request");
    assert!(matches!(message.1, IncomingSessionMessage::SessionRequest(_)));
    let req = if let IncomingSessionMessage::SessionRequest(req) = message.1 {
        req
    } else {
        panic!("Expected SessionRequest");
    };
    assert_eq!(req.params.chain_id, "eip155:11155111");
    assert_eq!(req.params.request.method, "personal_sign");
    assert_eq!(
        req.params.request.params,
        serde_json::Value::String("0x0".to_string())
    );
    assert_eq!(req.params.request.expiry, Some(0));
    tracing::debug!("Responding to session request");
    wallet_client
        .respond(
            message.0,
            SessionRequestJsonRpcResponse::Result(
                SessionRequestJsonRpcResultResponse {
                    id: req.id,
                    jsonrpc: "2.0".to_string(),
                    result: serde_json::Value::String("0x0".to_string()),
                },
            ),
        )
        .await
        .unwrap();
    tracing::debug!("Receiving session request response");
    let message = app_session_request_rx.recv().await.unwrap();
    assert!(matches!(
        message.1,
        IncomingSessionMessage::SessionRequestResponse(_, _, _)
    ));
    let resp =
        if let IncomingSessionMessage::SessionRequestResponse(_, _, resp) =
            message.1
        {
            resp
        } else {
            panic!("Expected SessionRequestResponse");
        };
    assert!(matches!(resp, SessionRequestJsonRpcResponse::Result(_)));
    let resp = if let SessionRequestJsonRpcResponse::Result(resp) = resp {
        resp
    } else {
        panic!("Expected SessionRequestResponse");
    };
    assert_eq!(resp.result, serde_json::Value::String("0x0".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_sign() {
    let result = test_sign_impl().await;

    let config = aws_config::load_from_env().await;
    let cloudwatch_client = aws_sdk_cloudwatch::Client::new(&config);
    cloudwatch_client
        .put_metric_data()
        .namespace("dev_Canary_RustSignClient")
        .set_metric_data(Some(vec![MetricDatum::builder()
            .metric_name("HappyPath.connects.success".to_string())
            .dimensions(
                Dimension::builder()
                    .name("Target".to_string())
                    .value("test".to_string())
                    .build(),
            )
            .dimensions(
                Dimension::builder()
                    .name("Region".to_string())
                    .value("eu-central-1".to_string())
                    .build(),
            )
            .dimensions(
                Dimension::builder()
                    .name("Tag".to_string())
                    .value("test".to_string())
                    .build(),
            )
            .value(if result.is_ok() { 1. } else { 0. })
            .unit(StandardUnit::Count)
            .timestamp(DateTime::from(SystemTime::now()))
            .build()]))
        .send()
        .await
        .unwrap();

    if let Err(e) = result {
        assert!(false, "Test failed: {e}");
    }
}
