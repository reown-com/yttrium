mod descriptor;
mod eip712;
mod engine;
mod resolver;
mod token_registry;

pub use eip712::{format_typed_data, Eip712Error, TypeMember, TypedData};
pub use engine::{
    format_with_resolved_call, DisplayItem, DisplayModel, EngineError,
    RawPreview,
};
pub use resolver::{ResolvedCall, ResolvedDescriptor};
pub use token_registry::{
    lookup_erc20_token, lookup_native_token, lookup_token_by_caip19, TokenMeta,
};

use resolver::ResolverError;

/// Formats a clear signing preview including an optional native value.
pub fn format_with_value(
    chain_id: u64,
    to: &str,
    value: Option<&[u8]>,
    calldata: &[u8],
) -> Result<DisplayModel, EngineError> {
    let resolved = resolver::resolve_call(chain_id, to, calldata, value)
        .map_err(map_resolver_error)?;

    format_with_resolved_call(resolved, chain_id, to, value, calldata)
}

/// Formats a clear signing preview without an explicit call value.
pub fn format(
    chain_id: u64,
    to: &str,
    calldata: &[u8],
) -> Result<DisplayModel, EngineError> {
    format_with_value(chain_id, to, None, calldata)
}

fn map_resolver_error(err: ResolverError) -> EngineError {
    let message = err.to_string();
    if message.contains("token registry missing entry") {
        EngineError::TokenRegistry(message)
    } else {
        EngineError::Resolver(message)
    }
}
