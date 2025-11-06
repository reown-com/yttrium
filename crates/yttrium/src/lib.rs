#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
#[cfg(feature = "uniffi")]
pub mod uniffi_compat;

#[cfg(feature = "wasm")]
pub mod wasm_compat;

#[cfg(feature = "account_client")]
pub mod account_client;
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
pub mod blockchain_api;
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
pub mod bundler;
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
pub mod call;
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
pub mod chain;
#[cfg(feature = "chain_abstraction_client")]
pub mod chain_abstraction;
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
pub mod config;
#[cfg(feature = "account_client")]
pub mod eip7702;
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
pub mod entry_point;
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
pub mod erc20;
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
pub mod erc4337;
#[cfg(feature = "erc6492_client")]
pub mod erc6492_client;
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
pub mod erc7579;
pub mod error;
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
pub mod jsonrpc;
#[cfg(any(
    feature = "chain_abstraction_client",
    feature = "transaction_sponsorship_client"
))]
pub mod provider_pool;
#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "sign_client"
))]
pub mod pulse;
pub mod serde;
#[cfg(feature = "sign_client")]
pub mod sign;
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
pub mod smart_accounts;
pub mod spawn;
#[cfg(feature = "stacks")]
pub mod stacks_provider;
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
pub mod test_helpers;
pub mod time;
#[cfg(feature = "ton")]
pub mod ton_provider;
#[cfg(feature = "transaction_sponsorship_client")]
pub mod transaction_sponsorship;
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
pub mod user_operation;
pub mod utils;
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
pub mod wallet_provider;
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
pub mod wallet_service_api;

#[cfg(test)]
pub mod examples;

// Android JNI initialization for rustls-platform-verifier
// TODO try to move this to uniffi_compat or kotlin-ffi
#[cfg(all(target_os = "android", feature = "android"))]
#[no_mangle]
pub extern "C" fn Java_com_yttrium_YttriumKt_initializeTls(
    mut env: jni::JNIEnv,
    _class: jni::objects::JClass,
    context: jni::objects::JObject,
) {
    rustls_platform_verifier::android::init_hosted(&mut env, context).unwrap();
}

// Android JNI initialization for rustls-platform-verifier (utils variant)
#[cfg(all(target_os = "android", feature = "android"))]
#[no_mangle]
pub extern "C" fn Java_com_yttrium_utils_YttriumUtilsKt_initializeTls(
    mut env: jni::JNIEnv,
    _class: jni::objects::JClass,
    context: jni::objects::JObject,
) {
    rustls_platform_verifier::android::init_hosted(&mut env, context).unwrap();
}
