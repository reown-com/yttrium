use {
    relay_rpc::domain::MessageId,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

pub mod methods {
    pub const SESSION_PROPOSE: &str = "wc_sessionPropose";
    pub const SESSION_REQUEST: &str = "wc_sessionRequest";
    pub const SESSION_UPDATE: &str = "wc_sessionUpdate";
    pub const SESSION_EXTEND: &str = "wc_sessionExtend";
    pub const SESSION_EVENT: &str = "wc_sessionEvent";
    pub const SESSION_DELETE: &str = "wc_sessionDelete";
    pub const SESSION_PING: &str = "wc_sessionPing";
    pub const SESSION_SETTLE: &str = "wc_sessionSettle";
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProposalJsonRpc {
    // deserialize number from string (Flutter support)
    pub id: u64,
    pub method: String,
    pub params: Proposal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    pub required_namespaces: ProposalNamespaces,
    pub optional_namespaces: Option<ProposalNamespaces>,
    pub relays: Vec<Relay>,
    pub proposer: Proposer,
    pub session_properties: Option<HashMap<String, String>>,
    pub scoped_properties: Option<HashMap<String, String>>,
    pub expiry_timestamp: Option<u64>,
}

pub type ProposalNamespaces = HashMap<String, ProposalNamespace>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct ProposalNamespace {
    pub chains: Vec<String>,
    pub methods: Vec<String>,
    pub events: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct Relay {
    pub protocol: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Proposer {
    pub public_key: String,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalResultResponseJsonRpc {
    pub id: u64,
    pub jsonrpc: String,
    pub result: ProposalResponse,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalErrorResponseJsonRpc {
    pub id: u64,
    pub jsonrpc: String,
    pub error: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SessionProposalJsonRpcResponse {
    Result(ProposalResultResponseJsonRpc),
    Error(ProposalErrorResponseJsonRpc),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProposalResponse {
    pub relay: Relay,
    pub responder_public_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcRequest {
    pub id: u64,
    pub jsonrpc: String,
    pub method: String,
    pub params: JsonRpcRequestParams,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcRequestParams {
    SessionSettle(SessionSettle),
    SessionPropose(Proposal),
    SessionUpdate(SessionUpdate),
    SessionExtend(SessionExtend),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionSettle {
    pub relay: Relay,
    pub namespaces: SettleNamespaces,
    pub controller: Controller,
    pub expiry: u64,
    pub session_properties: Option<HashMap<String, String>>,
    pub scoped_properties: Option<HashMap<String, String>>,
    // pub session_config: serde_json::Value,
}

pub type SettleNamespaces = HashMap<String, SettleNamespace>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionUpdate {
    pub namespaces: SettleNamespaces,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionUpdateJsonRpc {
    pub id: u64,
    pub jsonrpc: String,
    pub method: String,
    pub params: SessionUpdate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimpleJsonRpcBoolResponse {
    pub id: u64,
    pub jsonrpc: String,
    pub result: bool,
}

#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SettleNamespace {
    pub accounts: Vec<String>,
    pub methods: Vec<String>,
    pub events: Vec<String>,
    pub chains: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Controller {
    pub public_key: String,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct Metadata {
    pub name: String,
    pub description: String,
    pub url: String,
    pub icons: Vec<String>,
    pub verify_url: Option<String>,
    pub redirect: Option<Redirect>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct Redirect {
    pub native: Option<String>,
    pub universal: Option<String>,
    pub link_mode: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionRequestJsonRpc {
    pub id: u64,
    pub method: String,
    pub params: SessionRequest,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionRequest {
    pub chain_id: String,
    pub request: SessionRequestRequest,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionRequestRequest {
    pub method: String,
    pub params: serde_json::Value,
    pub expiry: Option<u64>, // specs say optional
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRequestJsonRpcResultResponse {
    pub id: u64,
    pub jsonrpc: String,
    pub result: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRequestJsonRpcErrorResponse {
    pub id: u64,
    pub jsonrpc: String,
    pub error: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SessionRequestJsonRpcResponse {
    Result(SessionRequestJsonRpcResultResponse),
    Error(SessionRequestJsonRpcErrorResponse),
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct SessionDeleteJsonRpc {
    pub id: u64,
    pub jsonrpc: String,
    pub method: String,
    pub params: SessionDelete,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct SessionDelete {
    pub code: u64,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionExtend {
    pub expiry: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionExtendJsonRpc {
    pub id: u64,
    pub jsonrpc: String,
    pub method: String,
    pub params: SessionExtend,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionEventVO {
    pub name: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventParams {
    pub event: SessionEventVO,
    pub chain_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionEventJsonRpc {
    pub id: u64,
    pub jsonrpc: String,
    pub method: String,
    pub params: EventParams,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum GenericJsonRpcMessage {
    Request(GenericJsonRpcRequest),
    Response(GenericJsonRpcResponse),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenericJsonRpcRequest {
    pub id: MessageId,
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum GenericJsonRpcResponse {
    Success(GenericJsonRpcResponseSuccess),
    Error(GenericJsonRpcResponseError),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenericJsonRpcResponseSuccess {
    pub id: MessageId,
    pub jsonrpc: String,
    pub result: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenericJsonRpcResponseError {
    pub id: MessageId,
    pub jsonrpc: String,
    pub error: serde_json::Value,
}
