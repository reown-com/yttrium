use relay_rpc::rpc::ErrorData;

#[cfg(feature = "uniffi")]
use crate::sign::ffi_types::SessionProposal;
use crate::sign::{Session};
use crate::sign::ffi_types::{ErrorDataFfi, SessionFfi, SessionProposalFfi, SessionRequestFfi, SessionRequestJsonRpcFfi, SessionRequestRequestFfi, SessionRequestResponseJsonRpcFfi};
use crate::sign::protocol_types::{SessionRequestJsonRpc, SessionRequestResponseJsonRpc};

#[cfg(feature = "uniffi")]
impl From<SessionProposalFfi> for SessionProposal {
    fn from(proposal: SessionProposalFfi) -> Self {
        Self {
            session_proposal_rpc_id: proposal.id.parse::<u64>().unwrap(),
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

#[cfg(feature = "uniffi")]
impl From<SessionRequestJsonRpc> for SessionRequestJsonRpcFfi {
    fn from(request: SessionRequestJsonRpc) -> Self {
        Self {
            id: request.id,
            method: request.method,
            params: SessionRequestFfi {
                chain_id: request.params.chain_id,
                request: SessionRequestRequestFfi {
                    method: request.params.request.method,
                    params: serde_json::to_string(&request.params.request.params).unwrap_or_default(),
                    expiry: request.params.request.expiry,
                },
            },
        }
    }
}

#[cfg(feature = "uniffi")]
impl From<SessionProposal> for SessionProposalFfi {
    fn from(proposal: SessionProposal) -> Self {
        // Ensure both id and topic are properly converted to valid UTF-8 strings
        let id_string = proposal.session_proposal_rpc_id.to_string();

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

#[cfg(feature = "uniffi")]
impl From<SessionRequestResponseJsonRpcFfi> for SessionRequestResponseJsonRpc {
    fn from(response: SessionRequestResponseJsonRpcFfi) -> Self {
        Self {
            id: response.id,
            jsonrpc: response.jsonrpc,
            result: serde_json::from_str(&response.result).unwrap_or_default(),
        }
    }
}

#[cfg(feature = "uniffi")]
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

#[cfg(feature = "uniffi")]
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
            controller_key: session.controller_key.map(|k| k.try_into().unwrap()),
            self_meta_data: session.self_meta_data,
            peer_public_key: session.peer_public_key.map(|k| k.try_into().unwrap()),
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

#[cfg(feature = "uniffi")]
impl From<ErrorDataFfi> for ErrorData {
    fn from(error_data: ErrorDataFfi) -> Self {
        Self {
            code: error_data.code,
            message: error_data.message,
            data: error_data.data,
        }
    }
}