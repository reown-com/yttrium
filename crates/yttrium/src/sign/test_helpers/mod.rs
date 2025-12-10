use {
    crate::sign::{
        client::{Client, generate_client_id_key},
        client_types::{ConnectParams, Session, TransportType},
        protocol_types::{
            JsonRpcVersion, Metadata, ProposalNamespace, ProtocolRpcId,
            SessionRequest, SessionRequestJsonRpcResponse,
            SessionRequestJsonRpcResultResponse, SessionRequestRequest,
            SettleNamespace,
        },
        relay::IncomingSessionMessage,
        storage::{Jwk, Storage, StorageError, StoragePairing},
        verify::validate::VerifyValidation,
    },
    relay_rpc::domain::Topic,
    std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    },
};

#[derive(Clone)]
struct JsonRpcHistoryEntry {
    topic: Topic,
    response: Option<String>,
}

struct MySessionStoreInner {
    sessions: Vec<Session>,
    pairing_keys: HashMap<Topic, (ProtocolRpcId, StoragePairing)>,
    partial_sessions: HashMap<Topic, [u8; 32]>,
    json_rpc_history: HashMap<ProtocolRpcId, JsonRpcHistoryEntry>,
}

struct MySessionStore(Arc<Mutex<MySessionStoreInner>>);

impl MySessionStore {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(MySessionStoreInner {
            sessions: vec![],
            pairing_keys: HashMap::new(),
            partial_sessions: HashMap::new(),
            json_rpc_history: HashMap::new(),
        })))
    }
}

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
        rpc_id: ProtocolRpcId,
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
        _rpc_id: ProtocolRpcId,
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

    fn get_verify_public_key(&self) -> Result<Option<Jwk>, StorageError> {
        Ok(None)
    }

    fn set_verify_public_key(
        &self,
        _public_key: Jwk,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    fn insert_json_rpc_history(
        &self,
        request_id: ProtocolRpcId,
        topic: Topic,
        _method: String,
        _body: String,
        _transport_type: Option<TransportType>,
    ) -> Result<(), StorageError> {
        let mut inner = self.0.lock().unwrap();
        inner
            .json_rpc_history
            .insert(request_id, JsonRpcHistoryEntry { topic, response: None });
        Ok(())
    }

    fn update_json_rpc_history_response(
        &self,
        request_id: ProtocolRpcId,
        response: String,
    ) -> Result<(), StorageError> {
        let mut inner = self.0.lock().unwrap();
        let entry =
            inner.json_rpc_history.get_mut(&request_id).ok_or_else(|| {
                StorageError::Runtime(format!(
                    "JSON-RPC history entry not found for request_id: {}",
                    request_id
                ))
            })?;
        entry.response = Some(response);
        Ok(())
    }

    fn delete_json_rpc_history_by_topic(
        &self,
        topic: Topic,
    ) -> Result<(), StorageError> {
        let mut inner = self.0.lock().unwrap();
        inner.json_rpc_history.retain(|_, entry| entry.topic != topic);
        Ok(())
    }

    fn does_json_rpc_exist(
        &self,
        request_id: ProtocolRpcId,
    ) -> Result<bool, StorageError> {
        let inner = self.0.lock().unwrap();
        Ok(inner.json_rpc_history.contains_key(&request_id))
    }
}

pub async fn test_sign_impl() -> Result<(), String> {
    let app_client_id = generate_client_id_key();
    tracing::debug!(group = "app", probe = "client_id_generated");
    let (mut app_client, mut app_session_request_rx) = Client::new(
        std::env::var("REOWN_PROJECT_ID").unwrap().into(),
        app_client_id,
        Arc::new(MySessionStore::new()),
    );
    app_client.set_probe_group("app".to_string());
    tracing::debug!(group = "app", probe = "client_created");
    app_client.start();
    tracing::debug!(group = "app", probe = "client_started");
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
    tracing::debug!(group = "app", probe = "connect_finished");

    let wallet_client_id = generate_client_id_key();
    tracing::debug!(group = "wallet", probe = "client_id_generated");
    let (mut wallet_client, mut wallet_session_request_rx) = Client::new(
        std::env::var("REOWN_PROJECT_ID").unwrap().into(),
        wallet_client_id,
        Arc::new(MySessionStore::new()),
    );
    wallet_client.set_probe_group("wallet".to_string());
    tracing::debug!(group = "wallet", probe = "client_created");
    wallet_client.start();
    tracing::debug!(group = "wallet", probe = "client_started");
    let pairing = wallet_client
        .pair(&connect_result.uri)
        .await
        .map_err(|e| format!("Failed to pair: {e}"))?;
    tracing::debug!(group = "wallet", probe = "pair_finished");

    assert_eq!(pairing.1.validation, VerifyValidation::Unknown);

    let mut namespaces = HashMap::new();
    for (namespace, namespace_proposal) in pairing.0.required_namespaces.clone()
    {
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
    tracing::debug!(group = "wallet", probe = "metadata");

    wallet_client
        .approve(pairing.0, namespaces, metadata)
        .await
        .map_err(|e| format!("Failed to approve: {e}"))?;
    tracing::debug!(group = "wallet", probe = "approve_finished");

    let message = wallet_session_request_rx
        .recv()
        .await
        .ok_or_else(|| "Failed to receive session connect".to_string())?;
    if !(matches!(message.1, IncomingSessionMessage::SessionConnect(_, _))) {
        Err(format!("Expected SessionConnect, got {:?}", message.1))?;
    }
    tracing::debug!(group = "wallet", probe = "session_connect_received");

    let message = app_session_request_rx
        .recv()
        .await
        .ok_or_else(|| "Failed to receive session connect".to_string())?;
    assert!(matches!(message.1, IncomingSessionMessage::SessionConnect(_, _)));
    tracing::debug!(group = "app", probe = "session_connect_received");

    tracing::debug!(group = "app", probe = "requesting_personal_sign");
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
    tracing::debug!(group = "wallet", probe = "receiving_session_request");
    let message = wallet_session_request_rx.recv().await.unwrap();
    assert!(matches!(message.1, IncomingSessionMessage::SessionRequest(_, _)));
    tracing::debug!(group = "wallet", probe = "received_session_request");
    let req = if let IncomingSessionMessage::SessionRequest(req, _) = message.1
    {
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
    tracing::debug!(group = "wallet", probe = "responding_to_session_request");
    wallet_client
        .respond(
            message.0,
            SessionRequestJsonRpcResponse::Result(
                SessionRequestJsonRpcResultResponse {
                    id: req.id,
                    jsonrpc: JsonRpcVersion::version_2(),
                    result: serde_json::Value::String("0x0".to_string()),
                },
            ),
        )
        .await
        .unwrap();
    tracing::debug!(
        group = "app",
        probe = "receiving_session_request_response"
    );
    let message = app_session_request_rx.recv().await.unwrap();
    tracing::debug!(group = "app", probe = "received_session_request_response");
    tracing::debug!("message: {:?}", message);
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
