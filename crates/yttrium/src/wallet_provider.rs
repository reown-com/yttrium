use {
    crate::wallet_service_api::{GetAssetsParams, GetAssetsResult},
    alloy::{rpc::client::RpcClient, transports::TransportResult},
};

pub struct WalletProvider {
    pub client: RpcClient,
}

impl WalletProvider {
    pub async fn wallet_get_assets(
        &self,
        params: GetAssetsParams,
    ) -> TransportResult<GetAssetsResult> {
        self.client.request("wallet_getAssets", params).await
    }
}
