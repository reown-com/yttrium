use {
    super::module::{Module, ModuleType},
    crate::smart_accounts::safe::Owners,
    alloy::{
        primitives::{address, bytes, Address, Bytes, U256},
        sol_types::SolValue,
    },
};

// https://github.com/rhinestonewtf/module-sdk/blob/main/src/module/ownable-validator/constants.ts
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

pub fn get_ownable_validator_signature(signatures: Vec<Bytes>) -> Bytes {
    signatures.into_iter().flat_map(Bytes::into_iter).collect()
}

pub fn get_ownable_validator_mock_signature(owners: &Owners) -> Bytes {
    // https://github.com/rhinestonewtf/module-sdk/blob/4b4174fad195a16977a3a989e63f85b46c71bbfe/src/module/ownable-validator/usage.ts#L208
    const MOCK_SIGNATURE: Bytes =
      bytes!("e8b94748580ca0b4993c9a1b86b5be851bfc076ff5ce3a1ff65bf16392acfcb800f9b4f1aef1555c7fce5599fffb17e7c635502154a0333ba21f3ae491839af51c");
    get_ownable_validator_signature(
        (0..owners.threshold).map(|_| MOCK_SIGNATURE).collect(),
    )
}
