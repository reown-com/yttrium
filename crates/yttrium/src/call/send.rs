use {
    crate::{
        call::Call, config::Config,
        smart_accounts::account_address::AccountAddress,
        user_operation::UserOperationV07,
    },
    alloy::primitives::Bytes,
    core::fmt,
    safe_test::{
        DoSendTransactionParams, OwnerSignature, PreparedSendTransaction,
    },
};

pub mod safe_test;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserOperationEstimated(UserOperationV07);

impl From<UserOperationEstimated> for UserOperationV07 {
    fn from(val: UserOperationEstimated) -> Self {
        val.0
    }
}

#[derive(Debug, Clone)]
pub struct SignedUserOperation(UserOperationV07);

impl From<SignedUserOperation> for UserOperationV07 {
    fn from(val: SignedUserOperation) -> Self {
        val.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SentUserOperationHash(String);

impl From<SentUserOperationHash> for String {
    fn from(user_operation_hash: SentUserOperationHash) -> Self {
        user_operation_hash.0
    }
}

impl fmt::Display for SentUserOperationHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub async fn prepare_send_transaction(
    calls: Vec<Call>,
    owner: AccountAddress,
    _chain_id: u64,
    config: Config,
) -> eyre::Result<PreparedSendTransaction> {
    let user_operation_hash = safe_test::prepare_send_transactions(
        calls,
        owner.into(),
        None,
        None,
        config,
    )
    .await?;

    Ok(user_operation_hash)
}

pub async fn do_send_transactions(
    signatures: Vec<OwnerSignature>,
    do_send_transaction_params: DoSendTransactionParams,
    _chain_id: u64,
    config: Config,
) -> eyre::Result<Bytes> {
    let user_operation_hash = safe_test::do_send_transactions(
        signatures,
        do_send_transaction_params,
        config,
    )
    .await?;

    Ok(user_operation_hash)
}
