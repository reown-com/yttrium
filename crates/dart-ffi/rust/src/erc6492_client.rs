use crate::dart_ffi::Erc6492Error;
use alloy::{
    network::Ethereum,
    primitives::{Address, Bytes, B256},
    providers::ReqwestProvider,
};

pub struct Erc6492Client {
    provider: ReqwestProvider<Ethereum>,
}

impl Erc6492Client {
    pub fn new(rpc_url: String) -> Self {
        let url = rpc_url.parse().expect("Invalid RPC URL");
        let provider = ReqwestProvider::<Ethereum>::new_http(url);
        Self { provider }
    }

    pub async fn verify_signature(
        &self,
        signature: String,
        address: String,
        message_hash: String,
    ) -> Result<bool, Erc6492Error> {
        let signature = signature
            .parse::<Bytes>()
            .map_err(|e| Erc6492Error::InvalidSignature(e.to_string()))?;
        let address = address
            .parse::<Address>()
            .map_err(|e| Erc6492Error::InvalidAddress(e.to_string()))?;
        let message_hash = message_hash
            .parse::<B256>()
            .map_err(|e| Erc6492Error::InvalidMessageHash(e.to_string()))?;

        let verification = erc6492::verify_signature(
            signature,
            address,
            message_hash,
            &self.provider,
        )
        .await
        .map_err(|e| Erc6492Error::Verification(e.to_string()))?;

        Ok(verification.is_valid())
    }
}
