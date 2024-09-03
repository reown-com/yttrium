pub mod send;

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub to: String,
    pub value: String,
    pub data: String,
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
            to: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
            value: "0".to_string(),
            data: "0x68656c6c6f".to_string(),
        }
    }
}
