use alloy::{rpc::client::RpcClient, transports::TransportResult};

#[derive(Clone)]
pub struct TonProvider {
    pub client: RpcClient,
}

impl TonProvider {
    pub fn new(client: RpcClient) -> Self {
        Self { client }
    }

    // JSON-RPC: method "sendMessage" (per TONX API), params { "boc": "<base64>" }
    pub async fn send_boc(
        &self,
        boc: String,
    ) -> TransportResult<serde_json::Value> {
        let params = serde_json::json!({ "boc": boc });
        // Try primary method name first
        match self.client.request("sendMessage", params.clone()).await {
            Ok(v) => Ok(v),
            Err(_) => {
                // Fallback to namespaced method some proxies expose
                self.client.request("ton_sendMessage", params).await
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
}
