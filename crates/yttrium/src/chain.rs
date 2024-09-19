use crate::entry_point::{EntryPointConfig, EntryPointVersion};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChainId(u64);

impl ChainId {
    pub const ETHEREUM_MAINNET: Self = ChainId::new_eip155(1);

    pub const ETHEREUM_SEPOLIA: Self = Self::new_eip155(11155111);

    pub const LOCAL_FOUNDRY_ETHEREUM_SEPOLIA: Self = Self::new_eip155(31337);

    pub const fn new_eip155(id: u64) -> Self {
        Self(id)
    }

    pub fn new_caip2(caip2_identifier: &str) -> eyre::Result<Self> {
        let components = caip2_identifier.split(':').collect::<Vec<_>>();
        let prefix = components
            .first()
            .map(ToOwned::to_owned)
            .ok_or_else(|| eyre::eyre!("Invalid CAIP2 chain identifier"))?;
        let chain_id = components
            .get(1)
            .map(ToOwned::to_owned)
            .ok_or_else(|| eyre::eyre!("Invalid CAIP2 chain identifier"))?;
        match prefix {
            "eip155" => {
                let id: u64 = chain_id.parse()?;
                Ok(Self(id))
            }
            _ => Err(eyre::eyre!("Invalid EIP155 chain ID")),
        }
    }

    pub fn caip2_identifier(&self) -> String {
        format!("eip155:{}", self.0)
    }

    pub fn eip155_chain_id(&self) -> u64 {
        self.0
    }
}

impl From<u64> for ChainId {
    fn from(id: u64) -> Self {
        Self::new_eip155(id)
    }
}

impl From<ChainId> for u64 {
    fn from(id: ChainId) -> Self {
        id.0
    }
}

impl From<ChainId> for String {
    fn from(chain_id: ChainId) -> Self {
        chain_id.caip2_identifier()
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Chain {
    pub id: ChainId,
    pub entry_point_version: EntryPointVersion,
    pub name: &'static str,
}

impl Chain {
    pub fn new(
        id: ChainId,
        entry_point_version: EntryPointVersion,
        name: &'static str,
    ) -> Self {
        Self { id, entry_point_version, name }
    }

    pub const ETHEREUM_MAINNET_V07: Self = Self {
        id: ChainId::ETHEREUM_MAINNET,
        entry_point_version: EntryPointVersion::V07,
        name: "Ethereum Mainnet",
    };

    pub const ETHEREUM_MAINNET_V06: Self = Self {
        id: ChainId::ETHEREUM_MAINNET,
        entry_point_version: EntryPointVersion::V06,
        name: "Ethereum Mainnet",
    };

    pub const ETHEREUM_SEPOLIA_V07: Self = Self {
        id: ChainId::ETHEREUM_SEPOLIA,
        entry_point_version: EntryPointVersion::V07,
        name: "Ethereum Sepolia",
    };

    pub const ETHEREUM_SEPOLIA_V06: Self = Self {
        id: ChainId::ETHEREUM_SEPOLIA,
        entry_point_version: EntryPointVersion::V06,
        name: "Ethereum Sepolia",
    };

    pub const LOCAL_ETHEREUM_SEPOLIA_V07: Self = Self {
        id: ChainId::LOCAL_FOUNDRY_ETHEREUM_SEPOLIA,
        entry_point_version: EntryPointVersion::V07,
        name: "Local Ethereum Sepolia",
    };

    pub const LOCAL_ETHEREUM_SEPOLIA_V06: Self = Self {
        id: ChainId::LOCAL_FOUNDRY_ETHEREUM_SEPOLIA,
        entry_point_version: EntryPointVersion::V06,
        name: "Local Ethereum Sepolia",
    };
}

impl Chain {
    pub fn entry_point_config(&self) -> EntryPointConfig {
        EntryPointConfig {
            chain_id: self.id.clone(),
            version: self.entry_point_version,
        }
    }

    pub fn caip2_identifier(&self) -> String {
        self.id.caip2_identifier()
    }
}

impl From<ChainId> for Chain {
    fn from(chain_id: ChainId) -> Self {
        Self {
            id: chain_id,
            entry_point_version: EntryPointVersion::V07,
            name: "",
        }
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}
