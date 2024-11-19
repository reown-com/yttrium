use alloy::{
    network::{Ethereum, EthereumWallet, TransactionBuilder},
    primitives::{keccak256, Address, U256},
    rpc::types::TransactionRequest,
    signers::{k256::ecdsa::SigningKey, local::LocalSigner},
};
use alloy_provider::{
    ext::AnvilApi, Provider, ProviderBuilder, ReqwestProvider,
};
use reqwest::IntoUrl;
use std::time::Duration;

pub fn private_faucet() -> LocalSigner<SigningKey> {
    use_account(None)
}

// Account index. Must have unique strings.
pub const BRIDGE_ACCOUNT_1: &str = "bridge_1";
pub const BRIDGE_ACCOUNT_2: &str = "bridge_2";

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

pub async fn anvil_faucet<T: IntoUrl>(url: T) -> LocalSigner<SigningKey> {
    let faucet = LocalSigner::random();
    let provider =
        ReqwestProvider::<Ethereum>::new_http(url.into_url().unwrap());
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
pub async fn use_faucet_gas(
    provider: ReqwestProvider,
    faucet: LocalSigner<SigningKey>,
    amount: U256,
    to: Address,
) {
    let max = 1000000000000_u128;
    assert!(amount < U256::from(max), "Crossed limit");
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
    let faucet_balance = provider.get_balance(faucet.address()).await.unwrap();
    if amount > faucet_balance {
        panic!("not enough funds in faucet. Needed to send {amount} but only had {faucet_balance} available");
    }
    ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::new(faucet))
        .on_provider(provider.clone())
        .send_transaction(
            TransactionRequest::default().with_to(to).with_value(amount),
        )
        .await
        .unwrap()
        .with_timeout(Some(Duration::from_secs(120)))
        .watch()
        .await
        .unwrap();
    let balance = provider.get_balance(to).await.unwrap();
    println!("Balance of {}: {}", to, balance);
    println!("amount: {}", amount);
    assert!(balance >= amount);
}
