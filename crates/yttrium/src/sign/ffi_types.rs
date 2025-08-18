use std::collections::HashMap;

use relay_rpc::domain::Topic;
use serde::{Serialize, Deserialize};

use crate::sign::protocol_types::{Metadata, ProposalNamespace, ProposalNamespaces, SettleNamespace};

#[cfg(feature = "uniffi")]
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
    pub optional_namespaces: Option<
        std::collections::HashMap<
            String,
            crate::sign::protocol_types::ProposalNamespace,
        >,
    >,
    pub metadata: crate::sign::protocol_types::Metadata,
    pub session_properties: Option<std::collections::HashMap<String, String>>,
    pub scoped_properties: Option<std::collections::HashMap<String, String>>,
    pub expiry_timestamp: Option<u64>,
}

#[cfg(feature = "uniffi")]
#[derive(uniffi_macros::Record, Serialize, Deserialize)]
pub struct SessionRequestRequestFfi {
    pub method: String,
    pub params: String, // JSON string instead of serde_json::Value
    pub expiry: Option<u64>,
}

#[cfg(feature = "uniffi")]
#[derive(uniffi_macros::Record, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRequestFfi {
    pub chain_id: String,
    pub request: SessionRequestRequestFfi,
}

#[cfg(feature = "uniffi")]
#[derive(uniffi_macros::Record, Serialize, Deserialize)]
pub struct SessionRequestJsonRpcFfi {
    pub id: u64,
    pub method: String,
    pub params: SessionRequestFfi,
}

#[cfg(feature = "uniffi")]
#[derive(uniffi_macros::Record, Debug, Serialize, Deserialize)]
pub struct SessionRequestResponseJsonRpcFfi {
    pub id: u64,
    pub jsonrpc: String,
    pub result: String, // JSON string instead of serde_json::Value
}

#[cfg(feature = "uniffi")]
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
    pub required_namespaces: HashMap<String, ProposalNamespace>,
    pub optional_namespaces: Option<HashMap<String, ProposalNamespace>>,
    pub properties: Option<HashMap<String, String>>,
    pub scoped_properties: Option<HashMap<String, String>>,
    pub is_acknowledged: bool,
    pub pairing_topic: String,
    pub transport_type: Option<TransportType>,
}

#[derive(Debug, Clone)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub request_id: u64,
    pub topic: Topic,
    pub expiry: u64,
    pub relay_protocol: String,
    pub relay_data: Option<String>,
    pub controller_key: Option<[u8; 32]>,
    pub session_sym_key: [u8; 32],
    pub self_public_key: [u8; 32],
    pub self_meta_data: Metadata,
    pub peer_public_key: Option<[u8; 32]>,
    pub peer_meta_data: Option<Metadata>,
    pub session_namespaces: HashMap<String, SettleNamespace>,
    pub required_namespaces: HashMap<String, ProposalNamespace>,
    pub optional_namespaces: Option<HashMap<String, ProposalNamespace>>,
    pub session_properties: Option<HashMap<String, String>>,
    pub scoped_properties: Option<HashMap<String, String>>,
    pub is_acknowledged: bool,
    pub pairing_topic: Topic,
    pub transport_type: Option<TransportType>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
pub enum TransportType {
    Relay,
    LinkMode,
}
