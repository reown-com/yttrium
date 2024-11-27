use super::module::{Module, ModuleType};
use crate::smart_accounts::safe::Owners;
use alloy::{
    primitives::{address, Address, Bytes, U256},
    sol_types::SolValue,
};

pub const OWNABLE_VALIDATOR_ADDRESS: Address =
    address!("2483DA3A338895199E5e538530213157e931Bf06");

// encodeValidationData: https://github.com/rhinestonewtf/module-sdk/blob/1f2f2c5380614ad07b6e1ccbb5a9ed55374c673c/src/module/ownable-validator/usage.ts#L171
pub fn encode_owners(owners: &Owners) -> Bytes {
    let mut owner_addresses = owners.owners.clone();
    owner_addresses.sort();
    (U256::from(owners.threshold), owner_addresses).abi_encode_params().into()
}

pub fn get_ownable_validator(owners: &Owners, hook: Option<Address>) -> Module {
    Module {
        address: OWNABLE_VALIDATOR_ADDRESS,
        module: OWNABLE_VALIDATOR_ADDRESS,
        init_data: encode_owners(owners),
        de_init_data: Bytes::default(),
        additional_context: Bytes::default(),
        hook,
        r#type: ModuleType::Validator,
    }
}
