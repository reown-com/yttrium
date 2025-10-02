use {
    base64::{engine::general_purpose::STANDARD as B64, Engine as _},
    ed25519_dalek::{Signer, SigningKey},
    rand::rngs::OsRng,
    std::{
        time::{SystemTime, UNIX_EPOCH},
    },
    ton_lib::ton_lib_core::{boc::BOC, cell::TonCell, types::TonAddress},
};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum TonError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Network mismatch: {0}")]
    NetworkMismatch(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Signing error: {0}")]
    SigningError(String),

    #[error("TON core error: {0}")]
    TonCoreError(String),
}

impl From<ton_lib::ton_lib_core::error::TLCoreError> for TonError {
    fn from(err: ton_lib::ton_lib_core::error::TLCoreError) -> Self {
        TonError::TonCoreError(err.to_string())
    }
}

#[derive(uniffi::Record)]
pub struct Keypair {
    pub sk: String, // base64 encoded private key
    pub pk: String, // hex encoded public key
}

#[derive(uniffi::Record)]
pub struct WalletIdentity {
    pub workchain: i8,
    pub raw_hex: String,
    pub friendly_eq: String,
}

#[derive(uniffi::Record)]
pub struct TonClientConfig {
    pub network_id: String,
}

#[derive(uniffi::Object)]
pub struct SendTxMessage {
    pub address: String,            // TEP-123 format address
    pub amount: String,             // nanotons as string
    pub state_init: Option<String>, // base64 encoded BoC
    pub payload: Option<String>,    // base64 encoded BoC
}

#[derive(uniffi::Object)]
pub struct SendTxParams {
    pub valid_until: u32, // unix seconds
    pub network: String,  // "-239" / "-3"
    pub from: String,     // TEP-123 format address
    pub messages: Vec<SendTxMessage>,
}

#[derive(uniffi::Object)]
pub struct TONClient {
    cfg: TonClientConfig
}

#[uniffi::export]
impl TONClient {
    #[uniffi::constructor]
    pub fn new(cfg: TonClientConfig) -> Result<Self, TonError> {
        Ok(Self { cfg })
    }

    pub fn generate_keypair(&self) -> Keypair {
        let sk = SigningKey::generate(&mut OsRng);
        let pk = sk.verifying_key();
        Keypair {
            sk: B64.encode(sk.to_bytes()),
            pk: hex::encode(pk.to_bytes()),
        }
    }

    pub fn get_address_from_keypair(
        &self,
        keypair: &Keypair,
    ) -> Result<WalletIdentity, TonError> {
        // Decode public key from hex string
        let pk_bytes = hex::decode(&keypair.pk).map_err(|e| {
            TonError::SerializationError(format!(
                "Invalid public key hex: {}",
                e
            ))
        })?;

        if pk_bytes.len() != 32 {
            return Err(TonError::SerializationError(
                "Invalid public key length".to_string(),
            ));
        }

        let mut pk_array = [0u8; 32];
        pk_array.copy_from_slice(&pk_bytes);

        // Create TON address from public key (workchain 0, wallet_id 0)
        let address = TonAddress::new(
            0,
            ton_lib::ton_lib_core::cell::TonHash::from(pk_array),
        );

        let friendly = address.to_string();
        let raw = format!(
            "{}:{}",
            address.workchain,
            hex::encode(address.hash.as_slice())
        );

        Ok(WalletIdentity {
            workchain: address.workchain as i8,
            raw_hex: raw,
            friendly_eq: friendly,
        })
    }

    pub fn sign_data(
        &self,
        from: String,
        text: String,
        keypair: &Keypair,
    ) -> Result<String, TonError> {
        let timestamp =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Build message to sign based on text payload
        let domain = "ton.app".to_string(); // Default domain
        let mut message_bytes = Vec::new();

        // TON Connect text signing format
        message_bytes.extend_from_slice(b"ton-connect:");
        message_bytes.extend_from_slice(text.as_bytes());

        // Decode private key from base64 string
        let sk_bytes = B64.decode(&keypair.sk).map_err(|e| {
            TonError::SerializationError(format!(
                "Invalid private key base64: {}",
                e
            ))
        })?;

        if sk_bytes.len() != 32 {
            return Err(TonError::SerializationError(
                "Invalid private key length".to_string(),
            ));
        }

        let mut sk_array = [0u8; 32];
        sk_array.copy_from_slice(&sk_bytes);
        let sk = SigningKey::from_bytes(&sk_array);

        // Sign the message using the provided keypair
        let signature = sk.sign(&message_bytes);
        let signature_base64 = B64.encode(signature.to_bytes());

        // Decode public key from hex string
        let pk_bytes = hex::decode(&keypair.pk).map_err(|e| {
            TonError::SerializationError(format!(
                "Invalid public key hex: {}",
                e
            ))
        })?;

        if pk_bytes.len() != 32 {
            return Err(TonError::SerializationError(
                "Invalid public key length".to_string(),
            ));
        }

        let mut pk_array = [0u8; 32];
        pk_array.copy_from_slice(&pk_bytes);

        // Get wallet address in raw format from the keypair
        let address = TonAddress::new(
            0,
            ton_lib::ton_lib_core::cell::TonHash::from(pk_array),
        );
        let wallet_address = format!(
            "{}:{}",
            address.workchain,
            hex::encode(address.hash.as_slice())
        );

        Ok(signature_base64)
    }

    pub fn send_message(
        &self,
        network: String,
        from: String,
        keypair: &Keypair,
    ) -> Result<String, TonError> {
        // Validate network matches client
        if network != self.cfg.network_id {
            return Err(TonError::NetworkMismatch(format!(
                "client={} request={}",
                self.cfg.network_id, network
            )));
        }

        // For now, create a simple transaction message
        // In a full implementation, this would create proper TON transaction messages
        let mut cell = TonCell::builder();

        // Add basic transaction data
        cell.write_bits(&[1, 2, 3, 4], 32)?; // Some dummy data
        let transaction_cell = cell.build()?;

        // Create a simple BOC representation
        // In a full implementation, this would use proper BOC serialization
        let boc_data = format!("ton_transaction_{}", from);
        let boc_base64 = B64.encode(boc_data.as_bytes());

        Ok(boc_base64)
    }
}

fn boc_from_base64(b64: &str) -> Result<TonCell, TonError> {
    let bytes = B64
        .decode(b64)
        .map_err(|e| TonError::SerializationError(e.to_string()))?;
    let _boc = BOC::from_bytes(&bytes)
        .map_err(|e| TonError::SerializationError(e.to_string()))?;
    // For now, return a dummy cell since we don't have access to the root method
    // In a full implementation, this would properly extract the root cell
    let mut cell = TonCell::builder();
    cell.write_bits(&[0], 1)
        .map_err(|e| TonError::SerializationError(e.to_string()))?;
    Ok(cell.build().map_err(|e| TonError::SerializationError(e.to_string()))?)
}
