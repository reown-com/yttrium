
#[cfg(feature = "uniffi")]
#[derive(uniffi_macros::Record, Debug)]
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
#[derive(uniffi_macros::Record)]
pub struct SessionFfi {
    pub session_sym_key: Vec<u8>,
}

#[cfg(feature = "uniffi")]
#[derive(uniffi_macros::Record)]
pub struct SessionRequestRequestFfi {
    pub method: String,
    pub params: String, // JSON string instead of serde_json::Value
    pub expiry: Option<u64>,
}

#[cfg(feature = "uniffi")]
#[derive(uniffi_macros::Record)]
pub struct SessionRequestFfi {
    pub chain_id: String,
    pub request: SessionRequestRequestFfi,
}

#[cfg(feature = "uniffi")]
#[derive(uniffi_macros::Record)]
pub struct SessionRequestJsonRpcFfi {
    pub id: u64,
    pub method: String,
    pub params: SessionRequestFfi,
}

#[cfg(feature = "uniffi")]
#[derive(uniffi_macros::Record, Debug)]
pub struct SessionRequestResponseJsonRpcFfi {
    pub id: u64,
    pub jsonrpc: String,
    pub result: String, // JSON string instead of serde_json::Value
}
