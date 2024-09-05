use crate::chain::ChainId;
use crate::entry_point::EntryPointVersion;
use alloy::{
    contract::private::{Network, Provider, Transport},
    primitives::Address,
    sol,
};
use std::marker::PhantomData;

pub struct FactoryInstance<T, P, N = alloy::contract::private::Ethereum> {
    address: Address,
    provider: P,
    _network_transport: PhantomData<(N, T)>,
}

impl<T: Transport + Clone, P: Provider<T, N> + Clone, N: Network>
    FactoryInstance<T, P, N>
{
    pub fn new(address: Address, provider: P) -> Self {
        Self { address, provider, _network_transport: PhantomData }
    }

    pub fn local_v07(provider: P) -> Self {
        let address = FactoryAddress::local_v07().to_address();
        Self::new(address, provider)
    }

    pub fn instance(
        self,
    ) -> SimpleAccountFactory::SimpleAccountFactoryInstance<T, P, N> {
        SimpleAccountFactory::new(self.address, self.provider)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FactoryAddress(alloy::primitives::Address);

impl FactoryAddress {
    pub const V06: &'static str = ""; // TODO

    pub const V07: &'static str = "0x91E60e0613810449d098b0b5Ec8b51A0FE8c8985";

    pub const SEPOLIA_V06: &'static str =
        "0x9406Cc6185a346906296840746125a0E44976454";

    pub const SEPOLIA_V07: &'static str =
        "0x91E60e0613810449d098b0b5Ec8b51A0FE8c8985";

    pub const LOCAL_V06: &'static str =
        "0x9406Cc6185a346906296840746125a0E44976454";

    pub const LOCAL_V07: &'static str =
        "0x91E60e0613810449d098b0b5Ec8b51A0FE8c8985";

    pub fn new(address: alloy::primitives::Address) -> Self {
        Self(address)
    }

    pub fn v06() -> Self {
        Self(Self::V06.parse().unwrap())
    }

    pub fn v07() -> Self {
        Self(Self::V07.parse().unwrap())
    }

    pub fn local_v06() -> Self {
        Self(Self::LOCAL_V06.parse().unwrap())
    }

    pub fn local_v07() -> Self {
        Self(Self::LOCAL_V07.parse().unwrap())
    }

    pub fn to_address(self) -> alloy::primitives::Address {
        self.into()
    }
}

impl From<FactoryAddress> for Address {
    fn from(factory_address: FactoryAddress) -> Self {
        factory_address.0
    }
}

pub fn factory_address_from_chain_id(chain_id: ChainId) -> FactoryAddress {
    factory_address_from_chain_id_and_version(chain_id, EntryPointVersion::V07)
}

pub fn factory_address_from_chain_id_and_version(
    chain_id: ChainId,
    entry_point_version: EntryPointVersion,
) -> FactoryAddress {
    match chain_id {
        ChainId::ETHEREUM_MAINNET => match entry_point_version {
            EntryPointVersion::V06 => FactoryAddress::v06(),
            EntryPointVersion::V07 => FactoryAddress::v07(),
        },
        ChainId::ETHEREUM_SEPOLIA => match entry_point_version {
            EntryPointVersion::V06 => FactoryAddress::v06(),
            EntryPointVersion::V07 => FactoryAddress::v07(),
        },
        ChainId::LOCAL_FOUNDRY_ETHEREUM_SEPOLIA => match entry_point_version {
            EntryPointVersion::V06 => FactoryAddress::local_v06(),
            EntryPointVersion::V07 => FactoryAddress::local_v07(),
        },
        _ => panic!("Unsupported chain ID"),
    }
}

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    SimpleAccountFactory,
    "src/contracts/artifacts/contracts/samples/SimpleAccountFactory.sol/SimpleAccountFactory.json"
);
