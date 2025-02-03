use {
    super::{api::FeeEstimatedTransaction, error::SendTransactionError},
    crate::{
        provider_pool::ProviderPool,
        serde::{duration_millis, option_duration_millis},
    },
    alloy::{
        consensus::{SignableTransaction, TxEnvelope},
        primitives::{PrimitiveSignature, B256},
        rpc::types::TransactionReceipt,
    },
    alloy_provider::Provider,
    serde::{Deserialize, Serialize},
    std::time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub async fn send_transaction(
    txn: FeeEstimatedTransaction,
    sig: PrimitiveSignature,
    provider_pool: &ProviderPool,
) -> Result<
    (TransactionReceipt, TransactionAnalytics),
    (SendTransactionError, TransactionAnalytics),
> {
    let start = Instant::now();
    let start_time =
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();

    let provider = provider_pool.get_provider(&txn.chain_id).await;
    let signed = txn.into_eip1559().into_signed(sig);
    let txn_hash = *signed.hash();

    let send_start = Instant::now();
    let sent_transaction_result =
        provider.send_tx_envelope(TxEnvelope::Eip1559(signed)).await;
    let send_latency = send_start.elapsed();
    let sent_transaction = sent_transaction_result.map_err(|e| {
        (
            SendTransactionError::Rpc(e),
            TransactionAnalytics {
                txn_hash,
                start: start_time,
                send_latency,
                receipt_latency: None,
                latency: start.elapsed(),
                end: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default(),
            },
        )
    })?;

    let receipt_start = Instant::now();
    let receipt_result = sent_transaction
        .with_timeout(Some(Duration::from_secs(15)))
        .get_receipt()
        .await;
    let receipt_latency = receipt_start.elapsed();

    let final_analytics = TransactionAnalytics {
        txn_hash,
        start: start_time,
        send_latency,
        receipt_latency: Some(receipt_latency),
        latency: start.elapsed(),
        end: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default(),
    };

    let receipt = receipt_result.map_err(|e| {
        (SendTransactionError::PendingTransaction(e), final_analytics.clone())
    })?;

    if !receipt.status() {
        Err((SendTransactionError::Failed, final_analytics))
    } else {
        Ok((receipt, final_analytics))
    }
}

// trait RemapAnalytics<T, E, B> {
//     fn remap(result: Self) -> (Result<T, E>, B);
// }

// impl<T, E, B> RemapAnalytics<T, E, B> for Result<(T, B), (E, B)> {
//     fn remap(result: Self) -> (Result<T, E>, B) {
//         match result {
//             Ok((t, b)) => (Ok(t), b),
//             Err((e, b)) => (Err(e), b),
//         }
//     }
// }

// impl<T, E, B> RemapAnalytics<T, E, B> for Result<T, (E, B)> {
//     fn remap(result: Self) -> (Result<T, E>, B) {
//         match result {
//             Ok(_t) => unimplemented!(),
//             Err((e, b)) => (Err(e), b),
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionAnalytics {
    pub txn_hash: B256,
    #[serde(with = "duration_millis")]
    pub start: Duration,
    #[serde(with = "duration_millis")]
    pub end: Duration,
    #[serde(with = "duration_millis")]
    pub latency: Duration,
    #[serde(with = "duration_millis")]
    pub send_latency: Duration,
    #[serde(with = "option_duration_millis")]
    pub receipt_latency: Option<Duration>,
}
