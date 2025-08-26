use {
    relay_rpc::domain::{MessageId, Topic},
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalJsonRpc {
    // deserialize number from string (Flutter support)
    pub id: MessageId,
    pub method: String,
    pub params: Proposal,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionProposal {
    pub session_proposal_rpc_id: u64,
    pub pairing_topic: Topic,
    pub pairing_sym_key: [u8; 32],
    pub proposer_public_key: [u8; 32],
    pub relays: Vec<crate::sign::protocol_types::Relay>,
    pub required_namespaces: ProposalNamespaces,
    pub optional_namespaces: Option<ProposalNamespaces>,
    pub metadata: Metadata,
    pub session_properties: Option<HashMap<String, String>>,
    pub scoped_properties: Option<HashMap<String, String>>,
    pub expiry_timestamp: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct Relay {
    pub protocol: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proposer {
    pub public_key: String,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalResponseJsonRpc {
    pub id: u64,
    pub jsonrpc: String,
    pub result: ProposalResponse,
}

#[derive(Debug, Serialize, Deserialize)]
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
    SessionPropose(SessionProposal),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionSettle {
    pub relay: Relay,
    pub namespaces: SettleNamespaces,
    pub controller: Controller,
    pub expiry: u64,
    pub session_properties: serde_json::Value,
    pub scoped_properties: serde_json::Value,
    // pub session_config: serde_json::Value,
}

pub type SettleNamespaces = HashMap<String, SettleNamespace>;

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

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum SessionRequestJsonRpcResponse {
    Result(SessionRequestJsonRpcResultResponse),
    Error(SessionRequestJsonRpcErrorResponse),
}
