// WASM is single-threaded and the browser host types are not `Send`; the same `Arc`-based code
// serves native, where everything is `Send + Sync`.
#![cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]

//! WalletConnect Sign Protocol v3 client. Implementation guide: `V3_RUST_SCOPE.md` in the
//! monorepo; normative spec: `sign-v3-spec`.

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

pub mod client;
pub mod ffi;
pub mod host;
pub mod protocol;
pub mod rt;
pub mod transport;

pub use {
    client::{
        SignClient,
        types::{
            ConnectResult, Namespace, Namespaces, ProposalParams, RpcError,
            RpcRequest, RpcResponse, Session, SessionEventData,
            SessionProposal, SignConfig, SignError, SignEvent,
        },
    },
    host::{Listener, MemoryStorage, Storage, StorageError},
};
