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
