use {
    super::module::{Module, ModuleType},
    alloy::{
        primitives::{Address, B256, Bytes, FixedBytes, address, keccak256},
        sol,
        sol_types::SolValue,
    },
};

// https://github.com/erc7579/smartsessions/blob/main/contracts/DataTypes.sol
sol! {
    struct ChainDigest {
        uint64 chainId;
        bytes32 sessionDigest;
    }

    struct PolicyData {
        address policy;
        bytes initData;
    }

    struct ERC7739Context {
        // we can not use a detailed EIP712Domain struct here.
        // EIP712 specifies: Protocol designers only need to include the fields that make sense for their signing domain.
        // Unused fields are left out of the struct type.
        bytes32 appDomainSeparator;
        string[] contentNames;
    }

    struct ERC7739Data {
        ERC7739Context[] allowedERC7739Content;
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
        // when setting `permitERC4337Paymaster` to true, the length of `userOpPolicies` needs to be at least 1
        bool permitERC4337Paymaster;
    }

    struct EnableSession {
        uint8 chainDigestIndex;
        ChainDigest[] hashesAndChainIds;
        Session sessionToEnable;
        // in order to enable a session, the smart account has to sign a digest. The signature for this is stored here.
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
    let use_registry = true;
    Module {
        address: SMART_SESSIONS_ADDRESS,
        module: SMART_SESSIONS_ADDRESS,
        init_data: (
            FixedBytes::from(
                if use_registry {
                    SmartSessionMode::Enable
                } else {
                    SmartSessionMode::UnsafeEnable
                }
                .to_u8(),
            ),
            sessions.abi_encode_params(),
        )
            .abi_encode_packed()
            .into(),
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
