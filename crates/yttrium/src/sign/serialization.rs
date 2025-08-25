// Ffi types
#[uniffi::export]
pub fn session_proposal_ffi_to_json(
    object: &super::ffi_types::SessionProposalFfi,
) -> String {
    serde_json::to_string(object).expect("Failed to serialize session proposal")
}

#[uniffi::export]
pub fn session_proposal_ffi_from_json(
    json: &str,
) -> super::ffi_types::SessionProposalFfi {
    serde_json::from_str(json).expect("Failed to deserialize session proposal")
}

#[uniffi::export]
pub fn session_ffi_to_json(object: &super::ffi_types::SessionFfi) -> String {
    serde_json::to_string(object).expect("Failed to serialize session")
}

#[uniffi::export]
pub fn session_ffi_from_json(json: &str) -> super::ffi_types::SessionFfi {
    serde_json::from_str(json).expect("Failed to deserialize session")
}

#[uniffi::export]
pub fn session_request_request_ffi_to_json(
    object: &super::ffi_types::SessionRequestRequestFfi,
) -> String {
    serde_json::to_string(object)
        .expect("Failed to serialize session request request")
}

#[uniffi::export]
pub fn session_request_request_from_json(
    json: &str,
) -> super::ffi_types::SessionRequestRequestFfi {
    serde_json::from_str(json)
        .expect("Failed to deserialize session request request")
}

#[uniffi::export]
pub fn session_request_ffi_to_json(
    object: &super::ffi_types::SessionRequestFfi,
) -> String {
    serde_json::to_string(object).expect("Failed to serialize session request")
}

#[uniffi::export]
pub fn session_request_ffi_from_json(
    json: &str,
) -> super::ffi_types::SessionRequestFfi {
    serde_json::from_str(json).expect("Failed to deserialize session request")
}

#[uniffi::export]
pub fn session_request_json_rpc_ffi_to_json(
    object: &super::ffi_types::SessionRequestJsonRpcFfi,
) -> String {
    serde_json::to_string(object)
        .expect("Failed to serialize session request json")
}

#[uniffi::export]
pub fn session_request_json_rpc_ffi_from_json(
    json: &str,
) -> super::ffi_types::SessionRequestJsonRpcFfi {
    serde_json::from_str(json)
        .expect("Failed to deserialize session request json")
}

#[uniffi::export]
pub fn session_request_json_rpc_result_response_ffi_to_json(object: &super::ffi_types::SessionRequestJsonRpcResultResponseFfi) -> String {
    serde_json::to_string(object).expect("Failed to serialize session request response json")
}

#[uniffi::export]
pub fn session_request_json_rpc_result_response_ffi_from_json(json: &str) -> super::ffi_types::SessionRequestJsonRpcResultResponseFfi {
    serde_json::from_str(json).expect("Failed to deserialize session request response json")
}

#[uniffi::export]
pub fn session_request_json_rpc_error_response_ffi_to_json(object: &super::ffi_types::SessionRequestJsonRpcErrorResponseFfi) -> String {
    serde_json::to_string(object).expect("Failed to serialize session request response json")
}

#[uniffi::export]
pub fn session_request_json_rpc_error_response_ffi_from_json(json: &str) -> super::ffi_types::SessionRequestJsonRpcErrorResponseFfi {
    serde_json::from_str(json).expect("Failed to deserialize session request response json")
}

#[uniffi::export]
pub fn error_data_ffi_to_json(object: &super::ffi_types::ErrorDataFfi) -> String {
    serde_json::to_string(object).expect("Failed to serialize error data")
}

#[uniffi::export]
pub fn error_data_ffi_from_json(json: &str) -> super::ffi_types::ErrorDataFfi {
    serde_json::from_str(json).expect("Failed to deserialize error data")
}

// protocol types

#[uniffi::export]
pub fn proposal_namespace_to_json(
    object: &crate::sign::protocol_types::ProposalNamespace,
) -> String {
    serde_json::to_string(object)
        .expect("Failed to serialize proposal namespace")
}

#[uniffi::export]
pub fn proposal_namespace_from_json(
    json: &str,
) -> crate::sign::protocol_types::ProposalNamespace {
    serde_json::from_str(json)
        .expect("Failed to deserialize proposal namespace")
}

#[uniffi::export]
pub fn relay_to_json(object: &crate::sign::protocol_types::Relay) -> String {
    serde_json::to_string(object).expect("Failed to serialize relay")
}

#[uniffi::export]
pub fn relay_from_json(json: &str) -> crate::sign::protocol_types::Relay {
    serde_json::from_str(json).expect("Failed to deserialize relay")
}

#[uniffi::export]
pub fn settle_namespace_to_json(
    object: &crate::sign::protocol_types::SettleNamespace,
) -> String {
    serde_json::to_string(object).expect("Failed to serialize settle namespace")
}

#[uniffi::export]
pub fn settle_namespace_from_json(
    json: &str,
) -> crate::sign::protocol_types::SettleNamespace {
    serde_json::from_str(json).expect("Failed to deserialize settle namespace")
}

#[uniffi::export]
pub fn metadata_to_json(
    object: &crate::sign::protocol_types::Metadata,
) -> String {
    serde_json::to_string(object).expect("Failed to serialize metadata")
}

#[uniffi::export]
pub fn metadata_from_json(json: &str) -> crate::sign::protocol_types::Metadata {
    serde_json::from_str(json).expect("Failed to deserialize metadata")
}

#[uniffi::export]
pub fn redirect_to_json(
    object: &crate::sign::protocol_types::Redirect,
) -> String {
    serde_json::to_string(object).expect("Failed to serialize redirect")
}

#[uniffi::export]
pub fn redirect_from_json(json: &str) -> crate::sign::protocol_types::Redirect {
    serde_json::from_str(json).expect("Failed to deserialize redirect")
}
