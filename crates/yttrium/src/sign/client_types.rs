use {
    crate::sign::protocol_types::{
        Metadata, ProposalNamespaces, Relay, SettleNamespace,
    },
    relay_rpc::domain::Topic,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionProposal {
    pub session_proposal_rpc_id: u64,
    pub pairing_topic: Topic,
    pub pairing_sym_key: [u8; 32],
    pub proposer_public_key: [u8; 32],
    pub relays: Vec<crate::sign::protocol_types::Relay>,
    pub required_namespaces: ProposalNamespaces,
    pub optional_namespaces: ProposalNamespaces,
    pub metadata: Metadata,
    pub session_properties: HashMap<String, String>,
    pub scoped_properties: HashMap<String, String>,
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
    pub required_namespaces: ProposalNamespaces,
    pub optional_namespaces: ProposalNamespaces,
    pub session_properties: HashMap<String, String>,
    pub scoped_properties: HashMap<String, String>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct ConnectParams {
    pub optional_namespaces: ProposalNamespaces,
    pub session_properties: Option<HashMap<String, String>>,
    pub scoped_properties: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct ConnectResult {
    pub topic: Topic,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct PairingInfo {
    pub topic: Topic,
    pub uri: String,
    pub sym_key: Vec<u8>,
    pub expiry: u64,
    pub relay: Relay,
    pub active: bool,
    pub methods: Option<Vec<String>>,
    pub peer_metadata: Option<Metadata>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Enum))]
pub enum RejectionReason {
    UserRejected,
    UnsupportedChains,
    UnsupportedMethods,
    UnsupportedAccounts,
    UnsupportedEvents,
}

impl RejectionReason {
    pub fn code(&self) -> i32 {
        match self {
            RejectionReason::UserRejected => 5000,
            RejectionReason::UnsupportedChains => 5001,
            RejectionReason::UnsupportedMethods => 5002,
            RejectionReason::UnsupportedEvents => 5003,
            RejectionReason::UnsupportedAccounts => 5004,
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            RejectionReason::UserRejected => "User rejected",
            RejectionReason::UnsupportedChains => {
                "User disapproved requested chains"
            }
            RejectionReason::UnsupportedMethods => {
                "User disapproved requested json-rpc methods"
            }
            RejectionReason::UnsupportedEvents => {
                "User disapproved requested event types"
            }
            RejectionReason::UnsupportedAccounts => {
                "User disapproved requested accounts"
            }
        }
    }
}

impl From<RejectionReason> for relay_rpc::rpc::ErrorData {
    fn from(reason: RejectionReason) -> Self {
        Self {
            code: reason.code(),
            message: reason.message().to_string(),
            data: None,
        }
    }
}
