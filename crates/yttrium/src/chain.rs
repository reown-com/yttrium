use crate::entry_point::{EntryPointConfig, EntryPointVersion};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChainId(&'static str);

impl ChainId {
    pub const ETHEREUM_MAINNET: Self = Self::new_const("eip155:1");

    pub const ETHEREUM_SEPOLIA: Self = Self::new_const("eip155:11155111");

    pub const LOCAL_FOUNDRY_ETHEREUM_SEPOLIA: Self =
        Self::new_const("eip155:31337");

    const fn new_const(caip2_identifier: &'static str) -> Self {
        Self(caip2_identifier)
    }

    pub fn new(caip2_identifier: &'static str) -> eyre::Result<Self> {
        let components = caip2_identifier.split(':').collect::<Vec<_>>();
        let prefix = components
            .get(0)
            .map(ToOwned::to_owned)
            .ok_or_else(|| eyre::eyre!("Invalid CAIP2 chain identifier"))?;
        let chain_id = components
            .get(1)
            .map(ToOwned::to_owned)
            .ok_or_else(|| eyre::eyre!("Invalid CAIP2 chain identifier"))?;
        match prefix {
            "eip155" => {
                let _: u64 = chain_id.parse()?;
                Ok(Self(&caip2_identifier))
            }
            _ => Err(eyre::eyre!("Invalid EIP155 chain ID")),
        }
    }

    pub fn caip2_identifier(&self) -> String {
        self.0.to_string()
    }

    pub fn eip155_chain_id(&self) -> eyre::Result<u64> {
        let components = self.0.split(':').collect::<Vec<_>>();
        let prefix = components
            .get(0)
            .map(ToOwned::to_owned)
            .ok_or_else(|| eyre::eyre!("Invalid CAIP2 chain identifier"))?;
        if prefix != "eip155" {
            return Err(eyre::eyre!("Invalid EIP155 chain ID"));
        }
        let chain_id_string = components
            .get(1)
            .map(ToOwned::to_owned)
            .ok_or_else(|| eyre::eyre!("Invalid CAIP2 chain identifier"))
            .unwrap();
        let chain_id = chain_id_string.parse()?;
        Ok(chain_id)
    }
}

impl Into<String> for ChainId {
    fn into(self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Chain {
    pub id: ChainId,
    pub entry_point_version: EntryPointVersion,
    pub name: &'static str,
}

impl Chain {
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
            chain_id: self.id,
            version: self.entry_point_version,
        }
    }

    pub fn caip2_identifier(&self) -> String {
        self.id.caip2_identifier()
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}
