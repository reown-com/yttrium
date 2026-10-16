//! Swift and Kotlin call these generated types directly (no adapter, no JSON), and implement the
//! generated `Storage` and `Listener` interfaces. `call` is the JSON facade for React Native.

use {
    crate::{
        ConnectResult, Listener, Session, SessionProposal, SignClient,
        SignConfig, SignError, Storage,
        client::types::{
            Namespaces, ProposalParams, RpcError, RpcRequest, SessionEventData,
        },
        protocol::types::{DappIntent, Mode},
    },
    std::sync::Arc,
};

#[derive(uniffi::Object)]
pub struct SignClientFfi {
    client: SignClient,
}

#[uniffi::export(async_runtime = "tokio")]
impl SignClientFfi {
    #[uniffi::constructor]
    pub async fn new(
        config: SignConfig,
        storage: Arc<dyn Storage>,
        listener: Arc<dyn Listener>,
    ) -> Result<Arc<Self>, SignError> {
        Ok(Arc::new(Self {
            client: SignClient::init(config, storage, listener).await?,
        }))
    }

    pub fn start(&self) {
        self.client.start();
    }

    pub fn stop(&self) {
        self.client.stop();
    }

    pub fn foreground(&self) {
        self.client.foreground();
    }

    pub fn client_id(&self) -> String {
        self.client.client_id()
    }

    pub async fn sessions(&self) -> Vec<Session> {
        self.client.sessions().await
    }

    pub async fn connect(
        &self,
        params: ProposalParams,
        intent: Option<DappIntent>,
    ) -> Result<ConnectResult, SignError> {
        self.client.connect(params, intent).await
    }

    pub async fn pair(
        &self,
        uri: String,
    ) -> Result<SessionProposal, SignError> {
        self.client.pair(uri).await
    }

    pub async fn approve(
        &self,
        topic: String,
        namespaces: Namespaces,
        own: Option<Mode>,
    ) -> Result<Session, SignError> {
        self.client.approve(topic, namespaces, own).await
    }

    pub async fn reject(
        &self,
        topic: String,
        reason: Option<String>,
    ) -> Result<(), SignError> {
        self.client.reject(topic, reason).await
    }

    pub async fn request(
        &self,
        topic: String,
        chain_id: String,
        request: RpcRequest,
        deadline_s: Option<u64>,
    ) -> Result<String, SignError> {
        self.client.request(topic, chain_id, request, deadline_s).await
    }

    pub async fn respond(
        &self,
        topic: String,
        id: i64,
        result: Option<String>,
        error: Option<RpcError>,
    ) -> Result<(), SignError> {
        self.client.respond(topic, id, result, error).await
    }

    pub async fn emit(
        &self,
        topic: String,
        chain_id: String,
        event: SessionEventData,
    ) -> Result<(), SignError> {
        self.client.emit(topic, chain_id, event).await
    }

    pub async fn ping(&self, topic: String) -> Result<(), SignError> {
        self.client.ping(topic).await
    }

    pub async fn update(
        &self,
        topic: String,
        namespaces: Namespaces,
    ) -> Result<(), SignError> {
        self.client.update(topic, namespaces).await
    }

    pub async fn extend(
        &self,
        topic: String,
        expiry: Option<u64>,
    ) -> Result<(), SignError> {
        self.client.extend(topic, expiry).await
    }

    pub async fn disconnect(&self, topic: String) -> Result<(), SignError> {
        self.client.disconnect(topic).await
    }

    /// The JSON facade, for React Native bridges.
    pub async fn call(
        &self,
        method: String,
        params_json: String,
    ) -> Result<String, SignError> {
        super::json::call(&self.client, &method, &params_json).await
    }
}
