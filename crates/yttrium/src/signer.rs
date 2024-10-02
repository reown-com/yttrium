use crate::user_operation::UserOperationV07;
use crate::{error::YttriumError, sign_service::SignService};
use alloy::{
    primitives::Address,
    signers::{
        local::{coins_bip39::English, MnemonicBuilder, PrivateKeySigner},
        SignerSync,
    },
};
use eyre;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

pub struct Signer {
    sign_service: Arc<Mutex<SignService>>,
}

impl Signer {
    pub fn new(sign_service: Arc<Mutex<SignService>>) -> Self {
        Self { sign_service }
    }

    pub fn signer_from_phrase(
        phrase: &str,
        _chain_id: u64,
    ) -> eyre::Result<Self> {
        let index = 0;
        let local_signer = MnemonicBuilder::<English>::default()
            .phrase(phrase)
            .index(index)?
            .build()?;
        let signer = Self::from(local_signer);
        Ok(signer)
    }

    pub async fn owner(&self) -> Address {
        let sign_service_clone = Arc::clone(&self.sign_service);
        let sign_service = sign_service_clone.lock().await;
        sign_service.owner()
    }

    pub async fn sign_message(&self, message: String) -> eyre::Result<Vec<u8>> {
        let sign_service_clone = Arc::clone(&self.sign_service);
        let sign_service = sign_service_clone.lock().await;
        let result = Self::sign_message_impl(message, &sign_service)?;
        Ok(result)
    }

    pub fn sign_user_operation_sync_v07(
        &self,
        uo: &crate::user_operation::UserOperationV07,
        ep: &Address,
        chain_id: u64,
    ) -> eyre::Result<UserOperationV07> {
        let sign_service_clone = Arc::clone(&self.sign_service);
        let sign_service = sign_service_clone.try_lock()?;
        let result = Self::sign_user_operation_impl_v07(
            uo,
            ep,
            chain_id,
            &sign_service,
        )?;
        Ok(result)
    }

    pub fn sign_message_sync(&self, message: String) -> eyre::Result<Vec<u8>> {
        let sign_service_clone = Arc::clone(&self.sign_service);
        let sign_service = sign_service_clone.try_lock()?;
        let result = Self::sign_message_impl(message, &sign_service)?;
        Ok(result)
    }

    pub fn sign_message_string_sync(
        &self,
        message: String,
    ) -> eyre::Result<String> {
        let sign_service_clone = Arc::clone(&self.sign_service);
        let sign_service = sign_service_clone.try_lock()?;
        let result = Self::sign_message_as_string_impl(message, &sign_service)?;
        Ok(result)
    }

    fn sign_user_operation_impl_v07(
        uo: &UserOperationV07,
        ep: &Address,
        chain_id: u64,
        sign_service: &MutexGuard<SignService>,
    ) -> eyre::Result<UserOperationV07> {
        let hash = uo.hash(ep, chain_id);
        let message_bytes = hash.0.to_vec();
        println!("message_bytes: {:?}", message_bytes.clone());

        let message = hex::encode(message_bytes);
        println!("message: {:?}", message.clone());

        let signature = sign_service.sign(message)?;
        let sig_vec: Vec<_> = signature.into();
        let mut user_operation = uo.clone();
        user_operation.signature = sig_vec.into();
        Ok(user_operation)
    }

    fn sign_message_impl(
        message: String,
        sign_service: &MutexGuard<SignService>,
    ) -> eyre::Result<Vec<u8>> {
        let signature =
            Self::sign_message_as_string_impl(message, sign_service)?;
        let signature_bytes: Vec<u8> = signature.into();
        Ok(signature_bytes)
    }

    fn sign_message_as_string_impl(
        message: String,
        sign_service: &MutexGuard<SignService>,
    ) -> eyre::Result<String> {
        let signature = sign_service.sign(message)?;
        Ok(signature)
    }
}

impl<S> From<S> for Signer
where
    S: alloy::signers::SignerSync<alloy::signers::Signature>
        + Send
        + Sync
        + 'static,
    S: alloy::signers::Signer<alloy::signers::Signature>
        + Send
        + Sync
        + 'static,
{
    fn from(signer: S) -> Self {
        let owner = signer.address();
        let sign_fn = Box::new(move |msg: String| {
            signer
                .sign_message_sync(msg.as_bytes())
                .map(|sig| sig.as_bytes().to_vec())
                .map(hex::encode)
                .map_err(YttriumError::from)
        });
        let sign_service_s = SignService::new(sign_fn, owner);
        let sign_service = Arc::new(Mutex::new(sign_service_s));
        Signer::new(sign_service)
    }
}

pub fn sign_user_operation_v07_with_ecdsa_and_sign_service(
    uo: &UserOperationV07,
    ep: &Address,
    chain_id: u64,
    signer: PrivateKeySigner,
    sign_service: &Arc<Mutex<SignService>>,
) -> eyre::Result<UserOperationV07> {
    let hash = uo.hash(ep, chain_id);

    println!("hash: {:?}", hash.clone());

    let message = hash.0;

    println!("message: {:?}", message.clone());

    let message_bytes = message.to_vec();

    println!("message_bytes: {:?}", message_bytes.clone());

    let sign_service = Arc::clone(sign_service);
    let sign_service = sign_service.try_lock()?;

    let message_hex = hex::encode(message_bytes.clone());

    let signature_native = sign_service.sign(message_hex)?;

    println!("signature_native: {:?}", signature_native);

    let signature_native_bytes = hex::decode(signature_native.clone())?;

    {
        let signature = signer.sign_message_sync(&message_bytes)?;
        println!("signature: {:?}", signature);
        let sig_vec: Vec<u8> = signature.into();
        let sig_vec_hex = hex::encode(sig_vec.clone());
        println!("sig_vec_hex: {:?}", sig_vec_hex);

        assert_eq!(
            sig_vec, signature_native_bytes,
            "sig_vec != signature_native_bytes"
        );
        assert_eq!(
            sig_vec_hex, signature_native,
            "sig_vec_hex != signature_native"
        );
    }
    let sig_vec = signature_native_bytes;

    let mut user_operation = uo.clone();
    user_operation.signature = sig_vec.into();
    Ok(user_operation)
}

pub fn sign_user_operation_v07_with_ecdsa(
    uo: &UserOperationV07,
    ep: &Address,
    chain_id: u64,
    signer: PrivateKeySigner,
) -> eyre::Result<UserOperationV07> {
    let hash = uo.hash(ep, chain_id);

    println!("hash: {:?}", hash.clone());

    let message = hash.0;

    println!("message: {:?}", message.clone());

    let message_bytes = message.to_vec();

    println!("message_bytes: {:?}", message_bytes.clone());

    let signature = signer.sign_message_sync(&message_bytes)?;
    println!("signature: {:?}", signature);
    let sig_vec: Vec<u8> = signature.into();
    println!("hex::encode(sig_vec): {:?}", hex::encode(sig_vec.clone()));

    let mut user_operation = uo.clone();
    user_operation.signature = sig_vec.into();
    Ok(user_operation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eyre::ensure;

    pub const ETHERIEUM_MAINNET_CHAIN_ID: u64 = 1;
    pub const MNEMONIC_PHRASE: &str =
        "test test test test test test test test test test test junk";
    pub const CHAIN_ID: u64 = ETHERIEUM_MAINNET_CHAIN_ID;

    #[tokio::test]
    async fn test_sign_message_sync() -> eyre::Result<()> {
        let signer = Signer::signer_from_phrase(MNEMONIC_PHRASE, CHAIN_ID)?;
        let message = "Hello, world!".to_string();
        let signature = signer.sign_message_sync(message)?;
        ensure!(!signature.is_empty(), "Signature is empty");
        Ok(())
    }
}
