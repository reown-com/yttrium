use {
    crate::{
        entry_point::{EntryPoint, EntryPointAddress},
        smart_accounts::account_address::AccountAddress,
    },
    alloy::{
        contract::{Error, private::Network},
        primitives::{U256, aliases::U192},
        providers::Provider,
    },
};

pub async fn get_nonce<P, N>(
    provider: &P,
    address: AccountAddress,
    entry_point_address: &EntryPointAddress,
) -> Result<U256, Error>
where
    P: Provider<N>,
    N: Network,
{
    get_nonce_with_key(provider, address, entry_point_address, U192::ZERO).await
}

pub async fn get_nonce_with_key<P, N>(
    provider: &P,
    address: AccountAddress,
    entry_point_address: &EntryPointAddress,
    key: U192,
) -> Result<U256, Error>
where
    P: Provider<N>,
    N: Network,
{
    let entry_point_instance =
        EntryPoint::new(entry_point_address.to_address(), provider);

    let nonce =
        entry_point_instance.getNonce(address.to_address(), key).call().await?;

    Ok(nonce)
}
