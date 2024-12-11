// Based off: https://github.com/rhinestonewtf/module-sdk-tutorials/blob/main/src/smart-sessions/permissionless-safe-7702.ts

use {
    crate::{
        config::Config,
        erc7579::{
            addresses::{MOCK_ATTESTER_ADDRESS, RHINESTONE_ATTESTER_ADDRESS},
            ownable_validator::{
                encode_owners, get_ownable_validator, OWNABLE_VALIDATOR_ADDRESS,
            },
            policy::get_sudo_policy,
            smart_sessions::{
                get_smart_sessions_validator, ActionData, ERC7739Data, Session,
            },
        },
        smart_accounts::safe::{
            Owners, SAFE_4337_MODULE_ADDRESS, SAFE_ERC_7579_LAUNCHPAD_ADDRESS,
            SAFE_L2_SINGLETON_1_4_1,
        },
        test_helpers::anvil_faucet,
    },
    alloy::{
        network::{Ethereum, EthereumWallet, TransactionBuilder7702},
        primitives::{address, fixed_bytes, Address, B256, U256},
        rpc::types::Authorization,
        signers::{local::LocalSigner, SignerSync},
        sol,
        sol_types::SolCall,
    },
    alloy_provider::{Provider, ProviderBuilder, ReqwestProvider},
    reqwest::Url,
};

#[tokio::test]
async fn test() {
    let config = Config::local();
    let rpc_url = config.endpoints.rpc.base_url.parse::<Url>().unwrap();
    let provider = ReqwestProvider::<Ethereum>::new_http(rpc_url.clone());

    let owner = LocalSigner::random();
    let safe_owner = LocalSigner::random();
    let owners = Owners { threshold: 1, owners: vec![safe_owner.address()] };

    // TODO ownableValidator
    // https://github.com/rhinestonewtf/module-sdk-tutorials/blob/656c52e200329c40ce633485bb8824df6c96ba20/src/smart-sessions/permissionless-safe-7702.ts#L80
    // https://github.com/rhinestonewtf/module-sdk/blob/main/src/module/ownable-validator/installation.ts
    let ownable_validator = get_ownable_validator(&owners, None);

    let session_owner = LocalSigner::random();

    let session = Session {
        sessionValidator: OWNABLE_VALIDATOR_ADDRESS,
        sessionValidatorInitData: encode_owners(&Owners {
            threshold: 1,
            owners: vec![session_owner.address()],
        }),
        salt: B256::default(),
        userOpPolicies: vec![],
        erc7739Policies: ERC7739Data {
            allowedERC7739Content: vec![],
            erc1271Policies: vec![],
        },
        actions: vec![ActionData {
            actionTarget: address!("a564cB165815937967a7d018B7F34B907B52fcFd"), /* an address as the target of the session execution */
            actionTargetSelector: fixed_bytes!("00000000"), /* function selector to be used in the execution, in this case no function selector is used */
            actionPolicies: vec![get_sudo_policy()],
        }],
    };

    let smart_sessions = get_smart_sessions_validator(vec![session], None);

    let auth_7702 = Authorization {
        chain_id: provider.get_chain_id().await.unwrap(),
        address: SAFE_L2_SINGLETON_1_4_1,
        // TODO should this be `pending` tag? https://github.com/wevm/viem/blob/a49c100a0b2878fbfd9f1c9b43c5cc25de241754/src/experimental/eip7702/actions/signAuthorization.ts#L149
        nonce: provider.get_transaction_count(owner.address()).await.unwrap(),
    };

    // Sign the authorization
    let sig = owner.sign_hash_sync(&auth_7702.signature_hash()).unwrap();
    let auth = auth_7702.into_signed(sig);

    sol! {
        #[allow(clippy::too_many_arguments)]
        #[sol(rpc)]
        contract SetupContract {
            function setup(address[] calldata _owners,uint256 _threshold,address to,bytes calldata data,address fallbackHandler,address paymentToken,uint256 payment, address paymentReceiver) external;
        }

        #[allow(clippy::too_many_arguments)]
        #[sol(rpc)]
        contract AddSafe7579Contract {
            struct ModuleInit {
                address module;
                bytes initData;
            }

            function addSafe7579(address safe7579, ModuleInit[] calldata validators, ModuleInit[] calldata executors, ModuleInit[] calldata fallbacks, ModuleInit[] calldata hooks, address[] calldata attesters, uint8 threshold) external;
        }
    };

    let faucet = anvil_faucet(rpc_url).await;
    let wallet = EthereumWallet::new(faucet);
    let wallet_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_provider(provider.clone());
    assert!(SetupContract::new(owner.address(), wallet_provider.clone())
        .setup(
            owners.owners,
            U256::from(owners.threshold),
            SAFE_ERC_7579_LAUNCHPAD_ADDRESS,
            AddSafe7579Contract::addSafe7579Call {
                safe7579: SAFE_4337_MODULE_ADDRESS,
                validators: vec![
                    AddSafe7579Contract::ModuleInit {
                        module: ownable_validator.address,
                        initData: ownable_validator.init_data,
                    },
                    AddSafe7579Contract::ModuleInit {
                        module: smart_sessions.address,
                        initData: smart_sessions.init_data,
                    },
                ],
                executors: vec![],
                fallbacks: vec![],
                hooks: vec![],
                attesters: vec![
                    RHINESTONE_ATTESTER_ADDRESS,
                    MOCK_ATTESTER_ADDRESS,
                ],
                threshold: owners.threshold,
            }
            .abi_encode()
            .into(),
            SAFE_4337_MODULE_ADDRESS,
            Address::ZERO,
            U256::ZERO,
            Address::ZERO,
        )
        .map(|mut t| {
            t.set_authorization_list(vec![auth]);
            t
        })
        .send()
        .await
        .unwrap()
        .get_receipt()
        .await
        .unwrap()
        .status());

    // TODO toSafeSmartAccount & more
    // https://github.com/rhinestonewtf/module-sdk-tutorials/blob/5592c407865122e04fb234b6a1533712e2f47d39/src/smart-sessions/permissionless-safe-7702.ts#L170
}
