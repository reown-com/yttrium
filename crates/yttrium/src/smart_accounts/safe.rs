use alloy::{
    dyn_abi::DynSolValue,
    primitives::{address, keccak256, Address, Bytes, Uint, U256},
    sol,
    sol_types::SolCall,
};

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    SafeProxyFactory,
    "safe-smart-account/build/artifacts/contracts/proxies/SafeProxyFactory.sol/SafeProxyFactory.json"
    // "../../target/.foundry/forge/out/SafeProxyFactory.sol/SafeProxyFactory.json"
    // concat!(env!("OUT_DIR"), "/../../../../.foundry/forge/out/SafeProxyFactory.sol/SafeProxyFactory.json")
);

sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc, abi)]
    Safe,
    "safe-smart-account/build/artifacts/contracts/Safe.sol/Safe.json"
);

sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    Safe7579Launchpad,
    "safe7579/artifacts/Safe7579Launchpad.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    Safe7579,
    "safe7579/artifacts/Safe7579.json"
);

// Had to copy from safe7579/artifacts/interfaces/IERC7579Account.json
// This struct doesn't seem to be in generated ABIs
sol!(
    #[allow(missing_docs)]
    #[sol(rpc, abi)]
    struct Execution {
        address target;
        uint256 value;
        bytes callData;
    }
);

// https://github.com/WalletConnect/secure-web3modal/blob/f1d16f973a313e598d124a0e4751aee12d5de628/src/core/SmartAccountSdk/utils.ts#L180
pub const SAFE_ERC_7579_LAUNCHPAD_ADDRESS: Address =
    address!("EBe001b3D534B9B6E2500FB78E67a1A137f561CE");
// https://github.com/WalletConnect/secure-web3modal/blob/f1d16f973a313e598d124a0e4751aee12d5de628/src/core/SmartAccountSdk/utils.ts#L181
// https://docs.pimlico.io/permissionless/how-to/accounts/use-erc7579-account
// https://docs.safe.global/advanced/erc-7579/tutorials/7579-tutorial
pub const SAFE_4337_MODULE_ADDRESS: Address =
    address!("3Fdb5BC686e861480ef99A6E3FaAe03c0b9F32e2");

// // https://github.com/pimlicolabs/permissionless.js/blob/b8798c121eecba6a71f96f8ddf8e0ad2e98a3236/packages/permissionless/accounts/safe/toSafeSmartAccount.ts#L436
pub const SEPOLIA_SAFE_ERC_7579_SINGLETON_ADDRESS: Address =
    address!("41675C099F32341bf84BFc5382aF534df5C7461a");

// https://github.com/safe-global/safe-modules-deployments/blob/d6642d90659de19e54bb4a20d646b30bd0a51885/src/assets/safe-4337-module/v0.3.0/safe-4337-module.json#L7
// https://github.com/pimlicolabs/permissionless.js/blob/b8798c121eecba6a71f96f8ddf8e0ad2e98a3236/packages/permissionless/accounts/safe/toSafeSmartAccount.ts#L432
// const SEPOLIA_SAFE_4337_MODULE_ADDRESS: Address =
//     address!("75cf11467937ce3F2f357CE24ffc3DBF8fD5c226");

// https://github.com/pimlicolabs/permissionless.js/blob/b8798c121eecba6a71f96f8ddf8e0ad2e98a3236/packages/permissionless/accounts/safe/toSafeSmartAccount.ts#L438C36-L438C76
// Only used for non-ERC-7579 accounts
// const SAFE_MULTI_SEND_ADDRESS: Address =
//     address!("38869bf66a61cF6bDB996A6aE40D5853Fd43B526");

// https://github.com/safe-global/safe-modules-deployments/blob/d6642d90659de19e54bb4a20d646b30bd0a51885/src/assets/safe-4337-module/v0.3.0/safe-module-setup.json#L7
// https://github.com/pimlicolabs/permissionless.js/blob/b8798c121eecba6a71f96f8ddf8e0ad2e98a3236/packages/permissionless/accounts/safe/toSafeSmartAccount.ts#L431
const _SAFE_MODULE_SETUP_ADDRESS: Address =
    address!("2dd68b007B46fBe91B9A7c3EDa5A7a1063cB5b47");

pub const SAFE_PROXY_FACTORY_ADDRESS: Address =
    address!("4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67");

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    SafeModuleSetup,
    "safe-modules/modules/4337/build/artifacts/contracts/SafeModuleSetup.sol/SafeModuleSetup.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    MultiSend,
    "safe-smart-account/build/artifacts/contracts/libraries/MultiSend.sol/MultiSend.json"
);

pub const DUMMY_SIGNATURE_HEX: &str = "0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

// https://github.com/WalletConnect/secure-web3modal/blob/c19a1e7b21c6188261728f4d521a17f94da4f055/src/core/SmartAccountSdk/constants.ts#L10
// const APPKIT_SALT: U256 = U256::from_str("zg3ijy0p46");

pub fn init_data() -> Safe7579Launchpad::initSafe7579Call {
    Safe7579Launchpad::initSafe7579Call {
        safe7579: SAFE_4337_MODULE_ADDRESS,
        executors: vec![],
        fallbacks: vec![],
        hooks: vec![],
        attesters: vec![],
        threshold: 0,
    }
}

#[derive(Debug, Clone)]
pub struct Owners {
    pub owners: Vec<Address>,
    pub threshold: u8,
}

// permissionless -> getInitializerCode
fn get_initializer_code(owners: Owners) -> Bytes {
    let init_hash = keccak256(
        DynSolValue::Tuple(vec![
            DynSolValue::Address(SEPOLIA_SAFE_ERC_7579_SINGLETON_ADDRESS),
            DynSolValue::Array(
                owners
                    .owners
                    .into_iter()
                    .map(DynSolValue::Address)
                    .collect::<Vec<_>>(),
            ),
            DynSolValue::Uint(Uint::from(owners.threshold), 256),
            DynSolValue::Address(SAFE_ERC_7579_LAUNCHPAD_ADDRESS),
            DynSolValue::Bytes(init_data().abi_encode()),
            DynSolValue::Address(SAFE_4337_MODULE_ADDRESS),
            DynSolValue::Array(vec![]),
        ])
        .abi_encode_params(),
    );

    Safe7579Launchpad::preValidationSetupCall {
        initHash: init_hash,
        to: Address::ZERO,
        preInit: Bytes::new(),
    }
    .abi_encode()
    .into()
}

pub fn factory_data(
    owners: Owners,
) -> SafeProxyFactory::createProxyWithNonceCall {
    let initializer = get_initializer_code(owners);

    // https://github.com/pimlicolabs/permissionless.js/blob/b8798c121eecba6a71f96f8ddf8e0ad2e98a3236/packages/permissionless/accounts/safe/toSafeSmartAccount.ts#L840
    SafeProxyFactory::createProxyWithNonceCall {
        _singleton: SAFE_ERC_7579_LAUNCHPAD_ADDRESS,
        initializer,
        saltNonce: U256::ZERO,
    }
}
