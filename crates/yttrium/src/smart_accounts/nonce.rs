use alloy::primitives::U256;

pub async fn get_nonce<P, T, N>(
    provider: &P,
    address: &crate::smart_accounts::simple_account::SimpleAccountAddress,
    entry_point_address: &crate::entry_point::EntryPointAddress,
) -> eyre::Result<u64>
where
    T: alloy::contract::private::Transport + ::core::clone::Clone,
    P: alloy::contract::private::Provider<T, N>,
    N: alloy::contract::private::Network,
{
    let entry_point_instance = crate::entry_point::EntryPoint::new(
        entry_point_address.to_address(),
        provider,
    );
    let key = U256::ZERO;

    let get_nonce_call =
        entry_point_instance.getNonce(address.to_address(), key).call().await?;

    let nonce_uint = get_nonce_call.nonce;

    let nonce: u64 = nonce_uint.to::<u64>();

    Ok(nonce)
}
