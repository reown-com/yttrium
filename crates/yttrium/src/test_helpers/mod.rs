use {
    alloy::{
        network::{EthereumWallet, TransactionBuilder},
        primitives::{keccak256, Address, U256},
        rpc::types::TransactionRequest,
        signers::{k256::ecdsa::SigningKey, local::LocalSigner},
    },
    alloy_provider::{
        ext::AnvilApi, Provider, ProviderBuilder, ReqwestProvider,
    },
    std::time::{Duration, Instant},
};

pub fn private_faucet() -> LocalSigner<SigningKey> {
    use_account(None)
}

// Account index. Must have unique strings.
pub const BRIDGE_ACCOUNT_1: &str = "bridge_1";
pub const BRIDGE_ACCOUNT_2: &str = "bridge_2";
pub const BRIDGE_ACCOUNT_USDC_1557_1: &str = "bridge_3";
pub const BRIDGE_ACCOUNT_USDC_1557_2: &str = "bridge_4";

pub fn use_account(name: Option<&str>) -> LocalSigner<SigningKey> {
    use alloy::signers::local::{coins_bip39::English, MnemonicBuilder};
    let mut builder = MnemonicBuilder::<English>::default().phrase(
        std::env::var("FAUCET_MNEMONIC")
            .expect("You've not set the FAUCET_MNEMONIC"),
    );

    if let Some(name) = name {
        builder = builder
            .index(u32::from_be_bytes(
                keccak256(name).as_slice()[..4].try_into().unwrap(),
            ))
            .unwrap();
    }

    builder.build().unwrap()
}

pub async fn anvil_faucet(
    provider: &ReqwestProvider,
) -> LocalSigner<SigningKey> {
    let faucet = LocalSigner::random();
    provider.anvil_set_balance(faucet.address(), U256::MAX).await.unwrap();
    faucet
}

// Get tiny amounts of wei to test with
pub async fn use_faucet(
    provider: ReqwestProvider,
    faucet: LocalSigner<SigningKey>,
    amount: U256,
    to: Address,
) {
    // Basic check (which we can tune) to make sure we don't use excessive
    // amounts (e.g. 0.1) of test ETH. It is not infinite, so we should use
    // the minimum amount necessary.
    assert!(amount < U256::from(20), "You probably don't need that much");

    use_faucet_unlimited(provider, faucet, amount, to).await;
}

// Get resonable amounts of gwei for gas
// Provide the maximum amount you need for 1 execution of this
// This must be lower than `max_usd` to prevent abuse (find a cheaper L2)
// Set `multiplier` to top-off with additional gas for later executions
pub async fn use_faucet_gas(
    provider: ReqwestProvider,
    faucet: LocalSigner<SigningKey>,
    amount: U256,
    to: Address,
    multiplier: u64,
) {
    // Set the maximum
    let max_usd = 0.10;
    let eth_price = 3000.;
    let max = max_usd / eth_price * 10_i64.pow(18) as f64;
    assert!(amount < U256::from(max), "Crossed limit");
    let amount = amount * U256::from(multiplier);
    println!("Using faucet (multiplier:{multiplier}) to send {amount} to {to}");
    use_faucet_unlimited(provider, faucet, amount, to).await;
}

// Use the faucet without any limits. Function is intentionally private to
// prevent accidental abuse
async fn use_faucet_unlimited(
    provider: ReqwestProvider,
    faucet: LocalSigner<SigningKey>,
    amount: U256,
    to: Address,
) {
    let chain_id = format!("eip155:{}", provider.get_chain_id().await.unwrap());
    let faucet_address = faucet.address();
    let faucet_balance = provider.get_balance(faucet_address).await.unwrap();
    if amount > faucet_balance {
        panic!("not enough funds in faucet. Needed to send {amount} but only had {faucet_balance} available. Please add more funds to the faucet at {chain_id}:{faucet_address}");
    }
    let txn = TransactionRequest::default().with_to(to).with_value(amount);
    let start = Instant::now();
    loop {
        println!("sending txn: {:?}", txn);
        let txn_sent = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::new(faucet.clone()))
            .on_provider(provider.clone())
            .send_transaction(txn.clone())
            .await
            .unwrap()
            // .with_required_confirmations(3)
            .with_timeout(Some(Duration::from_secs(15)));
        println!(
            "txn hash: {} on chain {}",
            txn_sent.tx_hash(),
            provider.get_chain_id().await.unwrap()
        );
        // if provider
        //     .get_transaction_by_hash(*txn_sent.tx_hash())
        //     .await
        //     .unwrap()
        //     .is_none()
        // {
        //     println!("get_transaction_by_hash returned None, retrying...");
        //     continue;
        // }
        let receipt = txn_sent.get_receipt().await;
        if let Ok(receipt) = receipt {
            assert!(receipt.status());
            break;
        }

        println!("error getting receipt: {:?}", receipt);
        if start.elapsed() > Duration::from_secs(30) {
            panic!("timed out");
        }
    }
    let balance = provider.get_balance(to).await.unwrap();
    println!("Balance of {}: {}", to, balance);
    println!("amount: {}", amount);
    assert!(balance >= amount);
}
