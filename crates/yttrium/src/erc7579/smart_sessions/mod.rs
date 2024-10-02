use alloy::sol;

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
        function isSessionEnabled(PermissionId permissionId, address account) external view returns (bool);
    }
}
