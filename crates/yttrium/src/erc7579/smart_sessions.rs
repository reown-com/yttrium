use alloy::{
    primitives::{address, Address},
    sol,
    sol_types::SolValue,
};

use super::module::{Module, ModuleType};

sol! {
    struct ChainDigest {
        uint64 chainId;
        bytes32 sessionDigest;
    }

    struct PolicyData {
        address policy;
        bytes initData;
    }

    struct ERC7739Data {
        string[] allowedERC7739Content;
        PolicyData[] erc1271Policies;
    }

    struct ActionData {
        bytes4 actionTargetSelector;
        address actionTarget;
        PolicyData[] actionPolicies;
    }

    struct Session {
        address sessionValidator;
        bytes sessionValidatorInitData;
        bytes32 salt;
        PolicyData[] userOpPolicies;
        ERC7739Data erc7739Policies;
        ActionData[] actions;
    }

    // https://github.com/erc7579/smartsessions/blob/b1624f851f56ec67cc677dce129e9caa12fcafd9/contracts/DataTypes.sol#L14
    struct EnableSession {
        uint8 chainDigestIndex;
        ChainDigest[] hashesAndChainIds;
        Session sessionToEnable;
        bytes permissionEnableSig;
    }

    function enableSessionSig(EnableSession session, bytes signature);
}

sol! {
    type PermissionId is bytes32;

    #[sol(rpc)]
    contract ISmartSession {
        function isPermissionEnabled(PermissionId permissionId, address account) external view returns (bool);
    }
}

pub const SMART_SESSIONS_ADDRESS: Address =
    address!("DDFF43A42726df11E34123f747bDce0f755F784d");

// https://github.com/rhinestonewtf/module-sdk/blob/1f2f2c5380614ad07b6e1ccbb5a9ed55374c673c/src/module/smart-sessions/installation.ts#L12
pub fn get_smart_sessions_validator(
    sessions: Vec<Session>,
    hook: Option<Address>,
) -> Module {
    Module {
        address: SMART_SESSIONS_ADDRESS,
        module: SMART_SESSIONS_ADDRESS,
        init_data: sessions.abi_encode_params().into(),
        de_init_data: Default::default(),
        additional_context: Default::default(),
        hook,
        r#type: ModuleType::Validator,
    }
}
