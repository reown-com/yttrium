use {
    crate::{
        sign::{
            client::{generate_client_id_key, Client},
            client_errors::{
                ApproveError, ConnectError, DisconnectError, PairError,
                RejectError, RespondError,
            },
            client_types::{ConnectParams, SessionProposal},
            protocol_types::{Metadata, SettleNamespace},
            IncomingSessionMessage,
        },
        uniffi_compat::sign::{
            ffi_types::{
                ConnectParamsFfi, ConnectResultFfi, ErrorDataFfi, SessionFfi,
                SessionProposalFfi, SessionRequestJsonRpcFfi,
                SessionRequestJsonRpcResponseFfi, RejectionReasonFfi,
            },
            session_store::{SessionStoreFfi, SessionStoreFfiProxy},
        },
    },
    relay_rpc::{
        domain::{ProjectId, Topic},
    },
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
    fn on_session_event(&self, id: u64, topic: String, params: bool);
    fn on_session_extend(&self, id: u64, topic: String);
    fn on_session_update(&self, id: u64, topic: String, params: bool);
    fn on_session_connect(&self, id: u64);
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
        session_store: Arc<dyn SessionStoreFfi>,
    ) -> Self {
        tracing::debug!(
            "Creating new SignClient with project_id: {project_id}"
        );
        let (client, session_request_rx) = Client::new(
            ProjectId::from(project_id),
            key.try_into().expect("Invalid key format - must be 32 bytes"),
            Arc::new(SessionStoreFfiProxy(session_store)),
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
                            id,
                            topic,
                            params,
                        ) => {
                            listener.on_session_event(
                                id,
                                topic.to_string(),
                                params,
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
                        IncomingSessionMessage::SessionConnect(id) => {
                            listener.on_session_connect(id);
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
        Ok(proposal.into())
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
        reason: RejectionReasonFfi,
    ) -> Result<(), RejectError> {
        let proposal: SessionProposal = proposal.into();
        let reason: crate::sign::client_types::RejectionReason = reason.into();
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

    pub async fn disconnect(
        &self,
        topic: String,
    ) -> Result<(), DisconnectError> {
        let mut client = self.client.lock().await;
        client.disconnect(topic.into()).await?;
        Ok(())
    }
}
