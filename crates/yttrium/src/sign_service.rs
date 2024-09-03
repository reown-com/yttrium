use crate::error::YttriumError;
use alloy::{
    primitives::Address,
    signers::{
        k256::ecdsa::SigningKey,
        local::{coins_bip39::English, LocalSigner, MnemonicBuilder},
    },
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type SignFN =
    dyn Fn(String) -> Result<String, YttriumError> + Send + 'static;

pub type BoxSignFN = Box<SignFN>;

pub struct SignService {
    sign_fn: Arc<Mutex<BoxSignFN>>,
    owner: Address,
}

impl SignService {
    pub fn new(sign_fn: BoxSignFN, owner: Address) -> Self {
        SignService { sign_fn: Arc::new(Mutex::new(sign_fn)), owner }
    }

    pub fn owner(&self) -> Address {
        self.owner
    }

    pub fn sign(&self, message: String) -> Result<String, YttriumError> {
        let sign_fn = self.sign_fn.clone();
        let sign_fn = sign_fn
            .try_lock()
            .map_err(|e| YttriumError { message: e.to_string() })?;
        (sign_fn)(message)
    }
}

impl SignService {
    pub fn mock() -> Self {
        SignService {
            sign_fn: Arc::new(Mutex::new(Box::new(|_| Ok("".to_string())))),
            owner: Address::ZERO,
        }
    }

    pub async fn mock_with_mnemonic(mnemonic: String) -> Self {
        let phrase = mnemonic.clone();
        let index: u32 = 0;

        let wallet = MnemonicBuilder::<English>::default()
            .phrase(phrase.to_string())
            .index(index)
            .unwrap()
            .build()
            .unwrap();

        let alloy_signer =
            alloy::signers::local::PrivateKeySigner::from(wallet.clone());

        let signer = crate::signer::Signer::from(alloy_signer.clone());

        let owner = alloy_signer.address();

        SignService {
            sign_fn: Arc::new(Mutex::new(Box::new(move |msg: String| {
                let signature = signer.sign_message_string_sync(msg).unwrap();

                Ok(signature)
            }))),
            owner,
        }
    }
}

pub fn address_from_string(address: &str) -> eyre::Result<Address> {
    let address = address.parse::<Address>()?;
    Ok(address)
}
