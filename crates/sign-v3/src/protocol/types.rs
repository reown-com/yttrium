use {
    serde::{Deserialize, Serialize},
    serde_json::Value,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Monitored,
    Sealed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[serde(rename_all = "lowercase")]
pub enum PeerRequirement {
    None,
    Monitored,
    Sealed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Dapp,
    Wallet,
}

impl Role {
    pub fn peer(self) -> Role {
        match self {
            Role::Dapp => Role::Wallet,
            Role::Wallet => Role::Dapp,
        }
    }
}

/// The dApp's intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct DappIntent {
    pub own: Mode,
    pub peer: PeerRequirement,
}

/// The wallet's intent carries only its own direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct WalletIntent {
    pub own: Mode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Metadata {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub icons: Vec<String>,
}

pub mod tags {
    pub const PROPOSAL: u32 = 1130;
    pub const APPROVAL: u32 = 1131;
    pub const UPDATE: u32 = 1104;
    pub const UPDATE_RESPONSE: u32 = 1105;
    pub const EXTEND: u32 = 1106;
    pub const EXTEND_RESPONSE: u32 = 1107;
    pub const REQUEST: u32 = 1108;
    pub const REQUEST_RESPONSE: u32 = 1109;
    pub const EVENT: u32 = 1110;
    pub const EVENT_RESPONSE: u32 = 1111;
    pub const PING: u32 = 1114;
    pub const PING_RESPONSE: u32 = 1115;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sender {
    Dapp,
    Wallet,
    Either,
}

/// Messages, Tags table: who may send each tag and whether its payload may be sealed.
pub fn tag_info(tag: u32) -> Option<(Sender, bool)> {
    use tags::*;
    Some(match tag {
        PROPOSAL => (Sender::Dapp, false),
        APPROVAL => (Sender::Wallet, false),
        UPDATE | EXTEND => (Sender::Wallet, false),
        UPDATE_RESPONSE | EXTEND_RESPONSE => (Sender::Dapp, false),
        PING | PING_RESPONSE => (Sender::Either, false),
        REQUEST | REQUEST_RESPONSE | EVENT | EVENT_RESPONSE => {
            (Sender::Either, true)
        }
        _ => return None,
    })
}

pub fn may_role_send(role: Role, tag: u32) -> bool {
    match tag_info(tag) {
        Some((Sender::Either, _)) => true,
        Some((Sender::Dapp, _)) => role == Role::Dapp,
        Some((Sender::Wallet, _)) => role == Role::Wallet,
        None => false,
    }
}

pub fn is_sealable(tag: u32) -> bool {
    matches!(tag_info(tag), Some((_, true)))
}

pub fn method_tag(method: &str) -> Option<u32> {
    use tags::*;
    Some(match method {
        "wc_sessionCreate" => PROPOSAL,
        "wc_sessionUpdate" => UPDATE,
        "wc_sessionExtend" => EXTEND,
        "wc_sessionRequest" => REQUEST,
        "wc_sessionEvent" => EVENT,
        "wc_sessionPing" => PING,
        _ => return None,
    })
}

/// A signed object: `{ payload: base64url(I-JSON bytes), sig: base64url }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedObject {
    pub payload: String,
    pub sig: String,
}

/// A message on the wire, each part base64url.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireMessage {
    pub plain: String,
    pub payload: String,
    pub sig: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Plain {
    pub tag: u32,
    pub seq: u64,
    pub sealed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BindingProofPayload {
    pub versions: Vec<u32>,
    pub intent: DappIntent,
    #[serde(rename = "pairingPub")]
    pub pairing_pub: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BindingPayload {
    #[serde(rename = "connectionId")]
    pub connection_id: String,
    pub version: u32,
    #[serde(rename = "pairingPub")]
    pub pairing_pub: String,
    pub intent: WalletIntent,
    pub expiry: u64,
    pub approval: WireMessage,
}

#[derive(Debug, Clone, Serialize)]
pub struct RejectionPayload {
    #[serde(rename = "connectionId")]
    pub connection_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Events, as the Relay delivers them. Unknown types are skipped but still acknowledged.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum RelayEvent {
    #[serde(rename = "message.received")]
    MessageReceived {
        #[serde(rename = "connectionId")]
        connection_id: String,
        message: WireMessage,
    },
    #[serde(rename = "connection.established")]
    ConnectionEstablished {
        #[serde(rename = "connectionId")]
        connection_id: String,
        binding: SignedObject,
    },
    #[serde(rename = "connection.rejected")]
    ConnectionRejected {
        #[serde(rename = "connectionId")]
        connection_id: String,
        rejection: SignedObject,
    },
    #[serde(rename = "connection.closed")]
    ConnectionClosed {
        #[serde(rename = "connectionId")]
        connection_id: String,
        reason: String,
        close: Option<String>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Backoff {
    pub base_ms: u64,
    pub max_ms: u64,
}

/// The poll's first-line config (receive-path.md), after local bounds.
#[derive(Debug, Clone)]
pub struct PollConfig {
    pub heartbeats_ms: u64,
    pub repoll_delay_ms: u64,
    pub backoff: Backoff,
    pub hosts: Vec<String>,
    pub foreground_stale_ms: u64,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            heartbeats_ms: 3000,
            repoll_delay_ms: 0,
            backoff: Backoff { base_ms: 1000, max_ms: 60_000 },
            hosts: vec![],
            foreground_stale_ms: 6000,
        }
    }
}

/// A JSON-RPC 2.0 object, kept as a value so unknown fields survive.
pub type JsonRpc = Value;
