use alloy::{rpc::client::RpcClient, transports::TransportResult};

#[derive(Clone)]
pub struct TonProvider {
    pub client: RpcClient,
}

impl TonProvider {
    pub fn new(client: RpcClient) -> Self {
        Self { client }
    }

    // JSON-RPC: method "ton_sendBoc" (staging), params ["<base64>"]
    pub async fn send_boc(
        &self,
        boc: String,
    ) -> TransportResult<serde_json::Value> {
        let params = serde_json::json!([boc]);
        match self.client.request("ton_sendBoc", params).await {
            Ok(v) => Ok(v),
            Err(m) => {
                tracing::error!("Error sending message: {}", m);
                Err(m)
            }
        }
    }

    pub async fn get_address_information(
        &self,
        address: &str,
    ) -> TransportResult<serde_json::Value> {
        let params = serde_json::json!({ "address": address });
        let response: serde_json::Value =
            self.client.request("getAddressInformation", params).await?;
        Ok(response)
    }

    pub async fn get_wallet_information(
        &self,
        address: &str,
    ) -> TransportResult<serde_json::Value> {
        let params = serde_json::json!({ "address": address });
        let response: serde_json::Value =
            self.client.request("getWalletInformation", params).await?;
        Ok(response)
    }
}
