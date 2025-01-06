use {
    super::module::{Module, ModuleType},
    alloy::{
        primitives::{address, keccak256, Address, Bytes, FixedBytes, B256},
        sol,
        sol_types::SolValue,
    },
};

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
        bool permitERC4337Paymaster;
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

// https://github.com/rhinestonewtf/module-sdk/blob/main/src/module/smart-sessions/constants.ts#L3
pub const SMART_SESSIONS_ADDRESS: Address =
    address!("00000000002B0eCfbD0496EE71e01257dA0E37DE");

// https://github.com/rhinestonewtf/module-sdk/blob/1f2f2c5380614ad07b6e1ccbb5a9ed55374c673c/src/module/smart-sessions/installation.ts#L12
pub fn get_smart_sessions_validator(
    sessions: &[Session],
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

pub fn get_permission_id(session: &Session) -> B256 {
    keccak256(
        (
            &session.sessionValidator,
            &session.sessionValidatorInitData,
            &session.salt,
        )
            .abi_encode_params(),
    )
}

#[derive(Debug, PartialEq)]
pub enum SmartSessionMode {
    Use,
    Enable,
    UnsafeEnable,
}

impl SmartSessionMode {
    pub fn to_u8(&self) -> u8 {
        match self {
            // https://github.com/rhinestonewtf/module-sdk/blob/18ef7ca998c0d0a596572f18575e1b4967d9227b/src/module/smart-sessions/types.ts#L42
            Self::Use => 0x00,
            Self::Enable => 0x01,
            Self::UnsafeEnable => 0x02,
        }
    }

    pub fn from_u8(value: &u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Use),
            0x01 => Some(Self::Enable),
            0x02 => Some(Self::UnsafeEnable),
            _ => None,
        }
    }
}

impl TryFrom<&u8> for SmartSessionMode {
    type Error = &'static str;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        Self::from_u8(value).ok_or("invalid SmartSessionMode")
    }
}

pub fn encode_use_signature(permission_id: B256, signature: Bytes) -> Bytes {
    encode_smart_session_signature(
        &SmartSessionMode::Use,
        permission_id,
        signature,
    )
}

fn encode_smart_session_signature(
    mode: &SmartSessionMode,
    permission_id: B256,
    signature: Bytes,
) -> Bytes {
    match mode {
        SmartSessionMode::Use => {
            (FixedBytes::from(mode.to_u8()), permission_id, signature)
                .abi_encode_packed()
                .into()
        }
        _ => unimplemented!("mode: {mode:?}"),
    }
}
