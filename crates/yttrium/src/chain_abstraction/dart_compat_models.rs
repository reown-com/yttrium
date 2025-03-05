#[cfg(feature = "chain_abstraction_client")]
use {
    super::{
        amount::Amount,
        api::{
            prepare::{
                BridgingError, FundingMetadata, InitialTransactionMetadata,
                Metadata, PrepareResponse, PrepareResponseAvailable,
                PrepareResponseError, PrepareResponseNotRequired,
                PrepareResponseSuccess,
            },
            FeeEstimatedTransaction, Transaction,
        },
        client::ExecuteDetails,
        error::{PrepareDetailedResponse, PrepareDetailedResponseSuccess},
        ui_fields::{TransactionFee, TxnDetails, UiFields},
    },
    crate::{
        call::Call,
        chain_abstraction::{
            local_fee_acc::LocalAmountAcc, pulse::PulseMetadata,
        },
    },
    alloy::{
        primitives::PrimitiveSignature, providers::utils::Eip1559Estimation,
        rpc::types::TransactionReceipt,
    },
    flutter_rust_bridge::frb,
    // hex::{decode, encode},
    std::str::FromStr,
};

#[derive(Debug, thiserror::Error)]
pub enum ErrorCompat {
    #[error("General {message}")]
    General { message: String },
}

#[cfg(feature = "chain_abstraction_client")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrimitiveSignatureCompat {
    pub y_parity: bool,
    pub r: String,
    pub s: String,
}

impl From<PrimitiveSignatureCompat> for PrimitiveSignature {
    fn from(compat: PrimitiveSignatureCompat) -> Self {
        type U256 = alloy::primitives::U256;
        PrimitiveSignature::new(
            U256::from_str(&compat.r).unwrap(),
            U256::from_str(&compat.s).unwrap(),
            compat.y_parity,
        )
    }
}

// ----------------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct PulseMetadataCompat {
    pub url: Option<String>,
    pub bundle_id: Option<String>,
    pub package_name: Option<String>,
    pub sdk_version: String,
    pub sdk_platform: String,
}

impl From<PulseMetadataCompat> for PulseMetadata {
    fn from(compat: PulseMetadataCompat) -> Self {
        let url = compat.url.and_then(|s| url::Url::parse(&s).ok());
        let bundle_id = compat.bundle_id.clone();
        // let package_name = compat.package_name.clone();
        let sdk_version = compat.sdk_version.clone();
        let sdk_platform = compat.sdk_platform.clone();

        Self { url, bundle_id, sdk_version, sdk_platform }
    }
}

// -----------------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct CallCompat {
    pub to: String,    // Convert Address to String
    pub value: u128,   // Convert U256 to String
    pub input: String, // Changed from Vec<u8> to String
}

// Convert `Call` → `CallCompat`
impl From<Call> for CallCompat {
    fn from(original: Call) -> Self {
        CallCompat {
            to: original.to.to_string(),
            value: original.value.try_into().unwrap(),
            // Convert Bytes to hex string (adding 0x prefix is optional)
            input: hex::encode(original.input.0), // or format!("0x{}", hex::encode(original.input.0))
        }
    }
}

// Convert `CallCompat` → `Call`
impl From<CallCompat> for Call {
    fn from(compat: CallCompat) -> Self {
        type Address = alloy::primitives::Address;
        type U256 = alloy::primitives::U256;
        type Bytes = alloy::primitives::Bytes;

        let to = Address::from_str(&compat.to).unwrap();
        let value = U256::from(compat.value);
        // Convert hex string back to Bytes
        let input = Bytes::from(hex::decode(&compat.input[2..]).unwrap()); // Skip "0x" prefix

        Call { to, value, input }
    }
}

// -----------------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct Eip1559EstimationCompat {
    /// The base fee per gas as a String.
    pub max_fee_per_gas: String,
    /// The max priority fee per gas as a String.
    pub max_priority_fee_per_gas: String,
}

impl From<Eip1559Estimation> for Eip1559EstimationCompat {
    fn from(original: Eip1559Estimation) -> Self {
        Self {
            max_fee_per_gas: original.max_fee_per_gas.to_string(),
            max_priority_fee_per_gas: original
                .max_priority_fee_per_gas
                .to_string(),
        }
    }
}

impl From<Eip1559EstimationCompat> for Eip1559Estimation {
    fn from(compat: Eip1559EstimationCompat) -> Self {
        Self {
            max_fee_per_gas: u128::from_str(&compat.max_fee_per_gas).unwrap(),
            max_priority_fee_per_gas: u128::from_str(
                &compat.max_priority_fee_per_gas,
            )
            .unwrap(),
        }
    }
}

// -----------------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct FeeCompat {
    pub fungible_amount: String,
    pub fungible_decimals: u8,
    pub fungible_price: String,
    pub fungible_price_decimals: u8,
}

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct LocalAmountAccCompat {
    pub fees: Vec<FeeCompat>,
}

impl From<LocalAmountAcc> for LocalAmountAccCompat {
    fn from(original: LocalAmountAcc) -> Self {
        Self { fees: original.get_fees_compat() }
    }
}

// impl From<LocalAmountAccCompat> for LocalAmountAcc {
//     fn from(compat: LocalAmountAccCompat) -> Self {
//         Self { fees: compat.get_fees_compat() }
//     }
// }

// -----------------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct UiFieldsCompat {
    pub route_response: PrepareResponseAvailableCompat,
    pub route: Vec<TxnDetailsCompat>,
    pub local_route_total: AmountCompat,
    pub bridge: Vec<TransactionFeeCompat>,
    pub local_bridge_total: AmountCompat,
    pub initial: TxnDetailsCompat,
    pub local_total: AmountCompat,
}

impl From<UiFields> for UiFieldsCompat {
    fn from(original: UiFields) -> Self {
        Self {
            route_response: PrepareResponseAvailableCompat::from(
                original.route_response,
            ),
            route: original
                .route
                .into_iter()
                .map(TxnDetailsCompat::from)
                .collect(),
            local_route_total: AmountCompat::from(original.local_route_total),
            bridge: original
                .bridge
                .into_iter()
                .map(TransactionFeeCompat::from)
                .collect(),
            local_bridge_total: AmountCompat::from(original.local_bridge_total),
            initial: TxnDetailsCompat::from(original.initial),
            local_total: AmountCompat::from(original.local_total),
        }
    }
}

impl From<UiFieldsCompat> for UiFields {
    fn from(compat: UiFieldsCompat) -> Self {
        Self {
            route_response: PrepareResponseAvailable::from(
                compat.route_response,
            ),
            route: compat.route.into_iter().map(TxnDetails::from).collect(),
            local_route_total: Amount::from(compat.local_route_total),
            bridge: compat
                .bridge
                .into_iter()
                .map(TransactionFee::from)
                .collect(),
            local_bridge_total: Amount::from(compat.local_bridge_total),
            initial: TxnDetails::from(compat.initial),
            local_total: Amount::from(compat.local_total),
        }
    }
}

// ------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct PrepareResponseAvailableCompat {
    pub orchestration_id: String,
    pub initial_transaction: TransactionCompat,
    pub transactions: Vec<TransactionCompat>,
    pub metadata: MetadataCompat,
}

impl From<PrepareResponseAvailable> for PrepareResponseAvailableCompat {
    fn from(original: PrepareResponseAvailable) -> Self {
        Self {
            orchestration_id: original.orchestration_id,
            initial_transaction: original.initial_transaction.into(),
            transactions: original
                .transactions
                .into_iter()
                .map(TransactionCompat::from)
                .collect(),
            metadata: original.metadata.into(),
        }
    }
}

impl From<PrepareResponseAvailableCompat> for PrepareResponseAvailable {
    fn from(compat: PrepareResponseAvailableCompat) -> Self {
        Self {
            orchestration_id: compat.orchestration_id,
            initial_transaction: compat.initial_transaction.into(),
            transactions: compat
                .transactions
                .into_iter()
                .map(Transaction::from)
                .collect(),
            metadata: compat.metadata.into(),
        }
    }
}

// ------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct MetadataCompat {
    pub funding_from: Vec<FundingMetadataCompat>,
    pub initial_transaction: InitialTransactionMetadataCompat,
    pub check_in: u64,
}

impl From<Metadata> for MetadataCompat {
    fn from(original: Metadata) -> Self {
        Self {
            funding_from: original
                .funding_from
                .into_iter()
                .map(FundingMetadataCompat::from)
                .collect(),
            initial_transaction: original.initial_transaction.into(),
            check_in: original.check_in,
        }
    }
}

impl From<MetadataCompat> for Metadata {
    fn from(compat: MetadataCompat) -> Self {
        Self {
            funding_from: compat
                .funding_from
                .into_iter()
                .map(FundingMetadata::from)
                .collect(),
            initial_transaction: compat.initial_transaction.into(),
            check_in: compat.check_in,
        }
    }
}

// ------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct FundingMetadataCompat {
    pub chain_id: String,
    pub token_contract: String,
    pub symbol: String,
    pub amount: String,
    pub bridging_fee: String,
    pub decimals: u8,
}

impl From<FundingMetadata> for FundingMetadataCompat {
    fn from(original: FundingMetadata) -> Self {
        Self {
            chain_id: original.chain_id,
            token_contract: format!("{:?}", original.token_contract),
            symbol: original.symbol,
            amount: original.amount.to_string(),
            bridging_fee: original.bridging_fee.to_string(),
            decimals: original.decimals,
        }
    }
}

impl From<FundingMetadataCompat> for FundingMetadata {
    fn from(compat: FundingMetadataCompat) -> Self {
        type Address = alloy::primitives::Address;
        type U256 = alloy::primitives::U256;
        Self {
            chain_id: compat.chain_id,
            token_contract: Address::from_str(&compat.token_contract).unwrap(),
            symbol: compat.symbol,
            amount: U256::from_str(&compat.amount).unwrap(),
            bridging_fee: U256::from_str(&compat.bridging_fee).unwrap(),
            decimals: compat.decimals,
        }
    }
}

// ------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct InitialTransactionMetadataCompat {
    pub transfer_to: String,
    pub amount: String,
    pub token_contract: String,
    pub symbol: String,
    pub decimals: u8,
}

impl From<InitialTransactionMetadataCompat> for InitialTransactionMetadata {
    fn from(compat: InitialTransactionMetadataCompat) -> Self {
        type Address = alloy::primitives::Address;
        type U256 = alloy::primitives::U256;
        Self {
            transfer_to: Address::from_str(&compat.transfer_to).unwrap(),
            amount: U256::from_str(&compat.amount).unwrap(),
            token_contract: Address::from_str(&compat.token_contract).unwrap(),
            symbol: compat.symbol,
            decimals: compat.decimals,
        }
    }
}

impl From<InitialTransactionMetadata> for InitialTransactionMetadataCompat {
    fn from(original: InitialTransactionMetadata) -> Self {
        Self {
            transfer_to: format!("{:?}", original.transfer_to),
            amount: original.amount.to_string(),
            token_contract: format!("{:?}", original.token_contract),
            symbol: original.symbol,
            decimals: original.decimals,
        }
    }
}

// ------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct TxnDetailsCompat {
    pub transaction: FeeEstimatedTransactionCompat,
    pub transaction_hash_to_sign: String,
    pub fee: TransactionFeeCompat,
}

impl From<TxnDetails> for TxnDetailsCompat {
    fn from(original: TxnDetails) -> Self {
        Self {
            transaction: original.transaction.into(),
            transaction_hash_to_sign: format!(
                "{:?}",
                original.transaction_hash_to_sign
            ),
            fee: original.fee.into(),
        }
    }
}

impl From<TxnDetailsCompat> for TxnDetails {
    fn from(compat: TxnDetailsCompat) -> Self {
        type B256 = alloy::primitives::B256;
        Self {
            transaction: compat.transaction.into(),
            transaction_hash_to_sign: B256::from_str(
                &compat.transaction_hash_to_sign,
            )
            .unwrap(),
            fee: compat.fee.into(),
        }
    }
}

// -------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct FeeEstimatedTransactionCompat {
    pub chain_id: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub input: String, // Changed from Vec<u8> to String
    pub gas_limit: String,
    pub nonce: String,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
}

impl From<FeeEstimatedTransaction> for FeeEstimatedTransactionCompat {
    fn from(original: FeeEstimatedTransaction) -> Self {
        Self {
            chain_id: original.chain_id,
            from: format!("{:?}", original.from),
            to: format!("{:?}", original.to),
            value: original.value.to_string(),
            // Convert Bytes to hex string
            input: hex::encode(original.input), // or format!("0x{}", hex::encode(original.input)) if you prefer 0x prefix
            gas_limit: original.gas_limit.to_string(),
            nonce: original.nonce.to_string(),
            max_fee_per_gas: original.max_fee_per_gas.to_string(),
            max_priority_fee_per_gas: original
                .max_priority_fee_per_gas
                .to_string(),
        }
    }
}

impl From<FeeEstimatedTransactionCompat> for FeeEstimatedTransaction {
    fn from(compat: FeeEstimatedTransactionCompat) -> Self {
        type Address = alloy::primitives::Address;
        type U256 = alloy::primitives::U256;
        type U64 = alloy::primitives::U64;
        type U128 = alloy::primitives::U128;
        type Bytes = alloy::primitives::Bytes;
        Self {
            chain_id: compat.chain_id,
            from: Address::from_str(&compat.from).unwrap(),
            to: Address::from_str(&compat.to).unwrap(),
            value: U256::from_str(&compat.value).unwrap(),
            // Convert hex string back to Bytes
            input: Bytes::from(hex::decode(&compat.input[2..]).unwrap()), // Skip "0x" prefix
            gas_limit: U64::from_str(&compat.gas_limit).unwrap(),
            nonce: U64::from_str(&compat.nonce).unwrap(),
            max_fee_per_gas: U128::from_str(&compat.max_fee_per_gas).unwrap(),
            max_priority_fee_per_gas: U128::from_str(
                &compat.max_priority_fee_per_gas,
            )
            .unwrap(),
        }
    }
}

// ------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct TransactionFeeCompat {
    pub fee: AmountCompat,
    pub local_fee: AmountCompat,
}

impl From<TransactionFee> for TransactionFeeCompat {
    fn from(original: TransactionFee) -> Self {
        Self { fee: original.fee.into(), local_fee: original.local_fee.into() }
    }
}

impl From<TransactionFeeCompat> for TransactionFee {
    fn from(compat: TransactionFeeCompat) -> Self {
        Self { fee: compat.fee.into(), local_fee: compat.local_fee.into() }
    }
}

// ------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct AmountCompat {
    pub symbol: String,
    pub amount: String,
    pub unit: u8,
    pub formatted: String,
    pub formatted_alt: String,
}

impl From<Amount> for AmountCompat {
    fn from(orginal: Amount) -> Self {
        Self {
            symbol: orginal.symbol,
            amount: orginal.amount.to_string(),
            unit: orginal.unit,
            formatted: orginal.formatted,
            formatted_alt: orginal.formatted_alt,
        }
    }
}

impl From<AmountCompat> for Amount {
    fn from(compat: AmountCompat) -> Self {
        type U256 = alloy::primitives::U256;
        Self {
            symbol: compat.symbol,
            amount: U256::from_str(&compat.amount).unwrap(),
            unit: compat.unit,
            formatted: compat.formatted,
            formatted_alt: compat.formatted_alt,
        }
    }
}

// -----

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct ExecuteDetailsCompat {
    pub initial_txn_receipt: TransactionReceiptCompat,
    pub initial_txn_hash: String,
}

impl From<ExecuteDetails> for ExecuteDetailsCompat {
    fn from(original: ExecuteDetails) -> Self {
        Self {
            initial_txn_receipt: original.initial_txn_receipt.into(),
            initial_txn_hash: original.initial_txn_hash.to_string(),
        }
    }
}

impl From<ExecuteDetailsCompat> for ExecuteDetails {
    fn from(compat: ExecuteDetailsCompat) -> Self {
        type B256 = alloy::primitives::B256;
        Self {
            initial_txn_receipt: compat.initial_txn_receipt.into(),
            initial_txn_hash: B256::from_str(&compat.initial_txn_hash).unwrap(),
        }
    }
}

// -------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct TransactionReceiptCompat {
    pub transaction_hash: String,
    pub transaction_index: Option<u64>,
    pub block_hash: Option<String>,
    pub block_number: Option<u64>,
    pub gas_used: u64,
    pub effective_gas_price: String,
    pub blob_gas_used: Option<u64>,
    pub blob_gas_price: Option<String>,
    pub from: String,
    pub to: Option<String>,
    pub contract_address: Option<String>,
}

impl From<alloy::rpc::types::TransactionReceipt> for TransactionReceiptCompat {
    fn from(original: alloy::rpc::types::TransactionReceipt) -> Self {
        Self {
            transaction_hash: format!("{:?}", original.transaction_hash),
            transaction_index: original.transaction_index,
            block_hash: original.block_hash.map(|h| format!("{:?}", h)),
            block_number: original.block_number,
            gas_used: original.gas_used,
            effective_gas_price: original.effective_gas_price.to_string(),
            blob_gas_used: original.blob_gas_used,
            blob_gas_price: original.blob_gas_price.map(|g| g.to_string()),
            from: format!("{:?}", original.from),
            to: original.to.map(|t| format!("{:?}", t)),
            contract_address: original
                .contract_address
                .map(|c| format!("{:?}", c)),
        }
    }
}

impl From<TransactionReceiptCompat> for TransactionReceipt {
    fn from(compat: TransactionReceiptCompat) -> Self {
        type Address = alloy::primitives::Address;
        type TxHash = alloy::primitives::B256;
        type BlockHash = alloy::primitives::B256;
        type Bloom = alloy::primitives::Bloom;

        let dummy_receipt = alloy::rpc::types::Receipt {
            status: alloy::consensus::Eip658Value::Eip658(true),
            cumulative_gas_used: 0,
            logs: Vec::new(),
        };
        let dummy_receipt_with_bloom = alloy::rpc::types::ReceiptWithBloom::new(
            dummy_receipt,
            Bloom::ZERO,
        );

        Self {
            // FIXME
            inner: alloy::rpc::types::ReceiptEnvelope::Legacy(
                dummy_receipt_with_bloom,
            ),
            transaction_hash: TxHash::from_str(&compat.transaction_hash)
                .unwrap(),
            transaction_index: compat.transaction_index,
            block_hash: compat
                .block_hash
                .map(|h| BlockHash::from_str(&h).unwrap()),
            block_number: compat.block_number,
            gas_used: compat.gas_used,
            effective_gas_price: compat
                .effective_gas_price
                .parse::<u128>()
                .unwrap(),
            blob_gas_used: compat.blob_gas_used,
            blob_gas_price: compat
                .blob_gas_price
                .map(|g| g.parse::<u128>().unwrap()),
            from: Address::from_str(&compat.from).unwrap(),
            to: compat.to.map(|t| Address::from_str(&t).unwrap()),
            contract_address: compat
                .contract_address
                .map(|c| Address::from_str(&c).unwrap()),
        }
    }
}

// -------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"))]
pub enum PrepareDetailedResponseCompat {
    Success { value: PrepareDetailedResponseSuccessCompat },
    Error { value: PrepareResponseError },
}

impl From<PrepareDetailedResponse> for PrepareDetailedResponseCompat {
    fn from(original: PrepareDetailedResponse) -> Self {
        match original {
            PrepareDetailedResponse::Success(success) => {
                PrepareDetailedResponseCompat::Success { value: success.into() }
            }
            PrepareDetailedResponse::Error(error) => {
                PrepareDetailedResponseCompat::Error { value: error }
            }
        }
    }
}

impl From<PrepareDetailedResponseCompat> for PrepareDetailedResponse {
    fn from(compat: PrepareDetailedResponseCompat) -> Self {
        match compat {
            PrepareDetailedResponseCompat::Success { value } => {
                PrepareDetailedResponse::Success(value.into())
            }
            PrepareDetailedResponseCompat::Error { value } => {
                PrepareDetailedResponse::Error(value)
            }
        }
    }
}

// ------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub enum PrepareDetailedResponseSuccessCompat {
    Available { value: UiFieldsCompat },
    NotRequired { value: PrepareResponseNotRequiredCompat },
}

// impl PrepareDetailedResponseSuccessCompat {
//   pub fn into_option(self) -> Option<UiFieldsCompat> {
//       match self {
//           Self::Available(a) => Some(a),
//           Self::NotRequired(_) => None,
//       }
//   }
// }

// impl PrepareDetailedResponse {
//   pub fn into_result(
//       self,
//   ) -> Result<PrepareDetailedResponseSuccess, PrepareResponseError> {
//       match self {
//           Self::Success(success) => Ok(success),
//           Self::Error(error) => Err(error),
//       }
//   }
// }

impl From<PrepareDetailedResponseSuccess>
    for PrepareDetailedResponseSuccessCompat
{
    fn from(original: PrepareDetailedResponseSuccess) -> Self {
        match original {
            PrepareDetailedResponseSuccess::Available(ui_fields) => {
                PrepareDetailedResponseSuccessCompat::Available {
                    value: ui_fields.into(),
                }
            }
            PrepareDetailedResponseSuccess::NotRequired(not_required) => {
                PrepareDetailedResponseSuccessCompat::NotRequired {
                    value: not_required.into(),
                }
            }
        }
    }
}

impl From<PrepareDetailedResponseSuccessCompat>
    for PrepareDetailedResponseSuccess
{
    fn from(compat: PrepareDetailedResponseSuccessCompat) -> Self {
        match compat {
            PrepareDetailedResponseSuccessCompat::Available { value } => {
                PrepareDetailedResponseSuccess::Available(value.into())
            }
            PrepareDetailedResponseSuccessCompat::NotRequired { value } => {
                PrepareDetailedResponseSuccess::NotRequired(value.into())
            }
        }
    }
}

// ------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"))]
pub struct PrepareResponseErrorCompat {
    pub error: BridgingErrorCompat,
    pub reason: String,
}

impl From<PrepareResponseError> for PrepareResponseErrorCompat {
    fn from(original: PrepareResponseError) -> Self {
        Self { error: original.error.into(), reason: original.reason }
    }
}

impl From<PrepareResponseErrorCompat> for PrepareResponseError {
    fn from(compat: PrepareResponseErrorCompat) -> Self {
        Self { error: compat.error.into(), reason: compat.reason }
    }
}

// ------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub enum BridgingErrorCompat {
    NoRoutesAvailable,
    InsufficientFunds,
    InsufficientGasFunds,
}

impl From<BridgingError> for BridgingErrorCompat {
    fn from(original: BridgingError) -> Self {
        match original {
            BridgingError::NoRoutesAvailable => {
                BridgingErrorCompat::NoRoutesAvailable
            }
            BridgingError::InsufficientFunds => {
                BridgingErrorCompat::InsufficientFunds
            }
            BridgingError::InsufficientGasFunds => {
                BridgingErrorCompat::InsufficientGasFunds
            }
        }
    }
}

impl From<BridgingErrorCompat> for BridgingError {
    fn from(compat: BridgingErrorCompat) -> Self {
        match compat {
            BridgingErrorCompat::NoRoutesAvailable => {
                BridgingError::NoRoutesAvailable
            }
            BridgingErrorCompat::InsufficientFunds => {
                BridgingError::InsufficientFunds
            }
            BridgingErrorCompat::InsufficientGasFunds => {
                BridgingError::InsufficientGasFunds
            }
        }
    }
}

// ------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct PrepareResponseNotRequiredCompat {
    pub initial_transaction: TransactionCompat,
    pub transactions: Vec<TransactionCompat>,
}

// from PrepareResponseNotRequired to PrepareResponseNotRequiredCompat
impl From<PrepareResponseNotRequired> for PrepareResponseNotRequiredCompat {
    fn from(original: PrepareResponseNotRequired) -> Self {
        Self {
            initial_transaction: original.initial_transaction.into(),
            transactions: original
                .transactions
                .into_iter()
                .map(TransactionCompat::from)
                .collect(),
        }
    }
}

// from PrepareResponseNotRequiredCompat to PrepareResponseNotRequired
impl From<PrepareResponseNotRequiredCompat> for PrepareResponseNotRequired {
    fn from(compat: PrepareResponseNotRequiredCompat) -> Self {
        Self {
            initial_transaction: compat.initial_transaction.into(),
            transactions: compat
                .transactions
                .into_iter()
                .map(Transaction::from)
                .collect(),
        }
    }
}

// ------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub struct TransactionCompat {
    // CAIP-2 chain ID
    pub chain_id: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub input: String, // Changed from Vec<u8> to String
    pub gas_limit: u64,
    pub nonce: u64,
}

// from Transaction to TransactionCompat
impl From<Transaction> for TransactionCompat {
    fn from(original: Transaction) -> Self {
        TransactionCompat {
            chain_id: original.chain_id,
            from: original.from.to_string(),
            to: original.to.to_string(),
            value: original.value.to_string(),
            // Convert Bytes to hex string
            input: hex::encode(original.input.0), // or format!("0x{}", hex::encode(original.input.0)) for 0x prefix
            gas_limit: original.gas_limit.to(),
            nonce: original.nonce.to(),
        }
    }
}

// from TransactionCompat to Transaction
impl From<TransactionCompat> for Transaction {
    fn from(compat: TransactionCompat) -> Transaction {
        type Address = alloy::primitives::Address;
        type U256 = alloy::primitives::U256;
        type U64 = alloy::primitives::U64;
        type Bytes = alloy::primitives::Bytes;

        let chain_id = compat.chain_id;
        let from = Address::from_str(&compat.from).unwrap();
        let to = Address::from_str(&compat.to).unwrap();
        let value = U256::from_str(&compat.value).unwrap();
        // Convert hex string back to Bytes
        let input = Bytes::from(hex::decode(&compat.input[2..]).unwrap()); // Skip "0x" prefix
        let gas_limit = U64::from(compat.gas_limit);
        let nonce = U64::from(compat.nonce);

        Transaction { chain_id, from, to, value, input, gas_limit, nonce }
    }
}

// impl TransactionCompat {
//     pub fn to_json(&self) -> String {
//         serde_json::to_string(self).unwrap()
//     }

//     pub fn from_json(json: &str) -> Self {
//         serde_json::from_str(json).unwrap()
//     }
// }

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"))]
pub enum PrepareResponseCompat {
    Success { value: PrepareResponseSuccessCompat },
    Error { value: PrepareResponseErrorCompat },
}

impl From<PrepareResponse> for PrepareResponseCompat {
    fn from(original: PrepareResponse) -> Self {
        match original {
            PrepareResponse::Success(success) => {
                PrepareResponseCompat::Success { value: success.into() }
            }
            PrepareResponse::Error(error) => {
                PrepareResponseCompat::Error { value: error.into() }
            }
        }
    }
}

impl From<PrepareResponseCompat> for PrepareResponse {
    fn from(compat: PrepareResponseCompat) -> Self {
        match compat {
            PrepareResponseCompat::Success { value } => {
                PrepareResponse::Success(value.into())
            }
            PrepareResponseCompat::Error { value } => {
                PrepareResponse::Error(value.into())
            }
        }
    }
}

// ------

#[cfg(feature = "chain_abstraction_client")]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[frb(dart_metadata=("freezed"), json_serializable)]
pub enum PrepareResponseSuccessCompat {
    Available { value: PrepareResponseAvailableCompat },
    NotRequired { value: PrepareResponseNotRequiredCompat },
}

impl From<PrepareResponseSuccess> for PrepareResponseSuccessCompat {
    fn from(original: PrepareResponseSuccess) -> Self {
        match original {
            PrepareResponseSuccess::Available(available) => {
                PrepareResponseSuccessCompat::Available {
                    value: available.into(),
                }
            }
            PrepareResponseSuccess::NotRequired(not_required) => {
                PrepareResponseSuccessCompat::NotRequired {
                    value: not_required.into(),
                }
            }
        }
    }
}

impl From<PrepareResponseSuccessCompat> for PrepareResponseSuccess {
    fn from(compat: PrepareResponseSuccessCompat) -> Self {
        match compat {
            PrepareResponseSuccessCompat::Available { value } => {
                PrepareResponseSuccess::Available(value.into())
            }
            PrepareResponseSuccessCompat::NotRequired { value } => {
                PrepareResponseSuccess::NotRequired(value.into())
            }
        }
    }
}

impl PrepareResponseSuccessCompat {
    pub fn into_option(self) -> Option<PrepareResponseAvailableCompat> {
        match self {
            Self::Available { value } => Some(value),
            Self::NotRequired { .. } => None,
        }
    }
}
