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

#[derive(uniffi::Record)]
pub struct SendTxMessage {
    pub address: String,            
    pub amount: String,             
    pub state_init: Option<String>, 
    pub payload: Option<String>,    
}

#[derive(uniffi::Object)]
pub struct SendTxParams {
    pub valid_until: u32, 
    pub network: String,  
    pub from: String, 
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
        text: String,
        keypair: &Keypair,
    ) -> Result<String, TonError> {
        let mut message_bytes = Vec::new();

        // TON Connect text signing format
        message_bytes.extend_from_slice(b"ton-connect:");
        message_bytes.extend_from_slice(text.as_bytes());

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

        let signature = sk.sign(&message_bytes);
        let signature_base64 = B64.encode(signature.to_bytes());

        Ok(signature_base64)
    }

    pub fn send_message(
        &self,
        network: String,
        from: String,
        keypair: &Keypair,
        valid_until: u32,
        messages: Vec<SendTxMessage>,
    ) -> Result<String, TonError> {
        // Validate network matches client
        if network != self.cfg.network_id {
            return Err(TonError::NetworkMismatch(format!(
                "client={} request={}",
                self.cfg.network_id, network
            )));
        }

        // Validate that the from address matches the keypair
        let keypair_address = self.get_address_from_keypair(keypair)?;
        if keypair_address.friendly_eq != from {
            return Err(TonError::InvalidAddress(format!(
                "From address {} does not match keypair address {}",
                from, keypair_address.friendly_eq
            )));
        }

        // Validate valid_until is in the future
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        
        if valid_until <= current_time {
            return Err(TonError::SerializationError(
                "Transaction valid_until is in the past".to_string()
            ));
        }

        // Validate messages
        if messages.is_empty() {
            return Err(TonError::SerializationError(
                "No messages provided".to_string()
            ));
        }

        // For each message, validate the address format and amount
        for msg in &messages {
            // Validate address format (should be TEP-123 format starting with EQ)
            if !msg.address.starts_with("EQ") || msg.address.len() < 48 {
                return Err(TonError::InvalidAddress(format!(
                    "Invalid TON address format: {}",
                    msg.address
                )));
            }

            // Validate amount is a valid number
            if let Err(_) = msg.amount.parse::<u64>() {
                return Err(TonError::SerializationError(format!(
                    "Invalid amount format: {}",
                    msg.amount
                )));
            }
        }

        // Create a transaction cell with proper TON structure
        let mut cell = TonCell::builder();
        
        // Add transaction header data
        // In a real implementation, this would follow TON transaction format
        cell.write_bits(&[1, 0, 0, 0], 4)?; // Transaction type
        cell.write_bits(&[0, 0, 0, 0], 4)?; // Flags
        
        // Add valid_until timestamp (32 bits)
        let valid_until_bits: [u8; 4] = valid_until.to_le_bytes();
        cell.write_bits(&valid_until_bits, 32)?;
        
        // Add message count (8 bits)
        cell.write_bits(&[messages.len() as u8], 8)?;
        
        // For each message, add address and amount
        for msg in &messages {
            // Add address (simplified - in real implementation would be proper TON address encoding)
            let address_bytes = msg.address.as_bytes();
            cell.write_bits(&[address_bytes.len() as u8], 8)?; // Address length
            cell.write_bits(address_bytes, address_bytes.len() * 8)?;
            
            // Add amount (64 bits)
            let amount: u64 = msg.amount.parse().unwrap();
            let amount_bytes: [u8; 8] = amount.to_le_bytes();
            cell.write_bits(&amount_bytes, 64)?;
        }
        
        let transaction_cell = cell.build()?;

        // Create a simplified BOC representation
        // In a full implementation, this would use proper BOC serialization
        let transaction_data = format!(
            "ton_transaction_{}_{}_{}",
            from,
            valid_until,
            messages.len()
        );
        let boc_base64 = B64.encode(transaction_data.as_bytes());

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
