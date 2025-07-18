use relay_rpc::domain::Topic;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    pub required_namespaces: ProposalNamespaces,
    pub optional_namespaces: ProposalNamespaces,
    pub relays: Vec<Relay>,
    pub proposer: Proposer,
    pub expiry_timestamp: u64,
    pub pairing_topic: Topic,
    pub id: u64,
}

pub type ProposalNamespaces = HashMap<String, ProposalNamespace>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProposalNamespace {
    pub chains: Vec<String>,
    pub methods: Vec<String>,
    pub events: Vec<String>,
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
    pub namespaces: SettleNamespaces,
    pub controller: Controller,
    pub expiry_timestamp: u64,
    pub session_properties: serde_json::Value,
    pub scoped_properties: serde_json::Value,
    pub session_config: serde_json::Value,
}

pub type SettleNamespaces = HashMap<String, SettleNamespace>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SettleNamespace {
    pub accounts: Vec<String>,
    pub methods: Vec<String>,
    pub events: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Controller {
    pub public_key: String,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub name: String,
    pub description: String,
    pub url: String,
    pub icons: Vec<String>,
}
