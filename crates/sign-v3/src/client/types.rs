use {
    crate::protocol::types::{
        DappIntent, Metadata, Mode, PeerRequirement, Role,
    },
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct SignConfig {
    /// Relay hosts, primary first.
    pub hosts: Vec<String>,
    /// The Relay URL the client JWT is for, when `hosts` point at a proxy. Defaults to the
    /// first host.
    #[serde(default)]
    pub audience: Option<String>,
    pub project_id: String,
    pub metadata: Metadata,
    /// Protocol versions this client speaks; empty means the SDK default.
    #[serde(default)]
    pub versions: Vec<u32>,
    /// Wallet policy: the dApp direction this wallet accepts (PAIR-09).
    #[serde(default)]
    pub require_dapp: Option<PeerRequirement>,
    #[serde(default = "default_prefix")]
    pub storage_prefix: String,
    /// Override of `FIRST_LINE_TIMEOUT` (10 s), for tests.
    #[serde(default)]
    pub first_line_timeout_ms: Option<u64>,
}

fn default_prefix() -> String {
    "wc3:".into()
}

/// Free-form JSON leaves (an RPC method's params, a result, an event's data) are held as JSON
/// text so the typed bindings can carry them as a `String`, and are embedded as real JSON when
/// serialized: the JSON facade sees objects, Swift and Kotlin see a string, as in v2's Kotlin SDK.
pub mod embedded_json {
    use {
        serde::{Deserialize, Deserializer, Serialize, Serializer},
        serde_json::Value,
    };

    pub fn serialize<S: Serializer>(
        s: &str,
        ser: S,
    ) -> Result<S::Ok, S::Error> {
        crate::protocol::ijson::value(s.as_bytes())
            .unwrap_or(Value::Null)
            .serialize(ser)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<String, D::Error> {
        Ok(Value::deserialize(d)?.to_string())
    }

    pub mod option {
        use super::*;

        pub fn serialize<S: Serializer>(
            s: &Option<String>,
            ser: S,
        ) -> Result<S::Ok, S::Error> {
            match s {
                Some(s) => super::serialize(s, ser),
                None => ser.serialize_none(),
            }
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(
            d: D,
        ) -> Result<Option<String>, D::Error> {
            Ok(Option::<Value>::deserialize(d)?.map(|v| v.to_string()))
        }
    }
}

/// A CAIP-25 namespace, as in v2. `accounts` is set by the wallet.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct Namespace {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chains: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<String>>,
    #[serde(default)]
    pub methods: Vec<String>,
    #[serde(default)]
    pub events: Vec<String>,
}

/// Namespaces keyed by namespace, e.g. `eip155`.
pub type Namespaces = HashMap<String, Namespace>;

/// The proposal's params: v2's `wc_sessionPropose` params minus the transport fields. The dApp's
/// metadata is added from the config.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct ProposalParams {
    #[serde(default)]
    pub required_namespaces: Namespaces,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional_namespaces: Option<Namespaces>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_properties: Option<HashMap<String, String>>,
}

/// An RPC call inside a session request, e.g. `personal_sign`. `params` is the method's own
/// params, which depend on the method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RpcRequest {
    pub method: String,
    #[serde(with = "embedded_json")]
    pub params: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

/// A response to a session request: `result` or `error`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RpcResponse {
    pub id: i64,
    #[serde(
        default,
        with = "embedded_json::option",
        skip_serializing_if = "Option::is_none"
    )]
    pub result: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

/// A session event, e.g. `chainChanged`. `data` depends on the event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SessionEventData {
    pub name: String,
    #[serde(with = "embedded_json")]
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// The connectionId.
    pub topic: String,
    pub role: Role,
    pub version: u32,
    pub namespaces: Namespaces,
    pub peer_metadata: Metadata,
    pub dapp_mode: Mode,
    pub wallet_mode: Mode,
    pub expiry: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct SessionProposal {
    /// The connection request's topic; pass it to `approve` or `reject`.
    pub id: String,
    /// The proposal's JSON-RPC id, for v2-style numeric proposal ids.
    pub proposal_id: i64,
    pub params: ProposalParams,
    pub metadata: Metadata,
    pub intent: DappIntent,
    pub versions: Vec<u32>,
    pub version: u32,
    pub sealable: bool,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct ConnectResult {
    pub uri: String,
    pub topic: String,
}

/// Events delivered to the host's listener.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum SignEvent {
    SessionProposal {
        proposal: SessionProposal,
    },
    SessionConnect {
        session: Session,
    },
    ProposalRejected {
        topic: String,
        reason: Option<String>,
    },
    /// `reason`: `expired` past the answer window, `replaced` by a newer `connect`, or
    /// `cursor_expired`.
    ProposalExpire {
        topic: String,
        reason: String,
    },
    ProposalFailed {
        topic: String,
        reason: String,
    },
    SessionRequest {
        id: i64,
        topic: String,
        chain_id: String,
        request: RpcRequest,
    },
    /// A response with no pending call, e.g. after a restart, with the stored request it answers.
    SessionResponse {
        topic: String,
        response: RpcResponse,
        request: Option<RpcRequest>,
    },
    SessionEvent {
        id: i64,
        topic: String,
        chain_id: String,
        event: SessionEventData,
    },
    SessionUpdate {
        id: i64,
        topic: String,
        namespaces: Namespaces,
    },
    SessionExtend {
        id: i64,
        topic: String,
        expiry: u64,
    },
    SessionPing {
        id: i64,
        topic: String,
    },
    SessionDelete {
        topic: String,
        reason: String,
        verified: bool,
    },
    /// `id` is the JSON-RPC id of the expired request, when it was one of ours.
    MessageExpired {
        topic: String,
        seq: u64,
        id: Option<i64>,
    },
    MessageInvalid {
        topic: String,
        reason: String,
    },
    CursorExpired,
    ClockSkew {
        skew_s: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum SignError {
    #[error("{code}: {message}")]
    Sdk { code: String, message: String },
}

impl SignError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        SignError::Sdk { code: code.into(), message: message.into() }
    }
    pub fn code(&self) -> &str {
        match self {
            SignError::Sdk { code, .. } => code,
        }
    }
}

impl From<crate::host::StorageError> for SignError {
    fn from(e: crate::host::StorageError) -> Self {
        SignError::new("storage", e.to_string())
    }
}

impl From<crate::protocol::ProtocolError> for SignError {
    fn from(e: crate::protocol::ProtocolError) -> Self {
        SignError::new("invalid_peer", e.to_string())
    }
}

/// errors.md: map a final Relay status to an SDK error code.
impl From<crate::transport::relay::RelayError> for SignError {
    fn from(e: crate::transport::relay::RelayError) -> Self {
        use crate::transport::relay::RelayError;
        match e {
            RelayError::Network(m) => SignError::new("network", m),
            RelayError::Http { status, error } => {
                let code = match (status, error.as_str()) {
                    (409, "seq_conflict") => "seq_reused",
                    (409, "connection_resolved") => "request_resolved",
                    (409, _) => "conflict",
                    (410, _) => "connection_closed",
                    (422, "incompatible_intent") => "incompatible_intent",
                    (422, "unsupported_version") => "unsupported_version",
                    (422, _) => "invalid_request",
                    (429, "event_log_full") => "backpressure",
                    (429, _) => "rate_limited",
                    (403, _) => "forbidden",
                    (404, _) => "not_found",
                    (401, _) => "unauthorized",
                    _ => "relay_error",
                };
                SignError::new(code, format!("{status} {error}"))
            }
        }
    }
}

#[cfg(test)]
mod json_shape_tests {
    use {super::*, serde_json::json};

    #[test]
    fn events_serialize_as_whole_objects_with_camel_case_fields() {
        let e = SignEvent::SessionRequest {
            id: 7,
            topic: "c_1".into(),
            chain_id: "eip155:1".into(),
            request: RpcRequest {
                method: "personal_sign".into(),
                params: r#"["0xdead","0xabc"]"#.into(),
            },
        };
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            json!({
                "type": "session_request",
                "id": 7,
                "topic": "c_1",
                "chainId": "eip155:1",
                "request": { "method": "personal_sign", "params": ["0xdead", "0xabc"] }
            })
        );
        let r = SignEvent::SessionResponse {
            topic: "c_1".into(),
            response: RpcResponse {
                id: 7,
                result: Some(r#""0xsig""#.into()),
                error: None,
            },
            request: None,
        };
        assert_eq!(
            serde_json::to_value(&r).unwrap(),
            json!({ "type": "session_response", "topic": "c_1", "response": { "id": 7, "result": "0xsig" }, "request": null })
        );
        assert_eq!(
            serde_json::to_value(SignEvent::ClockSkew { skew_s: 400 }).unwrap(),
            json!({ "type": "clock_skew", "skewS": 400 })
        );
    }

    #[test]
    fn records_carry_real_namespaces_and_parse_from_objects() {
        let ns: Namespaces = serde_json::from_value(json!({
            "eip155": { "chains": ["eip155:1"], "accounts": ["eip155:1:0xabc"], "methods": ["personal_sign"], "events": [] }
        }))
        .unwrap();
        assert_eq!(
            ns["eip155"].accounts.as_deref(),
            Some(&["eip155:1:0xabc".to_string()][..])
        );
        let req: RpcRequest = serde_json::from_value(json!({ "method": "eth_sendTransaction", "params": [{ "to": "0x1", "value": "0x0" }] })).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&req.params).unwrap(),
            json!([{ "to": "0x1", "value": "0x0" }])
        );
        let p: ProposalParams = serde_json::from_value(json!({ "requiredNamespaces": ns, "sessionProperties": { "caip154-mandatory": "true" } })).unwrap();
        assert_eq!(p.required_namespaces, ns);
        assert!(
            serde_json::to_value(&p)
                .unwrap()
                .get("optionalNamespaces")
                .is_none()
        );
    }
}
