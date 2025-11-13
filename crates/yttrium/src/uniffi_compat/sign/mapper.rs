use {
    crate::{
        sign::{
            client_types::{
                ConnectParams, ConnectResult, Session, SessionProposal,
            },
            protocol_types::{
                SessionRequest, SessionRequestJsonRpc,
                SessionRequestJsonRpcResultResponse, SessionRequestRequest,
            },
        },
        uniffi_compat::sign::ffi_types::{
            ConnectParamsFfi, ConnectResultFfi, ErrorDataFfi, SessionFfi,
            SessionProposalFfi, SessionRequestFfi,
            SessionRequestJsonRpcErrorResponseFfi, SessionRequestJsonRpcFfi,
            SessionRequestJsonRpcResponseFfi,
            SessionRequestJsonRpcResultResponseFfi, SessionRequestRequestFfi,
        },
    },
    relay_rpc::rpc::ErrorData,
};

impl From<SessionProposalFfi> for SessionProposal {
    fn from(proposal: SessionProposalFfi) -> Self {
        Self {
            session_proposal_rpc_id: proposal.id,
            pairing_topic: proposal.topic.into(),
            relays: proposal.relays,
            pairing_sym_key: proposal.pairing_sym_key.try_into().unwrap(),
            proposer_public_key: proposal
                .proposer_public_key
                .try_into()
                .unwrap(),
            required_namespaces: proposal.required_namespaces,
            optional_namespaces: proposal.optional_namespaces,
            metadata: proposal.metadata,
            session_properties: proposal.session_properties,
            scoped_properties: proposal.scoped_properties,
            expiry_timestamp: proposal.expiry_timestamp,
        }
    }
}

impl From<SessionRequestJsonRpc> for SessionRequestJsonRpcFfi {
    fn from(request: SessionRequestJsonRpc) -> Self {
        Self {
            id: request.id,
            method: request.method,
            params: SessionRequestFfi {
                chain_id: request.params.chain_id,
                request: SessionRequestRequestFfi {
                    method: request.params.request.method,
                    params: serde_json::to_string(
                        &request.params.request.params,
                    )
                    .unwrap_or_default(),
                    expiry: request.params.request.expiry,
                },
            },
        }
    }
}

impl From<SessionProposal> for SessionProposalFfi {
    fn from(proposal: SessionProposal) -> Self {
        // Ensure both id and topic are properly converted to valid UTF-8 strings
        let id_string = proposal.session_proposal_rpc_id;

        // Be extremely defensive about topic string conversion
        let topic_string = {
            let raw_string = if let Ok(serialized) =
                serde_json::to_string(&proposal.pairing_topic)
            {
                // Remove quotes from JSON string
                serialized.trim_matches('"').to_string()
            } else {
                // Fallback to display format
                format!("{}", proposal.pairing_topic)
            };

            // Ensure the string is valid UTF-8 and only contains safe ASCII characters
            if raw_string.is_ascii()
                && raw_string.chars().all(|c| c.is_ascii_alphanumeric())
            {
                raw_string
            } else {
                // If anything looks suspicious, force it to be safe ASCII hex
                // This is a defensive fallback that should never be needed
                format!("fallback_{}", hex::encode(raw_string.as_bytes()))
            }
        };

        Self {
            id: id_string,
            topic: topic_string,
            pairing_sym_key: proposal.pairing_sym_key.to_vec(),
            proposer_public_key: proposal.proposer_public_key.to_vec(),
            relays: proposal.relays,
            required_namespaces: proposal.required_namespaces,
            optional_namespaces: proposal.optional_namespaces,
            metadata: proposal.metadata,
            session_properties: proposal.session_properties,
            scoped_properties: proposal.scoped_properties,
            expiry_timestamp: proposal.expiry_timestamp,
        }
    }
}

impl From<SessionRequestJsonRpcResultResponseFfi>
    for SessionRequestJsonRpcResultResponse
{
    fn from(response: SessionRequestJsonRpcResultResponseFfi) -> Self {
        Self {
            id: response.id,
            jsonrpc: response.jsonrpc,
            result: serde_json::Value::String(response.result),
        }
    }
}

impl From<SessionRequestJsonRpcResponseFfi>
    for crate::sign::protocol_types::SessionRequestJsonRpcResponse
{
    fn from(ffi: SessionRequestJsonRpcResponseFfi) -> Self {
        match ffi {
            SessionRequestJsonRpcResponseFfi::Result(result) => {
                crate::sign::protocol_types::SessionRequestJsonRpcResponse::Result(
                    crate::sign::protocol_types::SessionRequestJsonRpcResultResponse {
                        id: result.id,
                        jsonrpc: result.jsonrpc,
                        result: serde_json::Value::String(result.result),
                    }
                )
            }
            SessionRequestJsonRpcResponseFfi::Error(error) => {
                crate::sign::protocol_types::SessionRequestJsonRpcResponse::Error(
                    crate::sign::protocol_types::GenericJsonRpcResponseError {
                        id: error.id,
                        jsonrpc: error.jsonrpc,
                        error: serde_json::from_str(&error.error).unwrap(),
                    }
                )
            }
        }
    }
}

impl From<Session> for SessionFfi {
    fn from(session: Session) -> Self {
        Self {
            request_id: session.request_id,
            session_sym_key: session.session_sym_key.to_vec(),
            self_public_key: session.self_public_key.to_vec(),
            topic: session.topic,
            expiry: session.expiry,
            relay_protocol: session.relay_protocol,
            relay_data: session.relay_data,
            controller_key: session.controller_key.map(|k| k.to_vec()),
            self_meta_data: session.self_meta_data,
            peer_public_key: session.peer_public_key.map(|k| k.to_vec()),
            peer_meta_data: session.peer_meta_data,
            session_namespaces: session.session_namespaces,
            required_namespaces: session.required_namespaces,
            optional_namespaces: session.optional_namespaces,
            properties: session.session_properties,
            scoped_properties: session.scoped_properties,
            is_acknowledged: session.is_acknowledged,
            pairing_topic: session.pairing_topic.to_string(),
            transport_type: session.transport_type,
        }
    }
}

impl From<SessionFfi> for Session {
    fn from(session: SessionFfi) -> Self {
        Self {
            request_id: session.request_id,
            session_sym_key: session.session_sym_key.try_into().unwrap(),
            self_public_key: session.self_public_key.try_into().unwrap(),
            topic: session.topic,
            expiry: session.expiry,
            relay_protocol: session.relay_protocol,
            relay_data: session.relay_data,
            controller_key: session
                .controller_key
                .map(|k| k.try_into().unwrap()),
            self_meta_data: session.self_meta_data,
            peer_public_key: session
                .peer_public_key
                .map(|k| k.try_into().unwrap()),
            peer_meta_data: session.peer_meta_data,
            session_namespaces: session.session_namespaces,
            required_namespaces: session.required_namespaces,
            optional_namespaces: session.optional_namespaces,
            session_properties: session.properties,
            scoped_properties: session.scoped_properties,
            is_acknowledged: session.is_acknowledged,
            pairing_topic: session.pairing_topic.into(),
            transport_type: session.transport_type,
        }
    }
}

impl From<ErrorDataFfi> for ErrorData {
    fn from(error_data: ErrorDataFfi) -> Self {
        Self {
            code: error_data.code,
            message: error_data.message,
            data: error_data.data,
        }
    }
}

impl From<ConnectParamsFfi> for ConnectParams {
    fn from(params: ConnectParamsFfi) -> Self {
        Self {
            optional_namespaces: params.optional_namespaces,
            session_properties: params.session_properties,
            scoped_properties: params.scoped_properties,
        }
    }
}

impl From<ConnectResult> for ConnectResultFfi {
    fn from(result: ConnectResult) -> Self {
        Self { topic: result.topic, uri: result.uri }
    }
}

impl From<crate::sign::protocol_types::SessionRequestJsonRpcResponse>
    for SessionRequestJsonRpcResponseFfi
{
    fn from(
        response: crate::sign::protocol_types::SessionRequestJsonRpcResponse,
    ) -> Self {
        match response {
            crate::sign::protocol_types::SessionRequestJsonRpcResponse::Result(result) => {
                SessionRequestJsonRpcResponseFfi::Result(result.into())
            }
            crate::sign::protocol_types::SessionRequestJsonRpcResponse::Error(error) => {
                SessionRequestJsonRpcResponseFfi::Error(error.into())
            }
        }
    }
}

impl From<crate::sign::protocol_types::SessionRequestJsonRpcResultResponse>
    for SessionRequestJsonRpcResultResponseFfi
{
    fn from(
        result: crate::sign::protocol_types::SessionRequestJsonRpcResultResponse,
    ) -> Self {
        SessionRequestJsonRpcResultResponseFfi {
            id: result.id,
            jsonrpc: result.jsonrpc,
            result: serde_json::to_string(&result.result).unwrap_or_default(),
        }
    }
}

impl From<crate::sign::protocol_types::GenericJsonRpcResponseError>
    for SessionRequestJsonRpcErrorResponseFfi
{
    fn from(
        error: crate::sign::protocol_types::GenericJsonRpcResponseError,
    ) -> Self {
        SessionRequestJsonRpcErrorResponseFfi {
            id: error.id,
            jsonrpc: error.jsonrpc,
            error: serde_json::to_string(&error.error).unwrap_or_default(),
        }
    }
}

impl From<SessionRequestFfi> for SessionRequest {
    fn from(session_request: SessionRequestFfi) -> Self {
        SessionRequest {
            chain_id: session_request.chain_id,
            request: SessionRequestRequest {
                method: session_request.request.method,
                params: serde_json::from_str(&session_request.request.params)
                    .unwrap(),
                expiry: session_request.request.expiry,
            },
        }
    }
}

#[cfg(test)]
mod conversion_tests {
    use {
        super::*,
        crate::sign::protocol_types::{Metadata, ProtocolRpcId},
        relay_rpc::domain::Topic,
    };

    #[test]
    fn test_session_proposal_conversion() {
        // Create a test SessionProposal with known values
        let test_topic = Topic::from(
            "0c814f7d2d56c0e840f75612addaa170af479b1c8499632430b41c298bf49907"
                .to_string(),
        );
        let test_id = ProtocolRpcId::generate();

        let session_proposal = SessionProposal {
            session_proposal_rpc_id: test_id,
            pairing_topic: test_topic.clone(),
            pairing_sym_key: [1u8; 32],
            proposer_public_key: [2u8; 32],
            relays: vec![],
            required_namespaces: std::collections::HashMap::new(),
            optional_namespaces: std::collections::HashMap::new(),
            metadata: Metadata {
                name: "Test".to_string(),
                description: "Test".to_string(),
                url: "https://test.com".to_string(),
                icons: vec![],
                verify_url: None,
                redirect: None,
            },
            session_properties: std::collections::HashMap::new(),
            scoped_properties: std::collections::HashMap::new(),
            expiry_timestamp: None,
        };

        // Convert to FFI
        let ffi_proposal: SessionProposalFfi = session_proposal.into();

        // Print the actual values to see what we get
        println!("Original topic: {test_topic:?}");
        println!("Topic Display: {test_topic}");
        println!("Topic Debug: {test_topic:?}");
        println!("Topic JSON: {:?}", serde_json::to_string(&test_topic));

        println!("FFI id: {}", ffi_proposal.id);
        println!("FFI topic: {}", ffi_proposal.topic);
        println!("FFI topic bytes: {:?}", ffi_proposal.topic.as_bytes());
        println!("FFI topic len: {}", ffi_proposal.topic.len());

        // Check if the values are reasonable
        assert_eq!(ffi_proposal.id, test_id);
        assert!(!ffi_proposal.topic.is_empty(), "Topic should not be empty");
        assert!(ffi_proposal.topic.is_ascii(), "Topic should be ASCII");

        // The topic should be a hex string
        if ffi_proposal.topic.len() == 64 {
            assert!(
                ffi_proposal.topic.chars().all(|c| c.is_ascii_hexdigit()),
                "Topic should be a hex string, got: {}",
                ffi_proposal.topic
            );
        }
    }
}
