use alloy::{
    network::{Ethereum, EthereumWallet, TransactionBuilder},
    primitives::{Address, U256},
    rpc::types::TransactionRequest,
    signers::{k256::ecdsa::SigningKey, local::LocalSigner},
};
use alloy_provider::{
    ext::AnvilApi, Provider, ProviderBuilder, ReqwestProvider,
};
use reqwest::IntoUrl;

pub async fn anvil_faucet<T: IntoUrl>(url: T) -> LocalSigner<SigningKey> {
    let faucet = LocalSigner::random();
    let provider =
        ReqwestProvider::<Ethereum>::new_http(url.into_url().unwrap());
    provider.anvil_set_balance(faucet.address(), U256::MAX).await.unwrap();
    faucet
}

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

    ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(EthereumWallet::new(faucet))
        .on_provider(provider.clone())
        .send_transaction(
            TransactionRequest::default().with_to(to).with_value(amount),
        )
        .await
        .unwrap()
        .watch()
        .await
        .unwrap();
    let balance = provider.get_balance(to).await.unwrap();
    assert_eq!(balance, amount);
}
