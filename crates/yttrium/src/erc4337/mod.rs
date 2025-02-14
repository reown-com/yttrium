use {
    alloy::{
        primitives::B256,
        rpc::types::UserOperationReceipt,
        transports::{TransportError, TransportResult},
    },
    alloy_provider::Provider,
    serde_json::Value,
};

pub async fn get_user_operation_receipt(
    provider: &impl Provider,
    hash: B256,
) -> TransportResult<Option<UserOperationReceipt>> {
    let receipt = provider.get_user_operation_receipt(hash).await?;

    // For some reason Pimlico bundler doesn't include these fields
    // Workaround by injecting them in
    let value = receipt.map(|mut value| {
        if let Some(value) = value.as_object_mut() {
            if let Some(receipt) = value.get_mut("receipt") {
                if let Some(receipt) = receipt.as_object_mut() {
                    receipt.insert(
                        "type".to_owned(),
                        serde_json::Value::String("0x0".to_owned()),
                    );
                }
            }
            value.insert(
                "reason".to_owned(),
                serde_json::Value::String("".to_owned()),
            );
        }
        value
    });

    if let Some(value) = value {
        Ok(Some(
            serde_json::from_value::<UserOperationReceipt>(value)
                .map_err(TransportError::ser_err)?,
        ))
    } else {
        Ok(None)
    }
}

// Private (for now) so that it forces folks to use Pimlico-compatible `get_user_operation_receipt` above
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
trait CustomErc4337Api: Send + Sync {
    async fn get_user_operation_receipt(
        &self,
        user_op_hash: B256,
    ) -> TransportResult<Option<Value>>;
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl<P> CustomErc4337Api for P
where
    P: Provider,
{
    async fn get_user_operation_receipt(
        &self,
        user_op_hash: B256,
    ) -> TransportResult<Option<Value>> {
        self.client()
            .request("eth_getUserOperationReceipt", (user_op_hash,))
            .await
    }
}
