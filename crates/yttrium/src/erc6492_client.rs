use alloy::{
    network::Ethereum,
    primitives::{Address, Bytes, B256},
    providers::ReqwestProvider,
};
use erc6492::RpcError;

#[derive(uniffi::Object)]
pub struct Erc6492Client {
    provider: ReqwestProvider<Ethereum>,
}

// TODO use universal version: https://linear.app/reown/issue/RES-142/universal-provider-router
#[cfg_attr(feature = "uniffi", uniffi::export(async_runtime = "tokio"))]
impl Erc6492Client {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    pub fn new(rpc_url: String) -> Self {
        let url = rpc_url.parse().expect("Invalid RPC URL");
        let provider = ReqwestProvider::<Ethereum>::new_http(url);
        Self { provider }
    }

    pub async fn verify_signature(
        &self,
        signature: Bytes,
        address: Address,
        message_hash: B256,
    ) -> Result<bool, RpcError> {
        let verification = erc6492::verify_signature(
            signature,
            address,
            message_hash,
            &self.provider,
        )
        .await?;

        Ok(verification.is_valid())
    }
}
