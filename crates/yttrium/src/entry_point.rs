use crate::chain::ChainId;
use alloy::sol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryPointAddress(alloy::primitives::Address);

impl EntryPointAddress {
    pub fn new(address: alloy::primitives::Address) -> Self {
        Self(address)
    }

    pub fn to_address(&self) -> alloy::primitives::Address {
        self.into()
    }
}

impl std::fmt::Display for EntryPointAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<EntryPointAddress> for alloy::primitives::Address {
    fn from(val: EntryPointAddress) -> Self {
        val.0
    }
}

impl From<&EntryPointAddress> for alloy::primitives::Address {
    fn from(val: &EntryPointAddress) -> Self {
        val.0
    }
}

pub const ENTRYPOINT_ADDRESS_V06: &str =
    "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789";
pub const ENTRYPOINT_ADDRESS_V07: &str =
    "0x0000000071727De22E5E9d8BAf0edAc6f37da032";

pub const ENTRYPOINT_V06_TYPE: &str = "v0.6";
pub const ENTRYPOINT_V07_TYPE: &str = "v0.7";

sol! (
    struct PackedUserOperation {
        address sender;
        uint256 nonce;
        bytes initCode;
        bytes callData;
        bytes32 accountGasLimits;
        uint256 preVerificationGas;
        bytes32 gasFees;
        bytes paymasterAndData;
        bytes signature;
    }
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    EntryPoint,
    ".foundry/forge/out/EntryPoint.sol/EntryPoint.json"
);

pub mod get_sender_address;

pub struct EntryPointConfig {
    pub chain_id: ChainId,
    pub version: EntryPointVersion,
}

impl EntryPointConfig {
    pub const V07_MAINNET: EntryPointConfig = EntryPointConfig {
        chain_id: ChainId::ETHEREUM_MAINNET,
        version: EntryPointVersion::V07,
    };

    pub const V07_SEPOLIA: EntryPointConfig = EntryPointConfig {
        chain_id: ChainId::ETHEREUM_SEPOLIA,
        version: EntryPointVersion::V07,
    };

    pub const V07_LOCAL_FOUNDRY_SEPOLIA: EntryPointConfig = EntryPointConfig {
        chain_id: ChainId::LOCAL_FOUNDRY_ETHEREUM_SEPOLIA,
        version: EntryPointVersion::V07,
    };

    pub fn address(&self) -> EntryPointAddress {
        match self.chain_id {
            ChainId::ETHEREUM_MAINNET
            | ChainId::ETHEREUM_SEPOLIA
            | ChainId::LOCAL_FOUNDRY_ETHEREUM_SEPOLIA => match self.version {
                EntryPointVersion::V06 => EntryPointAddress::new(
                    ENTRYPOINT_ADDRESS_V06.parse().unwrap(),
                ),
                EntryPointVersion::V07 => EntryPointAddress::new(
                    ENTRYPOINT_ADDRESS_V07.parse().unwrap(),
                ),
            },
            _ => panic!("Unsupported chain ID"),
        }
    }

    pub fn type_string(&self) -> String {
        self.version.type_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntryPointVersion {
    V06,
    V07,
}

impl EntryPointVersion {
    pub fn type_string(&self) -> String {
        match self {
            EntryPointVersion::V06 => ENTRYPOINT_V06_TYPE.to_string(),
            EntryPointVersion::V07 => ENTRYPOINT_V07_TYPE.to_string(),
        }
    }

    pub fn is_v06(&self) -> bool {
        self == &EntryPointVersion::V06
    }

    pub fn is_v07(&self) -> bool {
        self == &EntryPointVersion::V07
    }
}

impl From<EntryPointVersion> for String {
    fn from(value: EntryPointVersion) -> Self {
        value.type_string()
    }
}

impl From<String> for EntryPointVersion {
    fn from(value: String) -> Self {
        match value.as_str() {
            ENTRYPOINT_V06_TYPE => EntryPointVersion::V06,
            ENTRYPOINT_V07_TYPE => EntryPointVersion::V07,
            _ => panic!("invalid version string"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use eyre;

    #[test]
    fn test_address_type() -> eyre::Result<()> {
        {
            let expected_v06_address: Address =
                ENTRYPOINT_ADDRESS_V06.parse().unwrap();
            let v06 = EntryPointVersion::V06;
            let mainnet_config = EntryPointConfig {
                chain_id: ChainId::ETHEREUM_MAINNET,
                version: v06,
            };
            let mainnet_v06_address = mainnet_config.address();
            eyre::ensure!(
                mainnet_v06_address.to_address() == expected_v06_address,
                format!("unexpected address: {:?}", mainnet_v06_address)
            );
        };

        {
            let expected_v07_address: Address =
                ENTRYPOINT_ADDRESS_V07.parse().unwrap();
            let v07 = EntryPointVersion::V07;
            let mainnet_config = EntryPointConfig {
                chain_id: ChainId::ETHEREUM_MAINNET,
                version: v07,
            };
            let mainnet_v07_address = mainnet_config.address();
            eyre::ensure!(
                mainnet_v07_address.to_address() == expected_v07_address,
                format!("unexpected address: {:?}", mainnet_v07_address)
            );
        };

        {
            let expected_v07_address: Address =
                ENTRYPOINT_ADDRESS_V07.parse().unwrap();
            let v07 = EntryPointVersion::V07;
            let local_sepolia_config = EntryPointConfig {
                chain_id: ChainId::LOCAL_FOUNDRY_ETHEREUM_SEPOLIA,
                version: v07,
            };
            let local_sepolia_v07_address = local_sepolia_config.address();
            eyre::ensure!(
                local_sepolia_v07_address.to_address() == expected_v07_address,
                format!("unexpected address: {:?}", local_sepolia_v07_address)
            );
        };

        {
            let v07_type = ENTRYPOINT_V07_TYPE.to_string();
            let v07 = EntryPointVersion::from(v07_type);
            eyre::ensure!(v07.is_v07());
            eyre::ensure!(
                v07 == EntryPointVersion::V07,
                format!("unexpected type: {:?}", v07)
            );
        };

        {
            let v06_type = ENTRYPOINT_V06_TYPE.to_string();
            let v06 = EntryPointVersion::from(v06_type);
            eyre::ensure!(v06.is_v06());
            eyre::ensure!(
                v06 == EntryPointVersion::V06,
                format!("unexpected type: {:?}", v06)
            );
        };

        Ok(())
    }
}
