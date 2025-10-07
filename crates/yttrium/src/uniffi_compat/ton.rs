#[cfg(feature = "chain_abstraction_client")]
use crate::chain_abstraction::pulse::PulseMetadata;
use {
    crate::{
        blockchain_api::BLOCKCHAIN_API_URL_PROD, provider_pool::ProviderPool,
    },
    base64::{engine::general_purpose::STANDARD as B64, Engine as _},
    ed25519_dalek::{Signer, SigningKey},
    rand::rngs::OsRng,
    relay_rpc::domain::ProjectId,
    reqwest::Client as ReqwestClient,
    std::time::{SystemTime, UNIX_EPOCH},
    ton_lib::ton_lib_core::types::TonAddress,
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
    cfg: TonClientConfig,
    provider_pool: ProviderPool,
}

#[uniffi::export(async_runtime = "tokio")]
impl TONClient {
    #[uniffi::constructor]
    pub fn new(
        cfg: TonClientConfig,
        project_id: ProjectId,
        #[cfg(feature = "chain_abstraction_client")]
        pulse_metadata: PulseMetadata,
    ) -> Self {
        let client = ReqwestClient::new();

        let provider_pool = ProviderPool::new(
            project_id,
            client,
            #[cfg(feature = "chain_abstraction_client")]
            pulse_metadata,
            BLOCKCHAIN_API_URL_PROD.parse().unwrap(),
        );

        Self { cfg, provider_pool }
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

    pub async fn send_message(
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
        let current_time =
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
                as u32;

        if valid_until <= current_time {
            return Err(TonError::SerializationError(
                "Transaction valid_until is in the past".to_string(),
            ));
        }

        // Validate messages
        if messages.is_empty() {
            return Err(TonError::SerializationError(
                "No messages provided".to_string(),
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

        // Use ton-rs library to create and send the message
        self.broadcast_message(from, keypair, valid_until, messages).await
    }

    async fn broadcast_message(
        &self,
        from: String,
        keypair: &Keypair,
        valid_until: u32,
        messages: Vec<SendTxMessage>,
    ) -> Result<String, TonError> {
        use ton_lib::{
            block_tlb::{
                CommonMsgInfo, CommonMsgInfoInt, CurrencyCollection, Msg,
            },
            ton_lib_core::traits::tlb::TLB,
            wallet::{KeyPair as TonKeyPair, TonWallet, WalletVersion},
        };

        if messages.len() != 1 {
            return Err(TonError::SerializationError(
                "Only single message transfers supported".to_string(),
            ));
        }

        let msg = &messages[0];
        let to_addr = msg
            .address
            .parse::<ton_lib::ton_lib_core::types::TonAddress>()
            .map_err(|e| {
                TonError::InvalidAddress(format!("Invalid to address: {}", e))
            })?;
        let amount = msg.amount.parse::<u128>().map_err(|e| {
            TonError::SerializationError(format!("Invalid amount: {}", e))
        })?;

        // Build ton-lib wallet from provided keys (secret_key must be 64 bytes: sk||pk)
        let sk = B64.decode(&keypair.sk).map_err(|e| {
            TonError::SerializationError(format!(
                "Invalid private key base64: {}",
                e
            ))
        })?;
        if sk.len() != 32 {
            return Err(TonError::SerializationError(
                "Invalid private key length".to_string(),
            ));
        }
        let pk = hex::decode(&keypair.pk).map_err(|e| {
            TonError::SerializationError(format!(
                "Invalid public key hex: {}",
                e
            ))
        })?;
        if pk.len() != 32 {
            return Err(TonError::SerializationError(
                "Invalid public key length".to_string(),
            ));
        }
        let mut secret_key = Vec::with_capacity(64);
        secret_key.extend_from_slice(&sk);
        secret_key.extend_from_slice(&pk);
        let ton_keypair = TonKeyPair { public_key: pk, secret_key };
        let wallet =
            TonWallet::new(WalletVersion::V4R2, ton_keypair).map_err(|e| {
                TonError::TonCoreError(format!(
                    "Failed to create wallet: {}",
                    e
                ))
            })?;

        // Fetch seqno via blockchain API (getWalletInformation with fallback)
        let ton_provider = self
            .provider_pool
            .get_ton_client(&self.cfg.network_id, None, None)
            .await;

        // Fetch seqno using only getAddressInformation (per TONX Accounts API)
        let addr_info = ton_provider
            .get_address_information(&from)
            .await
            .map_err(|e| TonError::TonCoreError(format!(
                "Failed getAddressInformation: {}",
                e
            )))?;
        
        tracing::info!("addr_info: {addr_info:?}");

        // Some backends return a top-level object, others nest under result
        let root = addr_info.get("result").unwrap_or(&addr_info);

        // If account state is explicitly uninitialized, use seqno=0
        let is_uninitialized = root
            .get("state")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case("uninitialized"))
            .unwrap_or(false);

        let mut seqno_u64 = root
            .get("block_id").and_then(|b| b.get("seqno")).and_then(|v| v.as_u64())
            .or_else(|| root.get("block_id").and_then(|b| b.get("seqno")).and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok()))
            .or_else(|| root.get("last_block_id").and_then(|b| b.get("seqno")).and_then(|v| v.as_u64()))
            .or_else(|| root.get("last_block_id").and_then(|b| b.get("seqno")).and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok()))
            .or_else(|| root.get("blockId").and_then(|b| b.get("seqno")).and_then(|v| v.as_u64()))
            .or_else(|| root.get("blockId").and_then(|b| b.get("seqno")).and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok()))
            .or_else(|| root.get("lastBlockId").and_then(|b| b.get("seqno")).and_then(|v| v.as_u64()))
            .or_else(|| root.get("lastBlockId").and_then(|b| b.get("seqno")).and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok()))
            .or_else(|| root.get("seqno").and_then(|v| v.as_u64()))
            .or_else(|| root.get("seqno").and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok()));

        let seqno = if is_uninitialized { 0 } else {
            seqno_u64
                .ok_or_else(|| TonError::TonCoreError(
                    "Missing seqno in getAddressInformation (expected block_id.seqno)".into(),
                ))? as u32
        };

        // Build internal transfer message
        let int_msg = Msg {
            info: CommonMsgInfo::Int(CommonMsgInfoInt {
                ihr_disabled: false,
                bounce: false,
                bounced: false,
                src: ton_lib::ton_lib_core::types::tlb_core::MsgAddress::NONE,
                dst: ton_lib::ton_lib_core::types::tlb_core::MsgAddress::Int(
                    to_addr.to_msg_address_int(),
                ),
                value: CurrencyCollection::new(amount),
                ihr_fee: ton_lib::block_tlb::Coins::ZERO,
                fwd_fee: ton_lib::block_tlb::Coins::ZERO,
                created_lt: 0,
                created_at: 0,
            }),
            init: None,
            body: ton_lib::ton_lib_core::types::tlb_core::TLBEitherRef::new(
                ton_lib::ton_lib_core::cell::TonCell::EMPTY,
            ),
        };

        let ext_in_msg = wallet
            .create_ext_in_msg(
                vec![int_msg.to_cell_ref()?],
                seqno,
                valid_until,
                false,
            )
            .map_err(|e| {
                TonError::TonCoreError(format!(
                    "Failed to create external message: {}",
                    e
                ))
            })?;

        // Serialize to BOC
        let boc = ext_in_msg.to_boc().map_err(|e| {
            TonError::TonCoreError(format!("Failed to serialize BOC: {}", e))
        })?;
        let boc_base64 = B64.encode(&boc);

        let response =
            ton_provider.send_boc(boc_base64).await.map_err(|e| {
                TonError::TonCoreError(format!("Failed to send BOC: {}", e))
            })?;

        // Extract result from response
        if let Some(error) = response.get("error") {
            return Err(TonError::TonCoreError(format!(
                "Blockchain API error: {}",
                error
            )));
        }

        let result = response
            .get("result")
            .and_then(|v| v.as_str())
            .unwrap_or("Message sent successfully");

        Ok(result.to_string())
    }
}
