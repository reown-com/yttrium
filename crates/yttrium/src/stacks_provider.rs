use alloy::{
    primitives::Bytes, rpc::client::RpcClient, transports::TransportResult,
};

pub struct StacksProvider {
    pub client: RpcClient,
}

impl StacksProvider {
    pub async fn stacks_transactions(
        &self,
        params: Bytes,
    ) -> TransportResult<serde_json::Value> {
        // TODO proper return type
        self.client.request("stacks_transactions", (params,)).await
    }
}
