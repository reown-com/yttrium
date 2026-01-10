use {
    alloy::{
        contract,
        rpc::types::TransactionReceipt,
        transports::{RpcError, TransportErrorKind},
    },
    thiserror::Error,
};

#[derive(Debug, Error)]
#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi::Error))]
pub enum PrepareError {
    #[error("Checking account code: {0}")]
    CheckingAccountCode(RpcError<TransportErrorKind>),

    #[error("Getting nonce: {0}")]
    GettingNonce(RpcError<TransportErrorKind>),

    #[error("Creating sponsored user operation: {0}")]
    CreatingSponsoredUserOp(CreateSponsoredUserOpError),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi::Error))]
pub enum PrepareDeployError {
    #[error("Sending delegation transaction: {0}")]
    SendingDelegationTransaction(contract::Error),

    #[error("Delegation transaction failed: {0:?}")]
    DelegationTransactionFailed(TransactionReceipt),

    #[error("Getting delegation transaction receipt: {0:?}")]
    GettingDelegationTransactionReceipt(
        alloy::providers::PendingTransactionError,
    ),

    #[error("Creating sponsored user operation: {0}")]
    CreatingSponsoredUserOp(CreateSponsoredUserOpError),
}

#[derive(Debug, Error)]
#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi::Error))]
pub enum CreateSponsoredUserOpError {
    #[error("Getting nonce: {0}")]
    GettingNonce(contract::Error),

    #[error("Getting user operation gas price: {0}")]
    GettingUserOperationGasPrice(eyre::Report),

    #[error("Sponsoring user operation: {0}")]
    SponsoringUserOperation(eyre::Report),
}

#[derive(Debug, Error)]
#[cfg_attr(any(feature = "uniffi", feature = "uniffi_derive"), derive(uniffi::Error))]
pub enum SendError {
    #[error("Checking account code: {0}")]
    SendingUserOperation(eyre::Report),

    #[error("Waiting for user operation receipt: {0}")]
    WaitingForUserOperationReceipt(eyre::Report),
}
