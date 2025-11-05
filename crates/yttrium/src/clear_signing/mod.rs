mod eip712;
mod engine;
mod resolver;
mod token_registry;

pub use eip712::{format_typed_data, Eip712Error, TypeMember, TypedData};
pub use engine::{
    format_with_resolved, DisplayItem, DisplayModel, EngineError, RawPreview,
};
pub use resolver::ResolvedDescriptor;
pub use token_registry::{lookup_erc20_token, lookup_native_token, TokenMeta};

use resolver::ResolverError;

/// Formats a clear signing preview including an optional native value.
pub fn format_with_value(
    chain_id: u64,
    to: &str,
    value: Option<&[u8]>,
    calldata: &[u8],
) -> Result<DisplayModel, EngineError> {
    let resolved =
        resolver::resolve(chain_id, to).map_err(map_resolver_error)?;

    format_with_resolved(resolved, chain_id, to, value, calldata)
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
    EngineError::Resolver(err.to_string())
}
