pub const ETHERIEUM_MAINNET_CHAIN_ID: u64 = 1;
pub const MNEMONIC_PHRASE: &str =
    "test test test test test test test test test test test junk";
pub const CHAIN_ID: u64 = ETHERIEUM_MAINNET_CHAIN_ID;

#[tokio::test]
async fn test_send_transaction_on_sepolia() -> eyre::Result<()> {
    Ok(())
}
