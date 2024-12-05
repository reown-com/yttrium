// Based off: https://github.com/rhinestonewtf/module-sdk-tutorials/blob/main/src/smart-sessions/permissionless-safe-7702.ts

use {
    crate::{
        config::Config,
        erc7579::{
            ownable_validator::{encode_owners, OWNABLE_VALIDATOR_ADDRESS},
            policy::get_sudo_policy,
            smart_sessions::{ActionData, ERC7739Data, Session},
        },
        smart_accounts::safe::Owners,
    },
    alloy::{
        network::Ethereum,
        primitives::{address, fixed_bytes, B256},
        rpc::types::Authorization,
        signers::local::LocalSigner,
    },
    alloy_provider::{Provider, ReqwestProvider},
};

#[tokio::test]
async fn test() {
    let config = Config::local();
    let provider = ReqwestProvider::<Ethereum>::new_http(
        config.endpoints.rpc.base_url.parse().unwrap(),
    );

    let owner = LocalSigner::random();
    let _safe_owner = LocalSigner::random();

    // TODO ownableValidator
    // https://github.com/rhinestonewtf/module-sdk-tutorials/blob/656c52e200329c40ce633485bb8824df6c96ba20/src/smart-sessions/permissionless-safe-7702.ts#L80
    // https://github.com/rhinestonewtf/module-sdk/blob/main/src/module/ownable-validator/installation.ts
    let _owner_validator = ();

    let session_owner = LocalSigner::random();

    let _session = Session {
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

    let _auth_7702 = Authorization {
        chain_id: provider.get_chain_id().await.unwrap(),
        address: address!("29fcB43b46531BcA003ddC8FCB67FFE91900C762"), /* TODO make constant */
        // TODO should this be `pending` tag? https://github.com/wevm/viem/blob/a49c100a0b2878fbfd9f1c9b43c5cc25de241754/src/experimental/eip7702/actions/signAuthorization.ts#L149
        nonce: provider.get_transaction_count(owner.address()).await.unwrap(),
    };

    // Sign the authorization
    // let sig = owner.sign_hash_sync(&auth_7702.signature_hash())?;
    // let auth = auth_7702.into_signed(sig);

    // let authorization_list = vec![Authorization {
    //     contract_address: auth.address,
    //     chain_id: u64::from_be_bytes(
    //         U64::from(auth.chain_id).to_be_bytes(),
    //     ),
    //     nonce: auth.nonce,
    //     y_parity: auth.y_parity(),
    //     r: auth.r(),
    //     s: auth.s(),
    // }];
}
