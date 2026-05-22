pub use {
    bincode,
    solana_client::nonblocking::rpc_client::RpcClient as SolanaRpcClient,
    solana_commitment_config::CommitmentConfig as SolanaCommitmentConfig,
    solana_keypair::Keypair as SolanaKeypair,
    solana_sdk::pubkey::{
        ParsePubkeyError as SolanaParsePubkeyError, Pubkey as SolanaPubkey,
    },
    solana_signature::Signature as SolanaSignature,
    solana_transaction::versioned::VersionedTransaction as SolanaVersionedTransaction,
    spl_associated_token_account::get_associated_token_address,
};
use {const_format::formatcp, std::str::FromStr};

#[cfg(test)]
#[cfg(feature = "test_blockchain_api")]
mod tests;

pub const SOLANA_NAMESPACE: &str = "solana";
pub const SOLANA_MAINNET_CHAIN_ID: &str = "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp";
pub const SOLANA_MAINNET_CAIP2: &str =
    formatcp!("{SOLANA_NAMESPACE}:{SOLANA_MAINNET_CHAIN_ID}");

pub const SOLANA_USDC_ADDRESS: &str =
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

pub fn usdc_mint() -> SolanaPubkey {
    SolanaPubkey::from_str(SOLANA_USDC_ADDRESS).unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct SolanaSignedTransaction {
    pub signature: SolanaSignature,
    pub transaction: SolanaVersionedTransaction,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum SolanaSignTransactionError {
    #[error(
        "signer pubkey {pubkey} is not a required signer of this transaction"
    )]
    SignerNotRequired { pubkey: String },
}

pub fn sign_versioned_transaction(
    keypair: &SolanaKeypair,
    mut transaction: SolanaVersionedTransaction,
) -> Result<SolanaSignedTransaction, SolanaSignTransactionError> {
    use solana_signer::Signer;
    let pubkey = keypair.pubkey();
    let num_required =
        transaction.message.header().num_required_signatures as usize;
    // Only the static account keys can be signers; ALT-loaded keys cannot.
    let index = transaction
        .message
        .static_account_keys()
        .iter()
        .take(num_required)
        .position(|k| k == &pubkey)
        .ok_or_else(|| SolanaSignTransactionError::SignerNotRequired {
            pubkey: pubkey.to_string(),
        })?;

    let signature = keypair.sign_message(&transaction.message.serialize());

    if transaction.signatures.len() < num_required {
        transaction.signatures.resize(num_required, SolanaSignature::default());
    }
    transaction.signatures[index] = signature;

    Ok(SolanaSignedTransaction { signature, transaction })
}
