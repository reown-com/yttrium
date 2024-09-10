use crate::{
    entry_point::{EntryPoint, EntryPointAddress},
    smart_accounts::simple_account::SimpleAccountAddress,
};
use alloy::{
    contract::private::{Network, Provider, Transport},
    primitives::aliases::U192,
};
use core::clone::Clone;

pub async fn get_nonce<P, T, N>(
    provider: &P,
    address: &SimpleAccountAddress,
    entry_point_address: &EntryPointAddress,
) -> eyre::Result<u64>
where
    T: Transport + Clone,
    P: Provider<T, N>,
    N: Network,
{
    let entry_point_instance =
        EntryPoint::new(entry_point_address.to_address(), provider);
    let key = U192::ZERO;

    let get_nonce_call =
        entry_point_instance.getNonce(address.to_address(), key).call().await?;

    let nonce_uint = get_nonce_call.nonce;

    let nonce: u64 = nonce_uint.to::<u64>();

    Ok(nonce)
}
