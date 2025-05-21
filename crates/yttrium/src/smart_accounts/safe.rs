use {
    crate::{
        bundler::pimlico::paymaster::client::PaymasterClient,
        call::{
            send::safe_test::{
                encode_send_transactions, prepare_send_transactions_inner,
                DoSendTransactionParams, OwnerSignature,
                PreparedSendTransaction,
            },
            Call,
        },
        entry_point::{
            EntryPoint::{self, PackedUserOperation},
            ENTRYPOINT_ADDRESS_V07,
        },
        erc7579::addresses::RHINESTONE_ATTESTER_ADDRESS,
        smart_accounts::account_address::AccountAddress,
        user_operation::{
            hash::pack_v07::{
                combine::combine_and_trim_first_16_bytes,
                hashed_paymaster_and_data::get_data,
            },
            UserOperationV07,
        },
    },
    alloy::{
        dyn_abi::{DynSolValue, Eip712Domain},
        primitives::{
            address, aliases::U48, bytes, keccak256, Address, Bytes,
            FixedBytes, Uint, B256, U128, U256, U64,
        },
        providers::Provider,
        sol,
        sol_types::{SolCall, SolValue},
    },
    erc6492::create::create_erc6492_signature,
    serde::{Deserialize, Serialize},
};

sol! {
    #[sol(rpc)]
    contract SafeProxyFactory {
        function proxyCreationCode() returns (bytes memory);
        function createProxyWithNonce(address _singleton, bytes memory initializer, uint256 saltNonce) returns (address proxy);
    }
}

sol! {
    contract Safe7579Launchpad {
        struct ModuleInit {
            address module;
            bytes initData;
        }

        struct InitData {
            address singleton;
            address[] owners;
            uint256 threshold;
            address setupTo;
            bytes setupData;
            address safe7579;
            ModuleInit[] validators;
            bytes callData;
        }

        function initSafe7579(
            address safe7579,
            ModuleInit[] calldata executors,
            ModuleInit[] calldata fallbacks,
            ModuleInit[] calldata hooks,
            address[] calldata attesters,
            uint8 threshold
        );

        function addSafe7579(
            address safe7579,
            ModuleInit[] calldata validators,
            ModuleInit[] calldata executors,
            ModuleInit[] calldata fallbacks,
            ModuleInit[] calldata hooks,
            address[] calldata attesters,
            uint8 threshold
        );

        function setupSafe(InitData calldata initData);

        function preValidationSetup(
            bytes32 initHash,
            address to,
            bytes calldata preInit
        );
    }
}

sol! {
    contract Safe7579 {
        type ModeCode is bytes32;

        function execute(
            ModeCode mode,
            bytes calldata executionCalldata
        );
    }
}

sol!(
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
    struct SafeOp {
        address safe;
        uint256 nonce;
        bytes initCode;
        bytes callData;
        uint128 verificationGasLimit;
        uint128 callGasLimit;
        uint256 preVerificationGas;
        uint128 maxPriorityFeePerGas;
        uint128 maxFeePerGas;
        bytes paymasterAndData;
        uint48 validAfter;
        uint48 validUntil;
        address entryPoint;
    }
);

sol!(
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct SafeMessage {
        bytes message;
    }
);

// https://github.com/WalletConnect/secure-web3modal/blob/f1d16f973a313e598d124a0e4751aee12d5de628/src/core/SmartAccountSdk/utils.ts#L180
pub const SAFE_ERC_7579_LAUNCHPAD_ADDRESS: Address =
    // address!("EBe001b3D534B9B6E2500FB78E67a1A137f561CE"); // old version
    address!("7579011aB74c46090561ea277Ba79D510c6C00ff");
// https://github.com/WalletConnect/secure-web3modal/blob/f1d16f973a313e598d124a0e4751aee12d5de628/src/core/SmartAccountSdk/utils.ts#L181
// https://docs.pimlico.io/permissionless/how-to/accounts/use-erc7579-account
// https://docs.safe.global/advanced/erc-7579/tutorials/7579-tutorial
pub const SAFE_4337_MODULE_ADDRESS: Address =
    // address!("75cf11467937ce3F2f357CE24ffc3DBF8fD5c226"); // this is the safe 4337 module, not the one for 7579 (https://reown-inc.slack.com/archives/C077RPLSZ71/p1733866031056889?thread_ts=1729617897.410709&cid=C077RPLSZ71): https://github.com/safe-global/safe-modules/blob/d4f59362e9b16291feb88f14090fcf2311686e74/modules/4337/CHANGELOG.md?plain=1#L28
    address!("7579EE8307284F293B1927136486880611F20002"); // what recent docs use
                                                          // address!("3Fdb5BC686e861480ef99A6E3FaAe03c0b9F32e2"); // old version

// https://github.com/safe-global/safe-smart-account/blob/main/CHANGELOG.md#expected-addresses-with-safe-singleton-factory-2
pub const SAFE_SINGLETON_1_4_1: Address =
    address!("41675C099F32341bf84BFc5382aF534df5C7461a");
pub const SAFE_L2_SINGLETON_1_4_1: Address =
    address!("29fcB43b46531BcA003ddC8FCB67FFE91900C762");
pub const SAFE_PROXY_FACTORY_1_4_1: Address =
    address!("4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67");

// https://github.com/safe-global/safe-smart-account/blob/main/CHANGELOG.md#expected-addresses-with-safe-singleton-factory-1
pub const SAFE_SINGLETON_1_5_0: Address =
    address!("36F86745986cB49C0e8cB38f14e948bb82d8d1A8");
pub const SAFE_L2_SINGLETON_1_5_0: Address =
    address!("0200216A26588315deD46e5451388DfC358A5bD4");
pub const SAFE_PROXY_FACTORY_1_5_0: Address =
    address!("8b24df6da67319eE9638a798660547D67a29f4ce");

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

pub const DUMMY_SIGNATURE: Bytes = bytes!("000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");

// https://github.com/WalletConnect/secure-web3modal/blob/c19a1e7b21c6188261728f4d521a17f94da4f055/src/core/SmartAccountSdk/constants.ts#L10
// const APPKIT_SALT: U256 = U256::from_str("zg3ijy0p46");

pub fn init_data() -> Safe7579Launchpad::initSafe7579Call {
    Safe7579Launchpad::initSafe7579Call {
        safe7579: SAFE_4337_MODULE_ADDRESS,
        executors: vec![],
        fallbacks: vec![],
        hooks: vec![],
        attesters: vec![
            RHINESTONE_ATTESTER_ADDRESS,
            // MOCK_ATTESTER_ADDRESS
        ],
        threshold: 1,
    }
}

#[derive(Debug, Clone)]
pub struct Owners {
    pub owners: Vec<Address>,
    pub threshold: u8,
}

sol! {
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    contract SetupContract {
        function setup(
            address[] calldata _owners,
            uint256 _threshold,
            address to,
            bytes calldata data,
            address fallbackHandler,
            address paymentToken,
            uint256 payment,
            address paymentReceiver
        ) external;
    }
}

sol! {
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    contract AddSafe7579Contract {
        struct ModuleInit {
            address module;
            bytes initData;
        }

        function addSafe7579(address safe7579, ModuleInit[] calldata validators, ModuleInit[] calldata executors, ModuleInit[] calldata fallbacks, ModuleInit[] calldata hooks, address[] calldata attesters, uint8 threshold) external;
    }
}

// permissionless -> getInitializerCode
fn get_initializer_code(owners: Owners) -> Bytes {
    // let ownable_validator = get_ownable_validator(&owners, None);
    let init_hash = keccak256(
        DynSolValue::Tuple(vec![
            DynSolValue::Address(SAFE_SINGLETON_1_4_1),
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
            DynSolValue::Array(vec![
            //     DynSolValue::CustomStruct {
            //     name: "ModuleInit".to_owned(),
            //     prop_names: vec!["module".to_owned(), "initData".to_owned()],
            //     tuple: vec![
            //         DynSolValue::Address(ownable_validator.address),
            //         DynSolValue::Bytes(ownable_validator.init_data.into()),
            //     ],
            // }
            ]),
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

pub async fn get_account_address(
    provider: impl Provider,
    owners: Owners,
) -> AccountAddress {
    let creation_code =
        SafeProxyFactory::new(SAFE_PROXY_FACTORY_1_4_1, provider)
            .proxyCreationCode()
            .call()
            .await
            .unwrap()
            ._0;
    let initializer = get_initializer_code(owners.clone());
    let deployment_code = DynSolValue::Tuple(vec![
        DynSolValue::Bytes(creation_code.to_vec()),
        DynSolValue::FixedBytes(
            SAFE_ERC_7579_LAUNCHPAD_ADDRESS.into_word(),
            32,
        ),
    ])
    .abi_encode_packed();
    let salt = keccak256(
        DynSolValue::Tuple(vec![
            DynSolValue::FixedBytes(
                keccak256(initializer.abi_encode_packed()),
                32,
            ),
            DynSolValue::Uint(Uint::from(0), 256),
        ])
        .abi_encode_packed(),
    );
    SAFE_PROXY_FACTORY_1_4_1.create2(salt, keccak256(deployment_code)).into()
}

pub fn get_call_data(calls: Vec<Call>) -> Bytes {
    get_call_data_with_try(calls, false)
}

pub fn get_call_data_with_try(calls: Vec<Call>, exec_type: bool) -> Bytes {
    let batch = calls.len() != 1;
    let selector = [0u8; 4];
    let context = [0u8; 22];

    let mode = DynSolValue::Tuple(vec![
        DynSolValue::Uint(Uint::from(if batch { 0x01 } else { 0x00 }), 8), // DelegateCall is 0xFF
        DynSolValue::Uint(Uint::from(exec_type as u8), 8), // revertOnError in permissionless
        DynSolValue::Bytes(vec![0u8; 4]),
        DynSolValue::Bytes(selector.to_vec()),
        DynSolValue::Bytes(context.to_vec()),
    ])
    .abi_encode_packed();

    Safe7579::executeCall {
        mode: FixedBytes::from_slice(&mode),
        executionCalldata: encode_calls(calls),
    }
    .abi_encode()
    .into()
}

sol! {
    function executionBatch((address, uint256, bytes)[]);
}

fn encode_calls(calls: Vec<Call>) -> Bytes {
    fn call(call: Call) -> (Address, U256, Bytes) {
        (call.to, call.value, call.input)
    }

    let tuples = calls.into_iter().map(call).collect::<Vec<_>>();
    if tuples.len() == 1 {
        tuples.abi_encode_packed()
    } else {
        let call = executionBatchCall { _0: tuples };

        // encode without selector
        let mut out = Vec::with_capacity(call.abi_encoded_size());
        call.abi_encode_raw(&mut out);
        out
    }
    .into()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedSignature {
    pub safe_message: SafeMessage,
    pub domain: Eip712Domain,
}

pub fn prepare_sign(
    account_address: AccountAddress,
    chain_id: U256,
    message_hash: B256,
) -> PreparedSignature {
    let safe_message = SafeMessage { message: message_hash.into() };
    let domain = Eip712Domain {
        chain_id: Some(chain_id),
        verifying_contract: Some(account_address.to_address()),
        ..Default::default()
    };
    PreparedSignature { safe_message, domain }
}

#[allow(clippy::large_enum_variant)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum SignOutputEnum {
    Signature(Bytes),
    // renamed to `Object` to avoid conflicts: https://github.com/mozilla/uniffi-rs/issues/2402
    SignOutput(SignOutputObject),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SignOutputObject {
    pub to_sign: SignOutputToSign,
    pub sign_step_3_params: SignStep3Params,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SignOutputToSign {
    pub hash: B256,
    pub safe_op: SafeOp,
    pub domain: Eip712Domain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SignStep3Params {
    pub signature: Bytes,
    pub do_send_transaction_params: DoSendTransactionParams,
}

// TODO refactor to make account_address optional, if not provided it will
// determine it based on Owners TODO refactor to make owners optional, in the
// case where it already being deployed is assumed
pub async fn sign(
    owners: Owners,
    account_address: AccountAddress,
    signatures: Vec<OwnerSignature>,
    provider: &impl Provider,
    paymaster_client: PaymasterClient,
) -> eyre::Result<SignOutputEnum> {
    if signatures.len() > 1 {
        unimplemented!("multi-signature is not yet supported");
    }

    let signature = Bytes::from(signatures[0].signature.as_bytes());

    // Null validator address for regular Safe signature
    let signature = (Address::ZERO, signature).abi_encode_packed().into();

    if provider.get_code_at(account_address.into()).await?.is_empty() {
        let eip1559_est = provider.estimate_eip1559_fees(None).await?;
        let PreparedSendTransaction {
            safe_op,
            domain,
            hash,
            do_send_transaction_params,
        } = prepare_send_transactions_inner(
            vec![],
            owners,
            Some(account_address),
            None,
            provider,
            U128::from(eip1559_est.max_fee_per_gas).to(),
            U128::from(eip1559_est.max_priority_fee_per_gas).to(),
            paymaster_client,
        )
        .await?;

        Ok(SignOutputEnum::SignOutput(SignOutputObject {
            to_sign: SignOutputToSign { hash, safe_op, domain },
            sign_step_3_params: SignStep3Params {
                signature,
                do_send_transaction_params,
            },
        }))
    } else {
        Ok(SignOutputEnum::Signature(signature))
    }
}

pub async fn sign_step_3(
    user_op_signature: Vec<OwnerSignature>,
    sign_step_3_params: SignStep3Params,
) -> eyre::Result<Bytes> {
    let user_op = encode_send_transactions(
        user_op_signature,
        sign_step_3_params.do_send_transaction_params,
    )
    .await?;

    let factory_address = ENTRYPOINT_ADDRESS_V07;
    let factory_data = EntryPoint::handleOpsCall {
        ops: vec![PackedUserOperation {
            paymasterAndData: get_data(&user_op),
            sender: user_op.sender.into(),
            nonce: user_op.nonce,
            initCode: [
                // TODO refactor to remove unwrap()
                // This code double-checks for code deployed unnecessesarly
                // (i.e. here and inside prepare_send_transactions_inner())
                user_op.factory.unwrap().to_vec().into(),
                user_op.factory_data.unwrap(),
            ]
            .concat()
            .into(),
            callData: user_op.call_data,
            accountGasLimits: combine_and_trim_first_16_bytes(
                user_op.verification_gas_limit,
                user_op.call_gas_limit,
            ),
            preVerificationGas: user_op.pre_verification_gas,
            gasFees: combine_and_trim_first_16_bytes(
                user_op.max_priority_fee_per_gas,
                user_op.max_fee_per_gas,
            ),
            signature: user_op.signature,
        }],
        beneficiary: user_op.sender.into(),
    }
    .abi_encode()
    .into();

    Ok(create_erc6492_signature(
        factory_address,
        factory_data,
        sign_step_3_params.signature,
    ))
}

pub fn user_operation_to_safe_op(
    user_op: &UserOperationV07,
    entrypoint: Address,
    chain_id: U64,
    valid_after: U48,
    valid_until: U48,
) -> (SafeOp, Eip712Domain) {
    // TODO handle panic
    fn coerce_u256_to_u128(u: U256) -> U128 {
        U128::from(u)
    }

    let safe_op = SafeOp {
        safe: user_op.sender.into(),
        callData: user_op.call_data.clone(),
        nonce: user_op.nonce,
        initCode: user_op
            .factory
            .map(|factory| {
                factory
                    .into_iter()
                    .chain(user_op.factory_data.clone().unwrap())
                    .collect()
            })
            .unwrap_or_default(),
        maxFeePerGas: u128::from_be_bytes(
            coerce_u256_to_u128(user_op.max_fee_per_gas).to_be_bytes(),
        ),
        maxPriorityFeePerGas: u128::from_be_bytes(
            coerce_u256_to_u128(user_op.max_priority_fee_per_gas).to_be_bytes(),
        ),
        preVerificationGas: user_op.pre_verification_gas,
        verificationGasLimit: u128::from_be_bytes(
            coerce_u256_to_u128(user_op.verification_gas_limit).to_be_bytes(),
        ),
        callGasLimit: u128::from_be_bytes(
            coerce_u256_to_u128(user_op.call_gas_limit).to_be_bytes(),
        ),
        // signerToSafeSmartAccount -> getPaymasterAndData
        paymasterAndData: user_op
            .paymaster
            .map(|paymaster| {
                [
                    paymaster.to_vec(),
                    coerce_u256_to_u128(
                        user_op
                            .paymaster_verification_gas_limit
                            .unwrap_or(Uint::from(0)),
                    )
                    .to_be_bytes_vec(),
                    coerce_u256_to_u128(
                        user_op
                            .paymaster_post_op_gas_limit
                            .unwrap_or(Uint::from(0)),
                    )
                    .to_be_bytes_vec(),
                    user_op.paymaster_data.clone().unwrap_or_default().to_vec(),
                ]
                .concat()
                .into()
            })
            .unwrap_or_default(),
        validAfter: valid_after,
        validUntil: valid_until,
        entryPoint: entrypoint,
    };

    // This is always the Safe 4337 module address now: https://reown-inc.slack.com/archives/C077RPLSZ71/p1733864707609549?thread_ts=1729617897.410709&cid=C077RPLSZ71
    // let erc7579_launchpad_address = true;
    // let verifying_contract = if erc7579_launchpad_address && !deployed {
    //     user_op.sender.into()
    // } else {
    //     SAFE_4337_MODULE_ADDRESS
    // };
    let verifying_contract = SAFE_4337_MODULE_ADDRESS;

    let domain = Eip712Domain {
        chain_id: Some(Uint::from(chain_id)),
        verifying_contract: Some(verifying_contract),
        ..Default::default()
    };
    (safe_op, domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_execution_call_data() {
        assert_eq!(encode_calls(vec![]), bytes!("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000"));
    }

    #[test]
    fn single_execution_call_data_value() {
        assert_eq!(
            encode_calls(vec![Call {
                to: address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                value: U256::from(19191919),
                input: bytes!(""),
            }]),
            bytes!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa000000000000000000000000000000000000000000000000000000000124d86f")
        );
    }

    #[test]
    fn single_execution_call_data_data() {
        assert_eq!(
            encode_calls(vec![Call {
                to: address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                value: U256::ZERO,
                input: bytes!("7777777777777777"),
            }]),
            bytes!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa00000000000000000000000000000000000000000000000000000000000000007777777777777777")
        );
    }

    #[test]
    fn two_execution_call_data() {
        assert_eq!(
            encode_calls(vec![Call {
                to: address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                value: U256::from(19191919),
                input: bytes!(""),
            }, Call {
                to: address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                value: U256::ZERO,
                input: bytes!("7777777777777777"),
            }]),
            bytes!("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa000000000000000000000000000000000000000000000000000000000124d86f00000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000087777777777777777000000000000000000000000000000000000000000000000")
        );
    }

    #[test]
    fn empty_call_data() {
        assert_eq!(get_call_data(vec![]), bytes!("e9ae5c5301000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000"));
    }

    #[test]
    fn single_call_data_value() {
        assert_eq!(
            get_call_data(vec![Call {
                to: address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                value: U256::from(19191919),
                input: bytes!(""),
            }]),
            bytes!("e9ae5c53000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000034aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa000000000000000000000000000000000000000000000000000000000124d86f000000000000000000000000")
        );
    }

    #[test]
    fn single_call_data_data() {
        assert_eq!(
            get_call_data(vec![Call {
                to: address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                value: U256::ZERO,
                input: bytes!("7777777777777777"),
            }]),
            bytes!("e9ae5c5300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000003caaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0000000000000000000000000000000000000000000000000000000000000000777777777777777700000000")
        );
    }

    #[test]
    fn two_call_data() {
        assert_eq!(
            get_call_data(vec![Call {
                to: address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                value: U256::from(19191919),
                input: bytes!(""),
            }, Call {
                to: address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                value: U256::ZERO,
                input: bytes!("7777777777777777"),
            }]),
            bytes!("e9ae5c530100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa000000000000000000000000000000000000000000000000000000000124d86f00000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000087777777777777777000000000000000000000000000000000000000000000000")
        );
    }
}
