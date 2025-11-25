pub mod client;

pub use client::EvmSigningClient;
use {
    crate::provider_pool::ProviderPool,
    alloy::{
        consensus::{SignableTransaction, TxEip1559, TxEnvelope},
        dyn_abi::TypedData,
        network::TransactionBuilder,
        primitives::{
            Address, Bytes, PrimitiveSignature, B256, U128, U256, U64,
        },
        rlp,
        rpc::types::TransactionRequest,
        signers::{local::PrivateKeySigner, SignerSync},
    },
    alloy_provider::{utils::Eip1559Estimation, Provider, RootProvider},
    serde::{Deserialize, Serialize},
    thiserror::Error,
};

/// Parameters required to sign and broadcast an EVM transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct SignAndSendParams {
    /// CAIP-2 chain identifier (e.g. `eip155:10`).
    pub chain_id: String,
    /// Sender address.
    pub from: Address,
    /// Optional target of the transaction. None indicates contract creation.
    pub to: Option<Address>,
    /// Value in wei to transfer with the transaction.
    #[serde(default)]
    pub value: Option<U256>,
    /// Transaction calldata.
    #[serde(default)]
    pub data: Option<Bytes>,
    /// Pre-specified gas limit.
    #[serde(default)]
    pub gas_limit: Option<U64>,
    /// Pre-specified max fee per gas (EIP-1559).
    #[serde(default)]
    pub max_fee_per_gas: Option<U128>,
    /// Pre-specified max priority fee per gas (EIP-1559).
    #[serde(default)]
    pub max_priority_fee_per_gas: Option<U128>,
    /// Pre-specified nonce.
    #[serde(default)]
    pub nonce: Option<U64>,
}

/// Result of signing and broadcasting a transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[serde(rename_all = "camelCase")]
pub struct SignAndSendResult {
    pub transaction_hash: B256,
    pub raw_transaction: Bytes,
    pub gas_limit: U64,
    pub max_fee_per_gas: U128,
    pub max_priority_fee_per_gas: U128,
    pub nonce: U64,
}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
pub enum EvmSigningError {
    #[error("unsupported chain identifier format: {0}")]
    UnsupportedChainId(String),
    #[error("failed to parse chain identifier `{chain_id}`: {message}")]
    InvalidChainId { chain_id: String, message: String },
    #[error("provider error while estimating gas: {0}")]
    GasEstimation(String),
    #[error("provider error while estimating fees: {0}")]
    FeeEstimation(String),
    #[error("provider error while fetching nonce: {0}")]
    Nonce(String),
    #[error("failed to build transaction: {0}")]
    BuildTransaction(String),
    #[error("unsupported transaction type")]
    UnsupportedTransactionType,
    #[error("signing failed: {0}")]
    Signing(String),
    #[error("provider error while broadcasting transaction: {0}")]
    Broadcast(String),
    #[error("invalid typed data: {0}")]
    InvalidTypedData(String),
}

/// Signs and broadcasts an EVM transaction using the supplied signer and provider pool.
pub async fn sign_and_send_transaction(
    provider_pool: &ProviderPool,
    params: SignAndSendParams,
    signer: &PrivateKeySigner,
) -> Result<SignAndSendResult, EvmSigningError> {
    let chain_numeric = parse_chain_id(&params.chain_id)?;
    let provider = provider_pool.get_provider(&params.chain_id).await;

    let mut request = TransactionRequest::default()
        .with_chain_id(chain_numeric)
        .with_from(params.from);

    if let Some(to) = params.to {
        request = request.with_to(to);
    }
    if let Some(value) = params.value {
        request = request.with_value(value);
    }
    if let Some(data) = params.data.clone() {
        request = request.with_input(data);
    }
    if let Some(gas_limit) = params.gas_limit {
        request = request.with_gas_limit(gas_limit.to());
    }
    if let Some(fee) = params.max_fee_per_gas {
        request = request.with_max_fee_per_gas(fee.to());
    }
    if let Some(priority) = params.max_priority_fee_per_gas {
        request = request.with_max_priority_fee_per_gas(priority.to());
    }
    if let Some(nonce) = params.nonce {
        request = request.with_nonce(nonce.to());
    }

    let (request, gas_limit, fee_estimate, nonce) =
        populate_missing_fields(provider.clone(), request, &params).await?;

    let unsigned = request
        .clone()
        .build_unsigned()
        .map_err(|err| EvmSigningError::BuildTransaction(err.to_string()))?;
    let tx = unsigned
        .eip1559()
        .cloned()
        .ok_or(EvmSigningError::UnsupportedTransactionType)?;

    let signature = sign_transaction(&tx, signer)?;
    let signed = tx.into_signed(signature);
    let envelope = TxEnvelope::Eip1559(signed.clone());

    let raw = Bytes::from(rlp::encode(envelope.clone()));

    let pending = provider
        .send_tx_envelope(envelope)
        .await
        .map_err(|err| EvmSigningError::Broadcast(err.to_string()))?;

    Ok(SignAndSendResult {
        transaction_hash: *pending.tx_hash(),
        raw_transaction: raw,
        gas_limit: U64::from(gas_limit),
        max_fee_per_gas: U128::from(fee_estimate.max_fee_per_gas),
        max_priority_fee_per_gas: U128::from(
            fee_estimate.max_priority_fee_per_gas,
        ),
        nonce: U64::from(nonce),
    })
}

pub fn sign_typed_data(
    json_data: String,
    signer: &PrivateKeySigner,
) -> Result<String, EvmSigningError> {
    let typed_data: TypedData = serde_json::from_str(&json_data)
        .map_err(|err| EvmSigningError::InvalidTypedData(err.to_string()))?;

    let hash = typed_data.eip712_signing_hash(&typed_data.domain);

    let signature = signer
        .sign_hash_sync(&hash)
        .map_err(|err| EvmSigningError::Signing(err.to_string()))?;

    Ok(signature.to_string())
}

fn parse_chain_id(chain_id: &str) -> Result<u64, EvmSigningError> {
    let Some(rest) = chain_id.strip_prefix("eip155:") else {
        return Err(EvmSigningError::UnsupportedChainId(chain_id.to_string()));
    };
    rest.parse::<u64>().map_err(|err| EvmSigningError::InvalidChainId {
        chain_id: chain_id.to_string(),
        message: err.to_string(),
    })
}

fn sign_transaction(
    tx: &TxEip1559,
    signer: &PrivateKeySigner,
) -> Result<PrimitiveSignature, EvmSigningError> {
    let hash = tx.signature_hash();
    let signature = signer
        .sign_hash_sync(&hash)
        .map_err(|err| EvmSigningError::Signing(err.to_string()))?;
    Ok(signature)
}

async fn populate_missing_fields(
    provider: RootProvider,
    mut request: TransactionRequest,
    params: &SignAndSendParams,
) -> Result<(TransactionRequest, u64, Eip1559Estimation, u64), EvmSigningError>
{
    let gas_limit = match params.gas_limit {
        Some(gas) => gas.to(),
        None => provider
            .estimate_gas(&request)
            .await
            .map_err(|err| EvmSigningError::GasEstimation(err.to_string()))?,
    };
    request = request.with_gas_limit(gas_limit);

    let (max_fee_per_gas, max_priority_fee_per_gas) =
        match (params.max_fee_per_gas, params.max_priority_fee_per_gas) {
            (Some(max_fee), Some(max_priority)) => {
                (max_fee.to(), max_priority.to())
            }
            (maybe_max_fee, maybe_priority) => {
                let estimate =
                    provider.estimate_eip1559_fees(None).await.map_err(
                        |err| EvmSigningError::FeeEstimation(err.to_string()),
                    )?;

                let max_fee = maybe_max_fee
                    .map(|fee| fee.to())
                    .unwrap_or(estimate.max_fee_per_gas);
                let max_priority = maybe_priority
                    .map(|fee| fee.to())
                    .unwrap_or(estimate.max_priority_fee_per_gas);

                (max_fee, max_priority)
            }
        };

    request = request
        .with_max_fee_per_gas(max_fee_per_gas)
        .with_max_priority_fee_per_gas(max_priority_fee_per_gas);

    let nonce = match params.nonce {
        Some(nonce) => nonce.to(),
        None => provider
            .get_transaction_count(params.from)
            .await
            .map_err(|err| EvmSigningError::Nonce(err.to_string()))?,
    };

    Ok((
        request.with_nonce(nonce),
        gas_limit,
        Eip1559Estimation { max_fee_per_gas, max_priority_fee_per_gas },
        nonce,
    ))
}
