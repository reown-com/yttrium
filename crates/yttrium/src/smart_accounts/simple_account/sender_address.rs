use crate::{
    bundler::{
        client::BundlerClient, config::BundlerConfig,
        pimlico::client::BundlerClient as PimlicoBundlerClient,
    },
    chain::ChainId,
    config::Config,
    entry_point::{
        get_sender_address::get_sender_address_v07, EntryPointVersion,
    },
    smart_accounts::simple_account::{
        create_account::SimpleAccountCreate, factory::FactoryAddress,
    },
};
use alloy::{
    network::EthereumWallet, primitives::Address, providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};

pub async fn get_sender_address_with_signer(
    config: Config,
    chain_id: u64,
    signer: PrivateKeySigner,
) -> eyre::Result<Address> {
    let rpc_base_url = config.clone().endpoints.rpc.base_url;

    let chain_id = ChainId::new_eip155(chain_id.clone());
    let chain =
        crate::chain::Chain::new(chain_id.clone(), EntryPointVersion::V07, "");

    let entry_point_config = chain.entry_point_config();

    let entry_point_address = entry_point_config.address();

    // Create a provider

    let alloy_signer = signer;
    let ethereum_wallet = EthereumWallet::new(alloy_signer.clone());

    let owner = ethereum_wallet.clone().default_signer();
    let owner_address = owner.address();

    let rpc_url = rpc_base_url.parse()?;
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(ethereum_wallet.clone())
        .on_http(rpc_url);

    let simple_account_factory_address_primitives: Address =
        "0x91E60e0613810449d098b0b5Ec8b51A0FE8c8985".parse()?;
    let simple_account_factory_address =
        FactoryAddress::new(simple_account_factory_address_primitives);

    let factory_data_call = SimpleAccountCreate::new_u64(owner_address, 2);

    let factory_data_value = factory_data_call.encode();

    let factory_data_value_hex = hex::encode(factory_data_value.clone());

    let factory_data_value_hex_prefixed =
        format!("0x{}", factory_data_value_hex);

    println!(
        "Generated factory_data: {:?}",
        factory_data_value_hex_prefixed.clone()
    );

    // 5. Calculate the sender address

    let sender_address = get_sender_address_v07(
        &provider,
        simple_account_factory_address.into(),
        factory_data_value.clone().into(),
        entry_point_address,
    )
    .await?;

    println!("Calculated sender address: {:?}", sender_address);

    Ok(sender_address)
}
