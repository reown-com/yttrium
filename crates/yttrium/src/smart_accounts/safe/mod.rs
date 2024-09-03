use alloy::{
    dyn_abi::DynSolValue,
    primitives::{address, Address, Bytes, U256},
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
    #[sol(rpc)]
    Safe,
    "safe-smart-account/build/artifacts/contracts/Safe.sol/Safe.json"
);

// https://github.com/WalletConnect/secure-web3modal/blob/c19a1e7b21c6188261728f4d521a17f94da4f055/src/core/SmartAccountSdk/utils.ts#L178
// https://github.com/WalletConnect/secure-web3modal/blob/c19a1e7b21c6188261728f4d521a17f94da4f055/src/core/SmartAccountSdk/constants.ts#L24
const SEPOLIA_SAFE_ERC_7579_LAUNCHPAD_ADDRESS: Address =
    address!("EBe001b3D534B9B6E2500FB78E67a1A137f561CE");
const SEPOLIA_SAFE_4337_MODULE_ADDRESS: Address =
    address!("3Fdb5BC686e861480ef99A6E3FaAe03c0b9F32e2");

// https://github.com/pimlicolabs/permissionless.js/blob/b8798c121eecba6a71f96f8ddf8e0ad2e98a3236/packages/permissionless/accounts/safe/toSafeSmartAccount.ts#L438C36-L438C76
const SAFE_MULTI_SEND_ADDRESS: Address =
    address!("38869bf66a61cF6bDB996A6aE40D5853Fd43B526");

// https://github.com/safe-global/safe-modules-deployments/blob/d6642d90659de19e54bb4a20d646b30bd0a51885/src/assets/safe-4337-module/v0.3.0/safe-module-setup.json#L7
// https://github.com/pimlicolabs/permissionless.js/blob/b8798c121eecba6a71f96f8ddf8e0ad2e98a3236/packages/permissionless/accounts/safe/toSafeSmartAccount.ts#L431
const SAFE_MODULE_SETUP_ADDRESS: Address =
    address!("2dd68b007B46fBe91B9A7c3EDa5A7a1063cB5b47");

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

// https://github.com/WalletConnect/secure-web3modal/blob/c19a1e7b21c6188261728f4d521a17f94da4f055/src/core/SmartAccountSdk/constants.ts#L10
// const APPKIT_SALT: U256 = U256::from_str("zg3ijy0p46");

fn encode_internal_transaction(
    to: Address,
    data: Vec<u8>,
    value: U256,
    operation: bool,
) -> Bytes {
    // https://github.com/pimlicolabs/permissionless.js/blob/b8798c121eecba6a71f96f8ddf8e0ad2e98a3236/packages/permissionless/accounts/safe/toSafeSmartAccount.ts#L486
    DynSolValue::Tuple(vec![
        DynSolValue::Uint(U256::from(if operation { 1 } else { 0 }), 8),
        DynSolValue::Address(to),
        DynSolValue::Uint(value, 256),
        DynSolValue::Uint(U256::from(data.len()), 256),
        DynSolValue::Bytes(data),
    ])
    .abi_encode()
    .into()
}

fn init_code_call_data(
    owner: Address,
) -> SafeProxyFactory::createProxyWithNonceCall {
    // https://github.com/pimlicolabs/permissionless.js/blob/b8798c121eecba6a71f96f8ddf8e0ad2e98a3236/packages/permissionless/accounts/safe/toSafeSmartAccount.ts#L714C31-L714C46
    let enable_modules = SafeModuleSetup::enableModulesCall {
        modules: vec![SEPOLIA_SAFE_4337_MODULE_ADDRESS],
    }
    .abi_encode();

    // https://github.com/pimlicolabs/permissionless.js/blob/b8798c121eecba6a71f96f8ddf8e0ad2e98a3236/packages/permissionless/accounts/safe/toSafeSmartAccount.ts#L486
    let txn = encode_internal_transaction(
        SAFE_MODULE_SETUP_ADDRESS,
        enable_modules,
        U256::ZERO,
        true,
    ); // TODO join any setupTransactions

    let multi_send_call_data =
        MultiSend::multiSendCall { transactions: txn }.abi_encode().into();

    // https://github.com/pimlicolabs/permissionless.js/blob/b8798c121eecba6a71f96f8ddf8e0ad2e98a3236/packages/permissionless/accounts/safe/toSafeSmartAccount.ts#L728
    let initializer = Safe::setupCall {
        _owners: vec![owner],
        _threshold: U256::from(1),
        to: SAFE_MULTI_SEND_ADDRESS,
        data: multi_send_call_data,
        fallbackHandler: SAFE_MODULE_SETUP_ADDRESS,
        paymentToken: Address::ZERO,
        payment: U256::ZERO,
        paymentReceiver: Address::ZERO,
    }
    .abi_encode()
    .into();
    // https://github.com/pimlicolabs/permissionless.js/blob/b8798c121eecba6a71f96f8ddf8e0ad2e98a3236/packages/permissionless/accounts/safe/toSafeSmartAccount.ts#L840
    SafeProxyFactory::createProxyWithNonceCall {
        _singleton: SEPOLIA_SAFE_ERC_7579_LAUNCHPAD_ADDRESS,
        initializer,
        saltNonce: U256::ZERO,
    }
}
