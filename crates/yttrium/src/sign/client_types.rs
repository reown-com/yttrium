use {
    crate::sign::protocol_types::{
        Metadata, ProposalNamespaces, Relay, SettleNamespace,
    },
    relay_rpc::domain::Topic,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

pub trait SessionStore: Send + Sync {
    fn add_session(&self, session: Session);
    fn delete_session(&self, topic: String);
    fn get_session(&self, topic: String) -> Option<Session>;
    fn get_all_sessions(&self) -> Vec<Session>;
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
    pub optional_namespaces: Option<ProposalNamespaces>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct ConnectParams {
    pub optional_namespaces: Option<ProposalNamespaces>,
    pub relays: Option<Vec<Relay>>,
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
