use alloy::primitives::{address, Address, Bytes, U256};
use std::str::FromStr;

pub mod send;

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub to: Address,
    pub value: U256,
    pub data: Bytes,
}

impl Transaction {
    pub fn new(to: Address, value: U256, data: Bytes) -> Self {
        Self { to, value, data }
    }

    pub fn new_from_strings(
        to: String,
        value: String,
        data: String,
    ) -> eyre::Result<Self> {
        let to = to.parse()?;
        let value = value.parse()?;
        let data = data.parse()?;
        Ok(Self { to, value, data })
    }
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Transaction(to: {}, value: {}, data: {})",
            self.to, self.value, self.data
        )
    }
}

impl Transaction {
    pub fn mock() -> Self {
        Self {
            to: address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"),
            value: U256::ZERO,
            data: "0x68656c6c6f".parse().unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_from_strings() -> eyre::Result<()> {
        let expected_transaction = Transaction::mock();

        let transaction = Transaction::new_from_strings(
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
            "0".to_string(),
            "0x68656c6c6f".to_string(),
        )?;

        println!("transaction: {:?}", transaction);

        eyre::ensure!(
            transaction == expected_transaction,
            "transaction {} should be equal to expected transaction {}",
            transaction,
            expected_transaction
        );

        Ok(())
    }
}
