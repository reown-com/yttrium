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
    pub jsonrpc: String,
    pub method: String,
    pub params: Proposal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    pub required_namespaces: ProposalNamespaces,
    // Must be at least `{}`: https://reown-inc.slack.com/archives/C04DB2EAHE3/p1761048078934459?thread_ts=1761047215.003739&cid=C04DB2EAHE3
    #[serde(default)]
    pub optional_namespaces: ProposalNamespaces,
    pub relays: Vec<Relay>,
    pub proposer: Proposer,
    // skip serializing properties: https://reown-inc.slack.com/archives/C04DB2EAHE3/p1761047855481929?thread_ts=1761047215.003739&cid=C04DB2EAHE3
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub session_properties: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub scoped_properties: HashMap<String, String>,
    // also skip serializing expiry
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub session_properties: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub scoped_properties: HashMap<String, String>,
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
    pub jsonrpc: String,
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
#[serde(untagged)]
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
#[serde(untagged)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_minimal_proposal() -> Proposal {
        let mut required_namespaces = HashMap::new();
        required_namespaces.insert(
            "eip155".to_string(),
            ProposalNamespace {
                chains: vec!["eip155:1".to_string()],
                methods: vec!["eth_sendTransaction".to_string()],
                events: vec!["chainChanged".to_string()],
            },
        );

        Proposal {
            required_namespaces,
            optional_namespaces: HashMap::new(),
            relays: vec![Relay { protocol: "irn".to_string() }],
            proposer: Proposer {
                public_key: "test_public_key".to_string(),
                metadata: Metadata {
                    name: "Test Wallet".to_string(),
                    description: "Test Description".to_string(),
                    url: "https://test.com".to_string(),
                    icons: vec!["https://test.com/icon.png".to_string()],
                    verify_url: None,
                    redirect: None,
                },
            },
            session_properties: HashMap::new(),
            scoped_properties: HashMap::new(),
            expiry_timestamp: None,
        }
    }

    #[test]
    fn test_deserialize_optional_namespaces_undefined() {
        let json = r#"{
            "requiredNamespaces": {
                "eip155": {
                    "chains": ["eip155:1"],
                    "methods": ["eth_sendTransaction"],
                    "events": ["chainChanged"]
                }
            },
            "relays": [{"protocol": "irn"}],
            "proposer": {
                "publicKey": "test_public_key",
                "metadata": {
                    "name": "Test Wallet",
                    "description": "Test Description",
                    "url": "https://test.com",
                    "icons": ["https://test.com/icon.png"]
                }
            }
        }"#;

        let proposal: Proposal = serde_json::from_str(json).unwrap();
        assert_eq!(proposal.optional_namespaces.len(), 0);
        assert!(proposal.optional_namespaces.is_empty());
    }

    #[test]
    fn test_deserialize_optional_namespaces_null() {
        let json = r#"{
            "requiredNamespaces": {
                "eip155": {
                    "chains": ["eip155:1"],
                    "methods": ["eth_sendTransaction"],
                    "events": ["chainChanged"]
                }
            },
            "optionalNamespaces": null,
            "relays": [{"protocol": "irn"}],
            "proposer": {
                "publicKey": "test_public_key",
                "metadata": {
                    "name": "Test Wallet",
                    "description": "Test Description",
                    "url": "https://test.com",
                    "icons": ["https://test.com/icon.png"]
                }
            }
        }"#;

        let proposal: Proposal = serde_json::from_str(json).unwrap();
        assert_eq!(proposal.optional_namespaces.len(), 0);
        assert!(proposal.optional_namespaces.is_empty());
    }

    #[test]
    fn test_deserialize_session_properties_undefined() {
        let json = r#"{
            "requiredNamespaces": {
                "eip155": {
                    "chains": ["eip155:1"],
                    "methods": ["eth_sendTransaction"],
                    "events": ["chainChanged"]
                }
            },
            "relays": [{"protocol": "irn"}],
            "proposer": {
                "publicKey": "test_public_key",
                "metadata": {
                    "name": "Test Wallet",
                    "description": "Test Description",
                    "url": "https://test.com",
                    "icons": ["https://test.com/icon.png"]
                }
            }
        }"#;

        let proposal: Proposal = serde_json::from_str(json).unwrap();
        assert_eq!(proposal.session_properties.len(), 0);
        assert!(proposal.session_properties.is_empty());
    }

    #[test]
    fn test_deserialize_session_properties_null() {
        let json = r#"{
            "requiredNamespaces": {
                "eip155": {
                    "chains": ["eip155:1"],
                    "methods": ["eth_sendTransaction"],
                    "events": ["chainChanged"]
                }
            },
            "sessionProperties": null,
            "relays": [{"protocol": "irn"}],
            "proposer": {
                "publicKey": "test_public_key",
                "metadata": {
                    "name": "Test Wallet",
                    "description": "Test Description",
                    "url": "https://test.com",
                    "icons": ["https://test.com/icon.png"]
                }
            }
        }"#;

        let proposal: Proposal = serde_json::from_str(json).unwrap();
        assert_eq!(proposal.session_properties.len(), 0);
        assert!(proposal.session_properties.is_empty());
    }

    #[test]
    fn test_deserialize_scoped_properties_undefined() {
        let json = r#"{
            "requiredNamespaces": {
                "eip155": {
                    "chains": ["eip155:1"],
                    "methods": ["eth_sendTransaction"],
                    "events": ["chainChanged"]
                }
            },
            "relays": [{"protocol": "irn"}],
            "proposer": {
                "publicKey": "test_public_key",
                "metadata": {
                    "name": "Test Wallet",
                    "description": "Test Description",
                    "url": "https://test.com",
                    "icons": ["https://test.com/icon.png"]
                }
            }
        }"#;

        let proposal: Proposal = serde_json::from_str(json).unwrap();
        assert_eq!(proposal.scoped_properties.len(), 0);
        assert!(proposal.scoped_properties.is_empty());
    }

    #[test]
    fn test_deserialize_scoped_properties_null() {
        let json = r#"{
            "requiredNamespaces": {
                "eip155": {
                    "chains": ["eip155:1"],
                    "methods": ["eth_sendTransaction"],
                    "events": ["chainChanged"]
                }
            },
            "scopedProperties": null,
            "relays": [{"protocol": "irn"}],
            "proposer": {
                "publicKey": "test_public_key",
                "metadata": {
                    "name": "Test Wallet",
                    "description": "Test Description",
                    "url": "https://test.com",
                    "icons": ["https://test.com/icon.png"]
                }
            }
        }"#;

        let proposal: Proposal = serde_json::from_str(json).unwrap();
        assert_eq!(proposal.scoped_properties.len(), 0);
        assert!(proposal.scoped_properties.is_empty());
    }

    #[test]
    fn test_deserialize_expiry_timestamp_undefined() {
        let json = r#"{
            "requiredNamespaces": {
                "eip155": {
                    "chains": ["eip155:1"],
                    "methods": ["eth_sendTransaction"],
                    "events": ["chainChanged"]
                }
            },
            "relays": [{"protocol": "irn"}],
            "proposer": {
                "publicKey": "test_public_key",
                "metadata": {
                    "name": "Test Wallet",
                    "description": "Test Description",
                    "url": "https://test.com",
                    "icons": ["https://test.com/icon.png"]
                }
            }
        }"#;

        let proposal: Proposal = serde_json::from_str(json).unwrap();
        assert_eq!(proposal.expiry_timestamp, None);
    }

    #[test]
    fn test_deserialize_expiry_timestamp_null() {
        let json = r#"{
            "requiredNamespaces": {
                "eip155": {
                    "chains": ["eip155:1"],
                    "methods": ["eth_sendTransaction"],
                    "events": ["chainChanged"]
                }
            },
            "expiryTimestamp": null,
            "relays": [{"protocol": "irn"}],
            "proposer": {
                "publicKey": "test_public_key",
                "metadata": {
                    "name": "Test Wallet",
                    "description": "Test Description",
                    "url": "https://test.com",
                    "icons": ["https://test.com/icon.png"]
                }
            }
        }"#;

        let proposal: Proposal = serde_json::from_str(json).unwrap();
        assert_eq!(proposal.expiry_timestamp, None);
    }

    #[test]
    fn test_serialize_empty_required_namespaces() {
        let proposal = Proposal {
            required_namespaces: HashMap::new(),
            optional_namespaces: HashMap::new(),
            relays: vec![Relay { protocol: "irn".to_string() }],
            proposer: Proposer {
                public_key: "test_public_key".to_string(),
                metadata: Metadata {
                    name: "Test Wallet".to_string(),
                    description: "Test Description".to_string(),
                    url: "https://test.com".to_string(),
                    icons: vec!["https://test.com/icon.png".to_string()],
                    verify_url: None,
                    redirect: None,
                },
            },
            session_properties: HashMap::new(),
            scoped_properties: HashMap::new(),
            expiry_timestamp: None,
        };

        let serialized = serde_json::to_value(&proposal).unwrap();
        assert_eq!(serialized["requiredNamespaces"], serde_json::json!({}));
    }

    #[test]
    fn test_serialize_empty_optional_namespaces() {
        let proposal = create_minimal_proposal();
        let serialized = serde_json::to_value(&proposal).unwrap();
        assert_eq!(serialized["optionalNamespaces"], serde_json::json!({}));
    }

    #[test]
    fn test_serialize_empty_session_properties_as_undefined() {
        let proposal = create_minimal_proposal();
        let serialized = serde_json::to_string(&proposal).unwrap();

        // Should not contain sessionProperties field
        assert!(!serialized.contains("sessionProperties"));
    }

    #[test]
    fn test_serialize_empty_scoped_properties_as_undefined() {
        let proposal = create_minimal_proposal();
        let serialized = serde_json::to_string(&proposal).unwrap();

        // Should not contain scopedProperties field
        assert!(!serialized.contains("scopedProperties"));
    }

    #[test]
    fn test_serialize_none_expiry_timestamp_as_undefined() {
        let proposal = create_minimal_proposal();
        let serialized = serde_json::to_string(&proposal).unwrap();

        // Should not contain expiryTimestamp field
        assert!(!serialized.contains("expiryTimestamp"));
    }

    #[test]
    fn test_serialize_with_session_properties() {
        let mut proposal = create_minimal_proposal();
        proposal
            .session_properties
            .insert("key1".to_string(), "value1".to_string());

        let serialized = serde_json::to_string(&proposal).unwrap();

        // Should contain sessionProperties field when not empty
        assert!(serialized.contains("sessionProperties"));
    }

    #[test]
    fn test_serialize_with_scoped_properties() {
        let mut proposal = create_minimal_proposal();
        proposal
            .scoped_properties
            .insert("key1".to_string(), "value1".to_string());

        let serialized = serde_json::to_string(&proposal).unwrap();

        // Should contain scopedProperties field when not empty
        assert!(serialized.contains("scopedProperties"));
    }

    #[test]
    fn test_serialize_with_expiry_timestamp() {
        let mut proposal = create_minimal_proposal();
        proposal.expiry_timestamp = Some(1234567890);

        let serialized = serde_json::to_string(&proposal).unwrap();

        // Should contain expiryTimestamp field when Some
        assert!(serialized.contains("expiryTimestamp"));
        assert!(serialized.contains("1234567890"));
    }

    #[test]
    fn test_roundtrip_serialization() {
        let mut proposal = create_minimal_proposal();
        proposal.optional_namespaces.insert(
            "solana".to_string(),
            ProposalNamespace {
                chains: vec!["solana:mainnet".to_string()],
                methods: vec!["solana_signTransaction".to_string()],
                events: vec!["accountChanged".to_string()],
            },
        );
        proposal
            .session_properties
            .insert("theme".to_string(), "dark".to_string());
        proposal
            .scoped_properties
            .insert("scope1".to_string(), "value1".to_string());
        proposal.expiry_timestamp = Some(9999999999);

        // Serialize to JSON
        let json = serde_json::to_string(&proposal).unwrap();

        // Deserialize back
        let deserialized: Proposal = serde_json::from_str(&json).unwrap();

        // Verify all fields
        assert_eq!(
            deserialized.required_namespaces.len(),
            proposal.required_namespaces.len()
        );
        assert_eq!(
            deserialized.optional_namespaces.len(),
            proposal.optional_namespaces.len()
        );
        assert_eq!(
            deserialized.session_properties.len(),
            proposal.session_properties.len()
        );
        assert_eq!(
            deserialized.scoped_properties.len(),
            proposal.scoped_properties.len()
        );
        assert_eq!(deserialized.expiry_timestamp, proposal.expiry_timestamp);
    }

    // SessionSettle tests

    fn create_minimal_session_settle() -> SessionSettle {
        let mut namespaces = HashMap::new();
        namespaces.insert(
            "eip155".to_string(),
            SettleNamespace {
                accounts: vec!["eip155:1:0x1234567890abcdef".to_string()],
                methods: vec!["eth_sendTransaction".to_string()],
                events: vec!["chainChanged".to_string()],
                chains: vec!["eip155:1".to_string()],
            },
        );

        SessionSettle {
            relay: Relay { protocol: "irn".to_string() },
            namespaces,
            controller: Controller {
                public_key: "controller_public_key".to_string(),
                metadata: Metadata {
                    name: "Controller Wallet".to_string(),
                    description: "Controller Description".to_string(),
                    url: "https://controller.com".to_string(),
                    icons: vec!["https://controller.com/icon.png".to_string()],
                    verify_url: None,
                    redirect: None,
                },
            },
            expiry: 1234567890,
            session_properties: HashMap::new(),
            scoped_properties: HashMap::new(),
        }
    }

    #[test]
    fn test_session_settle_deserialize_session_properties_undefined() {
        let json = r#"{
            "relay": {"protocol": "irn"},
            "namespaces": {
                "eip155": {
                    "accounts": ["eip155:1:0x1234567890abcdef"],
                    "methods": ["eth_sendTransaction"],
                    "events": ["chainChanged"],
                    "chains": ["eip155:1"]
                }
            },
            "controller": {
                "publicKey": "controller_public_key",
                "metadata": {
                    "name": "Controller Wallet",
                    "description": "Controller Description",
                    "url": "https://controller.com",
                    "icons": ["https://controller.com/icon.png"]
                }
            },
            "expiry": 1234567890
        }"#;

        let session_settle: SessionSettle = serde_json::from_str(json).unwrap();
        assert_eq!(session_settle.session_properties.len(), 0);
        assert!(session_settle.session_properties.is_empty());
    }

    #[test]
    fn test_session_settle_deserialize_session_properties_null() {
        let json = r#"{
            "relay": {"protocol": "irn"},
            "namespaces": {
                "eip155": {
                    "accounts": ["eip155:1:0x1234567890abcdef"],
                    "methods": ["eth_sendTransaction"],
                    "events": ["chainChanged"],
                    "chains": ["eip155:1"]
                }
            },
            "controller": {
                "publicKey": "controller_public_key",
                "metadata": {
                    "name": "Controller Wallet",
                    "description": "Controller Description",
                    "url": "https://controller.com",
                    "icons": ["https://controller.com/icon.png"]
                }
            },
            "expiry": 1234567890,
            "sessionProperties": null
        }"#;

        let session_settle: SessionSettle = serde_json::from_str(json).unwrap();
        assert_eq!(session_settle.session_properties.len(), 0);
        assert!(session_settle.session_properties.is_empty());
    }

    #[test]
    fn test_session_settle_deserialize_scoped_properties_undefined() {
        let json = r#"{
            "relay": {"protocol": "irn"},
            "namespaces": {
                "eip155": {
                    "accounts": ["eip155:1:0x1234567890abcdef"],
                    "methods": ["eth_sendTransaction"],
                    "events": ["chainChanged"],
                    "chains": ["eip155:1"]
                }
            },
            "controller": {
                "publicKey": "controller_public_key",
                "metadata": {
                    "name": "Controller Wallet",
                    "description": "Controller Description",
                    "url": "https://controller.com",
                    "icons": ["https://controller.com/icon.png"]
                }
            },
            "expiry": 1234567890
        }"#;

        let session_settle: SessionSettle = serde_json::from_str(json).unwrap();
        assert_eq!(session_settle.scoped_properties.len(), 0);
        assert!(session_settle.scoped_properties.is_empty());
    }

    #[test]
    fn test_session_settle_deserialize_scoped_properties_null() {
        let json = r#"{
            "relay": {"protocol": "irn"},
            "namespaces": {
                "eip155": {
                    "accounts": ["eip155:1:0x1234567890abcdef"],
                    "methods": ["eth_sendTransaction"],
                    "events": ["chainChanged"],
                    "chains": ["eip155:1"]
                }
            },
            "controller": {
                "publicKey": "controller_public_key",
                "metadata": {
                    "name": "Controller Wallet",
                    "description": "Controller Description",
                    "url": "https://controller.com",
                    "icons": ["https://controller.com/icon.png"]
                }
            },
            "expiry": 1234567890,
            "scopedProperties": null
        }"#;

        let session_settle: SessionSettle = serde_json::from_str(json).unwrap();
        assert_eq!(session_settle.scoped_properties.len(), 0);
        assert!(session_settle.scoped_properties.is_empty());
    }

    #[test]
    fn test_session_settle_serialize_empty_session_properties_as_undefined() {
        let session_settle = create_minimal_session_settle();
        let serialized = serde_json::to_string(&session_settle).unwrap();

        // Should not contain sessionProperties field
        assert!(!serialized.contains("sessionProperties"));
    }

    #[test]
    fn test_session_settle_serialize_empty_scoped_properties_as_undefined() {
        let session_settle = create_minimal_session_settle();
        let serialized = serde_json::to_string(&session_settle).unwrap();

        // Should not contain scopedProperties field
        assert!(!serialized.contains("scopedProperties"));
    }

    #[test]
    fn test_session_settle_serialize_with_session_properties() {
        let mut session_settle = create_minimal_session_settle();
        session_settle
            .session_properties
            .insert("key1".to_string(), "value1".to_string());

        let serialized = serde_json::to_string(&session_settle).unwrap();

        // Should contain sessionProperties field when not empty
        assert!(serialized.contains("sessionProperties"));
        assert!(serialized.contains("key1"));
        assert!(serialized.contains("value1"));
    }

    #[test]
    fn test_session_settle_serialize_with_scoped_properties() {
        let mut session_settle = create_minimal_session_settle();
        session_settle
            .scoped_properties
            .insert("scope1".to_string(), "scopeValue1".to_string());

        let serialized = serde_json::to_string(&session_settle).unwrap();

        // Should contain scopedProperties field when not empty
        assert!(serialized.contains("scopedProperties"));
        assert!(serialized.contains("scope1"));
        assert!(serialized.contains("scopeValue1"));
    }

    #[test]
    fn test_session_settle_roundtrip_serialization() {
        let mut session_settle = create_minimal_session_settle();
        session_settle
            .session_properties
            .insert("theme".to_string(), "dark".to_string());
        session_settle
            .scoped_properties
            .insert("scope1".to_string(), "value1".to_string());

        // Serialize to JSON
        let json = serde_json::to_string(&session_settle).unwrap();

        // Deserialize back
        let deserialized: SessionSettle = serde_json::from_str(&json).unwrap();

        // Verify all fields
        assert_eq!(
            deserialized.namespaces.len(),
            session_settle.namespaces.len()
        );
        assert_eq!(
            deserialized.session_properties.len(),
            session_settle.session_properties.len()
        );
        assert_eq!(
            deserialized.scoped_properties.len(),
            session_settle.scoped_properties.len()
        );
        assert_eq!(deserialized.expiry, session_settle.expiry);
        assert_eq!(
            deserialized.session_properties.get("theme"),
            Some(&"dark".to_string())
        );
        assert_eq!(
            deserialized.scoped_properties.get("scope1"),
            Some(&"value1".to_string())
        );
    }
}
