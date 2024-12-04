use {
    crate::error::YttriumError,
    alloy::{
        primitives::Address,
        signers::{
            local::{coins_bip39::English, MnemonicBuilder, PrivateKeySigner},
            SignerSync,
        },
    },
    std::sync::Arc,
    tokio::sync::Mutex,
};

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

    pub fn new_with_mnemonic(mnemonic: String) -> Self {
        let phrase = mnemonic.clone();
        let index: u32 = 0;

        let wallet = MnemonicBuilder::<English>::default()
            .phrase(phrase.to_string())
            .index(index)
            .unwrap()
            .build()
            .unwrap();

        let alloy_signer = PrivateKeySigner::from(wallet.clone());

        let owner = alloy_signer.address();

        SignService {
            sign_fn: Arc::new(Mutex::new(Box::new(move |msg: String| {
                let message_bytes = hex::decode(msg).unwrap();
                let signature =
                    alloy_signer.sign_message_sync(&message_bytes)?;
                let sig_vec: Vec<u8> = signature.into();
                let sig_vec_hex = hex::encode(sig_vec.clone());
                println!("sig_vec_hex: {:?}", sig_vec_hex);
                Ok(sig_vec_hex)
            }))),
            owner,
        }
    }
}

pub fn address_from_string(address: &str) -> eyre::Result<Address> {
    let address = address.parse::<Address>()?;
    Ok(address)
}
