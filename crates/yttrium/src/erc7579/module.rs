use alloy::primitives::{Address, Bytes};

pub struct Module {
    pub address: Address,
    pub module: Address,
    pub init_data: Bytes,
    pub de_init_data: Bytes,
    pub additional_context: Bytes,
    pub r#type: ModuleType,
    pub hook: Option<Address>,
}

pub enum ModuleType {
    Validator,
    Executor,
    Fallback,
    Hook,
}
