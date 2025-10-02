use {
    crate::{
        sign::{
            client::{generate_client_id_key, Client},
            client_errors::{
                ApproveError, ConnectError, DisconnectError, EmitError,
                ExtendError, PairError, RejectError, RequestError,
                RespondError, UpdateError,
            },
            client_types::{ConnectParams, SessionProposal},
            protocol_types::{Metadata, SessionRequest, SettleNamespace},
            IncomingSessionMessage,
        },
        uniffi_compat::sign::{
            ffi_types::{
                ConnectParamsFfi, ConnectResultFfi, SessionFfi,
                SessionProposalFfi, SessionRequestFfi,
                SessionRequestJsonRpcFfi, SessionRequestJsonRpcResponseFfi,
            },
            storage::{StorageFfi, StorageFfiProxy},
        },
    },
    relay_rpc::domain::{ProjectId, Topic},
    std::{collections::HashMap, sync::Arc},
};

#[uniffi::export(with_foreign)]
pub trait SignListener: Send + Sync {
    fn on_session_request(
        &self,
        topic: String,
        session_request: SessionRequestJsonRpcFfi,
    );

    fn on_session_disconnect(&self, id: u64, topic: String);
    fn on_session_event(
        &self,
        topic: String,
        name: String,
        data: String,
        chain_id: String,
    );
    fn on_session_extend(&self, id: u64, topic: String);
    fn on_session_update(
        &self,
        id: u64,
        topic: String,
        namespaces: std::collections::HashMap<String, SettleNamespace>,
    );
    fn on_session_connect(&self, id: u64, topic: String);
    fn on_session_reject(&self, id: u64, topic: String);
    fn on_session_request_response(
        &self,
        id: u64,
        topic: String,
        response: SessionRequestJsonRpcResponseFfi,
    );
}

#[derive(uniffi::Object)]
pub struct SignClient {
    client: std::sync::Arc<tokio::sync::Mutex<Client>>,
    session_request_rx: std::sync::Mutex<
        Option<
            tokio::sync::mpsc::UnboundedReceiver<(
                Topic,
                IncomingSessionMessage,
            )>,
        >,
    >,
}

#[uniffi::export(async_runtime = "tokio")]
impl SignClient {
    #[uniffi::constructor]
    pub fn new(
        project_id: String,
        key: Vec<u8>,
        session_store: Arc<dyn StorageFfi>,
    ) -> Self {
        tracing::debug!(
            "Creating new SignClient with project_id: {project_id}"
        );
        let (client, session_request_rx) = Client::new(
            ProjectId::from(project_id),
            key.try_into().expect("Invalid key format - must be 32 bytes"),
            Arc::new(StorageFfiProxy(session_store)),
        );
        Self {
            client: std::sync::Arc::new(tokio::sync::Mutex::new(client)),
            session_request_rx: std::sync::Mutex::new(Some(session_request_rx)),
        }
    }

    pub async fn start(&self) {
        let mut client = self.client.lock().await;
        client.start();
    }

    pub fn generate_key(&self) -> Vec<u8> {
        generate_client_id_key().to_vec()
    }

    pub async fn register_sign_listener(
        &self,
        listener: Arc<dyn SignListener>,
    ) {
        let mut rx_guard = self.session_request_rx.lock().unwrap();
        if let Some(mut rx) = rx_guard.take() {
            tokio::spawn(async move {
                tracing::info!(
                    "Starting session request listener with debug logging"
                );
                while let Some((topic, message)) = rx.recv().await {
                    match message {
                        IncomingSessionMessage::SessionRequest(request) => {
                            tracing::debug!("Received session request - Topic: {:?}, SessionRequest: {:?}", topic, request);
                            let session_request_ffi: SessionRequestJsonRpcFfi =
                                request.into();
                            listener.on_session_request(
                                topic.to_string(),
                                session_request_ffi,
                            );
                        }
                        IncomingSessionMessage::Disconnect(id, topic) => {
                            listener
                                .on_session_disconnect(id, topic.to_string());
                        }
                        IncomingSessionMessage::SessionEvent(
                            topic,
                            name,
                            data,
                            chain_id,
                        ) => {
                            listener.on_session_event(
                                topic.to_string(),
                                name,
                                serde_json::to_string(&data)
                                    .unwrap_or_default(),
                                chain_id,
                            );
                        }
                        IncomingSessionMessage::SessionUpdate(
                            id,
                            topic,
                            params,
                        ) => {
                            listener.on_session_update(
                                id,
                                topic.to_string(),
                                params,
                            );
                        }
                        IncomingSessionMessage::SessionExtend(id, topic) => {
                            listener.on_session_extend(id, topic.to_string());
                        }
                        IncomingSessionMessage::SessionConnect(id, topic) => {
                            listener.on_session_connect(id, topic.to_string());
                        }
                        IncomingSessionMessage::SessionReject(id, topic) => {
                            listener.on_session_reject(id, topic.to_string());
                        }
                        IncomingSessionMessage::SessionRequestResponse(
                            id,
                            topic,
                            response,
                        ) => {
                            listener.on_session_request_response(
                                id,
                                topic.to_string(),
                                response.into(),
                            );
                        }
                    }
                }
                tracing::info!("Session request listener stopped");
            });
        } else {
            tracing::warn!("Session request listener already started or receiver not available");
        }
    }

    pub async fn online(&self) {
        tracing::info!("Calling online method");
        let mut client = self.client.lock().await;
        client.online();
    }

    pub async fn pair(
        &self,
        uri: String,
    ) -> Result<SessionProposalFfi, PairError> {
        let proposal = {
            let mut client = self.client.lock().await;
            client.pair(&uri).await?
        };
        Ok(proposal.0.into())
    }

    pub async fn connect(
        &self,
        params: ConnectParamsFfi,
        self_metadata: Metadata,
    ) -> Result<ConnectResultFfi, ConnectError> {
        let params: ConnectParams = params.into();
        tracing::debug!("connect params: {:?}", params);
        tracing::debug!("self_metadata: {:?}", self_metadata);
        let result = {
            let mut client = self.client.lock().await;
            client.connect(params, self_metadata).await?
        };
        Ok(result.into())
    }

    //TODO: Add approved namespaces builder util function
    pub async fn approve(
        &self,
        proposal: SessionProposalFfi,
        approved_namespaces: HashMap<String, SettleNamespace>,
        self_metadata: Metadata,
    ) -> Result<SessionFfi, ApproveError> {
        let proposal: SessionProposal = proposal.into();
        tracing::debug!("approved_namespaces: {:?}", approved_namespaces);
        tracing::debug!("self_metadata: {:?}", self_metadata);
        let session = {
            let mut client = self.client.lock().await;
            client.approve(proposal, approved_namespaces, self_metadata).await?
        };
        Ok(session.into())
    }

    pub async fn reject(
        &self,
        proposal: SessionProposalFfi,
        reason: crate::sign::client_types::RejectionReason,
    ) -> Result<(), RejectError> {
        let proposal: SessionProposal = proposal.into();
        tracing::debug!("reject session propose: {:?}", reason);

        let mut client = self.client.lock().await;
        client.reject(proposal, reason).await?;
        Ok(())
    }

    pub async fn respond(
        &self,
        topic: String,
        response: SessionRequestJsonRpcResponseFfi,
    ) -> Result<String, RespondError> {
        use crate::sign::protocol_types::SessionRequestJsonRpcResponse;

        tracing::debug!("responding session request: {:?}", response);

        let mut client = self.client.lock().await;
        let response_internal: SessionRequestJsonRpcResponse = response.into();
        let topic_topic: Topic = topic.clone().into();
        client.respond(topic_topic, response_internal).await?;
        Ok(topic)
    }

    pub async fn emit(
        &self,
        topic: String,
        name: String,
        data: String,
        chain_id: String,
    ) -> Result<(), EmitError> {
        let mut client = self.client.lock().await;
        let data_value = match serde_json::from_str::<serde_json::Value>(&data)
        {
            Ok(v) => v,
            Err(_) => serde_json::Value::String(data.clone()),
        };
        client.emit(topic.into(), name, data_value, chain_id).await
    }

    pub async fn disconnect(
        &self,
        topic: String,
    ) -> Result<(), DisconnectError> {
        let mut client = self.client.lock().await;
        client.disconnect(topic.into()).await?;
        Ok(())
    }

    pub async fn update(
        &self,
        topic: String,
        namespaces: std::collections::HashMap<String, SettleNamespace>,
    ) -> Result<(), UpdateError> {
        let mut client = self.client.lock().await;
        client.update(topic.into(), namespaces).await
    }

    pub async fn extend(&self, topic: String) -> Result<(), ExtendError> {
        let mut client = self.client.lock().await;
        client.extend(topic.into()).await
    }

    pub async fn request(
        &self,
        topic: String,
        session_request: SessionRequestFfi,
    ) -> Result<u64, RequestError> {
        let mut client = self.client.lock().await;
        let session_request: SessionRequest = session_request.into();
        client.request(topic.into(), session_request).await
    }
}
