use {
    crate::sign::{
        client_types::TransportType,
        protocol_types::{Metadata, ProposalNamespaces, SettleNamespace},
    },
    relay_rpc::domain::Topic,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[derive(uniffi_macros::Record, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionProposalFfi {
    pub id: String,
    pub topic: String,
    pub pairing_sym_key: Vec<u8>,
    pub proposer_public_key: Vec<u8>,
    pub relays: Vec<crate::sign::protocol_types::Relay>,
    pub required_namespaces: std::collections::HashMap<
        String,
        crate::sign::protocol_types::ProposalNamespace,
    >,
    pub optional_namespaces: std::collections::HashMap<
        String,
        crate::sign::protocol_types::ProposalNamespace,
    >,
    pub metadata: crate::sign::protocol_types::Metadata,
    pub session_properties: std::collections::HashMap<String, String>,
    pub scoped_properties: std::collections::HashMap<String, String>,
    pub expiry_timestamp: Option<u64>,
}

#[derive(uniffi_macros::Record, Serialize, Deserialize)]
pub struct SessionRequestRequestFfi {
    pub method: String,
    pub params: String, // JSON string instead of serde_json::Value
    pub expiry: Option<u64>,
}

#[derive(uniffi_macros::Record, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRequestFfi {
    pub chain_id: String,
    pub request: SessionRequestRequestFfi,
}

#[derive(uniffi_macros::Record, Serialize, Deserialize)]
pub struct SessionRequestJsonRpcFfi {
    pub id: u64,
    pub method: String,
    pub params: SessionRequestFfi,
}

#[derive(uniffi_macros::Record, Debug, Serialize, Deserialize)]
pub struct SessionRequestJsonRpcResultResponseFfi {
    pub id: u64,
    pub jsonrpc: String,
    pub result: String, // JSON string instead of serde_json::Value
}

#[derive(uniffi_macros::Record, Debug, Serialize, Deserialize)]
pub struct SessionRequestJsonRpcErrorResponseFfi {
    pub id: u64,
    pub jsonrpc: String,
    pub error: String, // JSON string instead of serde_json::Value
}

#[derive(uniffi_macros::Enum, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SessionRequestJsonRpcResponseFfi {
    Result(SessionRequestJsonRpcResultResponseFfi),
    Error(SessionRequestJsonRpcErrorResponseFfi),
}

#[derive(uniffi_macros::Record, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFfi {
    pub request_id: u64,
    pub session_sym_key: Vec<u8>,
    pub self_public_key: Vec<u8>,
    pub topic: Topic,
    pub expiry: u64,
    pub relay_protocol: String,
    pub relay_data: Option<String>,
    pub controller_key: Option<Vec<u8>>,
    pub self_meta_data: Metadata,
    pub peer_public_key: Option<Vec<u8>>,
    pub peer_meta_data: Option<Metadata>,
    pub session_namespaces: HashMap<String, SettleNamespace>,
    pub required_namespaces: ProposalNamespaces,
    pub optional_namespaces: ProposalNamespaces,
    pub properties: HashMap<String, String>,
    pub scoped_properties: HashMap<String, String>,
    pub is_acknowledged: bool,
    pub pairing_topic: String,
    pub transport_type: Option<TransportType>,
}

#[derive(uniffi_macros::Record, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDataFfi {
    /// Error code.
    pub code: i32,

    /// Error message.
    pub message: String,

    /// Error data, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

#[derive(uniffi_macros::Record, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectParamsFfi {
    pub optional_namespaces: ProposalNamespaces,
    pub session_properties: Option<HashMap<String, String>>,
    pub scoped_properties: Option<HashMap<String, String>>,
    pub metadata: Metadata,
}

#[derive(uniffi_macros::Record, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResultFfi {
    pub topic: Topic,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, uniffi_macros::Record)]
pub struct Pairing {
    pub topic: String,
    pub uri: String,
}
