use relay_rpc::domain::Topic;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    pub required_namespaces: serde_json::Value,
    pub optional_namespaces: serde_json::Value,
    pub relays: Vec<Relay>,
    pub proposer: Proposer,
    pub expiry_timestamp: u64,
    pub pairing_topic: Topic,
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Relay {
    pub protocol: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proposer {
    pub public_key: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalResponse {
    pub relay: Relay,
    pub responder_public_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionSettle {
    pub relay: Relay,
    pub namespaces: serde_json::Value,
    pub controller: Controller,
    pub expiry_timestamp: u64,
    pub session_properties: serde_json::Value,
    pub scoped_properties: serde_json::Value,
    pub session_config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Controller {
    pub public_key: String,
    pub metadata: serde_json::Value,
}
