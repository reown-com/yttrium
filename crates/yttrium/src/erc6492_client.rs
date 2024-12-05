use alloy::{
    network::Ethereum,
    primitives::{Address, Bytes, B256},
    providers::ReqwestProvider,
};

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct Erc6492Client {
    provider: ReqwestProvider<Ethereum>,
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum Erc6492Error {
    // TODO we can remove this stringification of the error when https://mozilla.github.io/uniffi-rs/next/udl/remote_ext_types.html#remote-types is available
    #[error("RpcError: {0}")]
    RpcError(String),
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
    ) -> Result<bool, Erc6492Error> {
        let verification = erc6492::verify_signature(
            signature,
            address,
            message_hash,
            &self.provider,
        )
        .await
        .map_err(|e| Erc6492Error::RpcError(e.to_string()))?;

        Ok(verification.is_valid())
    }
}
