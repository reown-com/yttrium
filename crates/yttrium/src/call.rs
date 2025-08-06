use {
    alloy::primitives::{address, Address, Bytes, U256},
    serde::{Deserialize, Serialize},
};

pub mod send;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct Call {
    pub to: Address,
    pub value: U256,
    pub input: Bytes,
}

impl Call {
    pub fn new(to: Address, value: U256, input: Bytes) -> Self {
        Self { to, value, input }
    }

    pub fn new_from_strings(
        to: String,
        value: String,
        input: String,
    ) -> eyre::Result<Self> {
        let to = to.parse()?;
        let value = value.parse()?;
        let input = input.parse()?;
        Ok(Self { to, value, input })
    }
}

impl std::fmt::Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Transaction(to: {}, value: {}, input: {})",
            self.to, self.value, self.input
        )
    }
}

impl Call {
    pub fn mock() -> Self {
        Self {
            to: address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"),
            value: U256::ZERO,
            input: "0x68656c6c6f".parse().unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_from_strings() -> eyre::Result<()> {
        let expected_transaction = Call::mock();

        let transaction = Call::new_from_strings(
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
            "0".to_string(),
            "0x68656c6c6f".to_string(),
        )?;

        println!("transaction: {transaction:?}");

        eyre::ensure!(
            transaction == expected_transaction,
            "transaction {} should be equal to expected transaction {}",
            transaction,
            expected_transaction
        );

        Ok(())
    }
}
