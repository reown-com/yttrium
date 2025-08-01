mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
use {
    alloy::{
        primitives::Address,
        providers::{Provider, ProviderBuilder},
    },
    relay_rpc::domain::ProjectId,
    std::time::Duration,
    yttrium::{
        call::Call,
        chain_abstraction::{
            api::{
                prepare::{PrepareResponse, PrepareResponseAvailable},
                status::{StatusResponse, StatusResponseCompleted},
            },
            client::Client,
            currency::Currency,
            pulse::PulseMetadata,
            ui_fields::UiFields,
        },
    },
};

// uniffi::custom_type!(FFIAddress, String, {
//     try_lift: |val| Ok(val.parse()?),
//     lower: |obj| obj.to_string(),
// });
// uniffi::custom_type!(AccountAddress, FFIAddress, {
//     try_lift: |val| Ok(val.into()),
//     lower: |obj| obj.into(),
// });

// fn uint_to_hex<const BITS: usize, const LIMBS: usize>(
//     obj: Uint<BITS, LIMBS>,
// ) -> String {
//     format!("0x{obj:x}")
// }

// uniffi::custom_type!(FFIU64, String, {
//     try_lift: |val| Ok(val.parse()?),
//     lower: |obj| uint_to_hex(obj),
// });

// uniffi::custom_type!(FFIU128, String, {
//     try_lift: |val| Ok(val.parse()?),
//     lower: |obj| uint_to_hex(obj),
// });

// uniffi::custom_type!(FFIU256, String, {
//     try_lift: |val| Ok(val.parse()?),
//     lower: |obj| uint_to_hex(obj),
// });

// uniffi::custom_type!(FFIBytes, String, {
//     try_lift: |val| Ok(val.parse()?),
//     lower: |obj| obj.to_string(),
// });

// // #[frb]
// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
// pub struct Call {
//     pub to: String,
//     pub value: String,
//     pub input: Vec<u8>,
// }

// // #[frb]
// impl From<yttrium::call::Call> for Call {
//     fn from(source: yttrium::call::Call) -> Self {
//         Self {
//             to: source.to.to_string(),
//             value: source.value.to_string(),
//             input: source.input.to_vec(),
//         }
//     }
// }

// #[frb]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Eip1559Estimation {
    /// The base fee per gas as a String.
    pub max_fee_per_gas: String,
    /// The max priority fee per gas as a String.
    pub max_priority_fee_per_gas: String,
}

// #[frb]
impl From<alloy::providers::utils::Eip1559Estimation> for Eip1559Estimation {
    fn from(source: alloy::providers::utils::Eip1559Estimation) -> Self {
        Self {
            max_fee_per_gas: source.max_fee_per_gas.to_string(),
            max_priority_fee_per_gas: source
                .max_priority_fee_per_gas
                .to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PreparedSignature {
    pub message_hash: String,
}

// #[frb]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("General {0}")]
    General(String),
}

// #[frb]
pub struct ChainAbstractionClient {
    pub project_id: String,
    client: Client,
}

// #[frb]
impl ChainAbstractionClient {
    // #[uniffi::constructor]
    pub fn new(project_id: String, pulse_metadata: PulseMetadata) -> Self {
        let client =
            Client::new(ProjectId::from(project_id.clone()), pulse_metadata);
        Self { project_id, client }
    }

    // #[frb]
    pub async fn prepare(
        &self,
        chain_id: String,
        from: Address,
        call: Call,
        accounts: Vec<String>,
        use_lifi: bool,
    ) -> Result<PrepareResponse, Error> {
        self.client
            .prepare(chain_id, from, call, accounts, use_lifi)
            .await
            .map_err(|e| Error::General(e.to_string()))
    }

    // #[frb]
    pub async fn get_ui_fields(
        &self,
        route_response: PrepareResponseAvailable,
        currency: Currency,
    ) -> Result<UiFields, Error> {
        self.client
            .get_ui_fields(route_response, currency)
            .await
            .map_err(|e| Error::General(e.to_string()))
    }

    // #[frb]
    pub async fn status(
        &self,
        orchestration_id: String,
    ) -> Result<StatusResponse, Error> {
        self.client
            .status(orchestration_id)
            .await
            .map_err(|e| Error::General(e.to_string()))
    }

    // #[frb]
    pub async fn wait_for_success_with_timeout(
        &self,
        orchestration_id: String,
        check_in: u64,
        timeout: u64,
    ) -> Result<StatusResponseCompleted, Error> {
        self.client
            .wait_for_success_with_timeout(
                orchestration_id,
                Duration::from_secs(check_in),
                Duration::from_secs(timeout),
            )
            .await
            .map_err(|e| Error::General(e.to_string()))
    }

    // #[frb]
    pub async fn estimate_fees(
        &self,
        chain_id: String,
    ) -> Result<Eip1559Estimation, Error> {
        let url = format!(
            "https://rpc.walletconnect.com/v1?chainId={chain_id}&projectId={}",
            self.project_id
        )
        .parse()
        .expect("Invalid RPC URL");
        let provider = ProviderBuilder::new().on_http(url);
        provider
            .estimate_eip1559_fees(None)
            .await
            .map(Into::into)
            .map_err(|e| Error::General(e.to_string()))
    }

    // #[frb]
    pub async fn erc20_token_balance(
        &self,
        chain_id: &str,
        token: Address,
        owner: Address,
    ) -> Result<String, Error> {
        self.client
            .erc20_token_balance(chain_id, token, owner)
            .await
            .map(|balance| balance.to_string())
            .map_err(|e| Error::General(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use {
        // super::*,
        alloy::{
            hex,
            primitives::{address, bytes},
            providers::{Provider, ProviderBuilder},
        },
    };

    #[tokio::test]
    #[ignore = "run manually"]
    async fn estimate_fees() {
        let chain_id = "eip155:42161";
        let project_id = std::env::var("REOWN_PROJECT_ID").unwrap();
        let url = format!(
            "https://rpc.walletconnect.com/v1?chainId={chain_id}&projectId={project_id}")
        .parse()
        .expect("Invalid RPC URL");
        let provider =
            ProviderBuilder::new().disable_recommended_fillers().on_http(url);

        let estimate = provider.estimate_eip1559_fees(None).await.unwrap();

        println!("estimate: {estimate:?}");
        // Simulate sending the data to Dart (convert U128 values to strings)
        let max_fee_per_gas = estimate.max_fee_per_gas.to_string();
        let max_priority_fee_per_gas =
            estimate.max_priority_fee_per_gas.to_string();

        println!("Max fee per gas: {max_fee_per_gas}, Max priority fee per gas: {max_priority_fee_per_gas}");
    }

    #[test]
    fn test_address_lower() {
        use alloy::hex;

        let addr = address!("abababababababababababababababababababab");

        // Convert address to hex string
        let addr_hex = format!("0x{}", hex::encode(addr.as_slice()));
        assert_eq!(addr_hex, "0xabababababababababababababababababababab");
    }

    #[test]
    fn test_u64_lower() {
        let num = 1234567890;

        // Convert number to hex string
        let num_hex = format!("0x{num:x}");
        assert_eq!(num_hex, "0x499602d2");
    }

    #[test]
    fn test_u128_lower() {
        let num = 1234567890;

        // Convert number to hex string
        let num_hex = format!("0x{num:x}");
        assert_eq!(num_hex, "0x499602d2");
    }

    #[test]
    fn test_u256_lower() {
        let num = 1234567890;

        // Convert U256 to hex string
        let num_hex = format!("0x{num:x}");
        assert_eq!(num_hex, "0x499602d2");
    }

    #[test]
    fn test_bytes_lower() {
        let ffi_u64 = bytes!("aabbccdd");

        // Convert byte data to hex string
        let byte_hex = format!("0x{}", hex::encode(ffi_u64));
        assert_eq!(byte_hex, "0xaabbccdd");
    }
}
