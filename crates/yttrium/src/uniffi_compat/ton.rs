#[cfg(feature = "chain_abstraction_client")]
use crate::pulse::PulseMetadata;
use {
    crate::{
        blockchain_api::BLOCKCHAIN_API_URL_PROD,
        provider_pool::{ProviderPool, network},
    },
    data_encoding::BASE64,
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
    pub friendly: String,
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

/// TON session properties for WalletConnect session approval.
/// These properties are required by TON Connect for signature verification
/// and wallet address computation.
#[derive(uniffi::Record)]
pub struct TonSessionProperties {
    /// Hex-encoded Ed25519 public key
    pub public_key: String,
    /// Base64-encoded StateInit BOC (Bag of Cells)
    pub state_init: String,
}

#[derive(uniffi::Object)]
pub struct TonClient {
    cfg: TonClientConfig,
    provider_pool: ProviderPool,
}

#[uniffi::export(async_runtime = "tokio")]
impl TonClient {
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
            sk: BASE64.encode(sk.to_bytes().as_ref()),
            pk: hex::encode(pk.to_bytes()),
        }
    }

    pub fn generate_keypair_from_ton_mnemonic(
        &self,
        mnemonic: String,
    ) -> Result<Keypair, TonError> {
        let mnemonic = ton_lib::wallet::Mnemonic::from_str(&mnemonic, None)
            .map_err(|e| {
                TonError::TonCoreError(format!("Invalid mnemonic: {}", e))
            })?;

        let key_pair = mnemonic.to_key_pair().map_err(|e| {
            TonError::TonCoreError(format!("Invalid keypair: {}", e))
        })?;

        let sk_bytes: [u8; 64] =
            key_pair.secret_key.try_into().map_err(|_| {
                TonError::TonCoreError("Invalid secret key length".to_string())
            })?;
        let sk = SigningKey::from_keypair_bytes(&sk_bytes).map_err(|e| {
            TonError::TonCoreError(format!("Invalid keypair bytes: {}", e))
        })?;
        let pk = sk.verifying_key();

        Ok(Keypair {
            sk: BASE64.encode(sk.to_bytes().as_ref()),
            pk: hex::encode(pk.to_bytes()),
        })
    }

    pub fn generate_keypair_from_bip39_mnemonic(
        &self,
        mnemonic: &str,
    ) -> Result<Keypair, TonError> {
        let mnemonic =
            bip39::Mnemonic::from_phrase(mnemonic, bip39::Language::English)
                .map_err(|e| TonError::TonCoreError(e.to_string()))?;

        // Seed::new() derives a 512-bit (64-byte) seed using PBKDF2-HMAC-SHA512 from the mnemonic + optional passphrase, as defined in the BIP-39 spec.
        let seed = bip39::Seed::new(&mnemonic, "");
        // But Ed25519’s private (signing) key is 32 bytes (256-bit scalar), so can’t feed a 64-byte BIP-39 seed directly into ed25519_dalek::SigningKey::from_bytes()
        let sk_bytes: [u8; 32] =
            seed.as_bytes()[0..32].try_into().map_err(|_| {
                TonError::TonCoreError("Invalid seed length".to_string())
            })?;
        // The 64-byte BIP-39 seed is deterministic from the mnemonic.
        // Every byte of it is pseudorandom and cryptographically strong (512 bits of entropy from your original entropy + PBKDF2 mixing).
        // To derive an Ed25519 key, we simply need one 256-bit scalar.
        // Taking the first 32 bytes of that seed gives us a reproducible, full-entropy 256-bit value,
        // the same approach used by several chains (e.g., Solana, Cardano, NEAR) when converting a BIP-39 mnemonic to an Ed25519 keypair.
        let sk = SigningKey::from_bytes(&sk_bytes);
        let pk = sk.verifying_key();

        Ok(Keypair {
            sk: BASE64.encode(sk.to_bytes().as_ref()),
            pk: hex::encode(pk.to_bytes()),
        })
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

        let sk_bytes = BASE64.decode(keypair.sk.as_bytes()).map_err(|e| {
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

        // Build ton-lib wallet keypair (secret_key must be 64 bytes: sk||pk)
        let mut secret_key = Vec::with_capacity(64);
        secret_key.extend_from_slice(&sk_bytes);
        secret_key.extend_from_slice(&pk_bytes);
        let ton_keypair =
            ton_lib::wallet::KeyPair { public_key: pk_bytes, secret_key };

        // Derive wallet (V4R2) address from StateInit
        let wallet = ton_lib::wallet::TonWallet::new(
            ton_lib::wallet::WalletVersion::V4R2,
            ton_keypair,
        )
        .map_err(|e| TonError::TonCoreError(e.to_string()))?;

        let address = wallet.address;
        // Use unbounceable, URL-safe mainnet friendly address (prefix "UQ...")
        let friendly = address.to_base64(true, false, true);
        let raw = address.to_hex();

        Ok(WalletIdentity {
            workchain: address.workchain as i8,
            raw_hex: raw,
            friendly,
        })
    }

    pub fn sign_data(
        &self,
        text: String,
        keypair: &Keypair,
    ) -> Result<String, TonError> {
        let mut message_bytes = Vec::new();

        message_bytes.extend_from_slice(text.as_bytes());

        let sk_bytes = BASE64.decode(keypair.sk.as_bytes()).map_err(|e| {
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

        let sk: [u8; 32] = sk_bytes.as_slice().try_into().map_err(|_| {
            TonError::SerializationError(
                "Invalid private key length".to_string(),
            )
        })?;
        let sk = SigningKey::from_bytes(&sk);

        let signature = sk.sign(&message_bytes);
        let signature_base64 = BASE64.encode(signature.to_bytes().as_ref());

        Ok(signature_base64)
    }

    /// Returns the StateInit BOC (base64-encoded) for the wallet derived
    /// from the given keypair. This is needed for WalletConnect session
    /// approval with TON namespaces.
    pub fn get_state_init_boc(
        &self,
        keypair: &Keypair,
    ) -> Result<String, TonError> {
        use ton_lib::{
            block_tlb::StateInit,
            ton_lib_core::traits::tlb::TLB,
            wallet::{WALLET_DEFAULT_ID, WalletVersion},
        };

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

        let sk_bytes = BASE64.decode(keypair.sk.as_bytes()).map_err(|e| {
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

        // Build ton-lib wallet keypair (secret_key must be 64 bytes: sk||pk)
        let mut secret_key = Vec::with_capacity(64);
        secret_key.extend_from_slice(&sk_bytes);
        secret_key.extend_from_slice(&pk_bytes);
        let ton_keypair =
            ton_lib::wallet::KeyPair { public_key: pk_bytes, secret_key };

        // Get wallet code and data for V4R2
        let version = WalletVersion::V4R2;
        let code = WalletVersion::get_code(version)
            .map_err(|e| TonError::TonCoreError(e.to_string()))?
            .clone();
        let data = WalletVersion::get_default_data(
            version,
            &ton_keypair,
            WALLET_DEFAULT_ID,
        )
        .map_err(|e| TonError::TonCoreError(e.to_string()))?;

        // Construct StateInit and serialize to BOC
        let state_init = StateInit::new(code, data);
        let boc = state_init.to_boc().map_err(|e| {
            TonError::TonCoreError(format!(
                "Failed to serialize StateInit: {}",
                e
            ))
        })?;

        Ok(BASE64.encode(&boc))
    }

    /// Returns session properties needed for TON WalletConnect approval.
    /// Includes both the hex-encoded public key and base64-encoded StateInit.
    pub fn get_session_properties(
        &self,
        keypair: &Keypair,
    ) -> Result<TonSessionProperties, TonError> {
        let state_init = self.get_state_init_boc(keypair)?;
        Ok(TonSessionProperties { public_key: keypair.pk.clone(), state_init })
    }

    pub async fn send_message(
        &self,
        network: String,
        from: String,
        keypair: &Keypair,
        valid_until: u32,
        messages: Vec<SendTxMessage>,
    ) -> Result<String, TonError> {
        // Validate network matches client (normalize both to CAIP-2 for comparison)
        let normalized_request = network::ton::normalize_chain_id(&network);
        let normalized_client =
            network::ton::normalize_chain_id(&self.cfg.network_id);
        if normalized_request != normalized_client {
            return Err(TonError::NetworkMismatch(format!(
                "client={} request={}",
                normalized_client, normalized_request
            )));
        }

        // Validate that the from address matches the keypair (normalize formats)
        let keypair_address = self.get_address_from_keypair(keypair)?;
        let from_addr = from.parse::<TonAddress>().map_err(|e| {
            TonError::InvalidAddress(format!("Invalid from address: {}", e))
        })?;
        let keypair_addr =
            keypair_address.raw_hex.parse::<TonAddress>().map_err(|e| {
                TonError::InvalidAddress(format!(
                    "Invalid keypair-derived address: {}",
                    e
                ))
            })?;
        if from_addr != keypair_addr {
            return Err(TonError::InvalidAddress(format!(
                "From address {} does not match keypair address {}",
                from, keypair_address.friendly
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

        // For each message, validate the address is parseable and amount is numeric
        for msg in &messages {
            if let Err(e) = msg.address.parse::<TonAddress>() {
                return Err(TonError::InvalidAddress(format!(
                    "Invalid TON address: {}",
                    e
                )));
            }

            if msg.amount.parse::<u128>().is_err() {
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
        _from: String,
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
        let sk = BASE64.decode(keypair.sk.as_bytes()).map_err(|e| {
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

        // Prefer getWalletInformation for seqno; fallback to getAddressInformation
        let wallet_addr_friendly = wallet.address.to_base64(true, false, true);
        let wallet_info = ton_provider
            .get_wallet_information(&wallet_addr_friendly)
            .await
            .unwrap_or_else(|_| serde_json::json!({}));

        let addr_info = if wallet_info.is_object()
            && !wallet_info.as_object().unwrap().is_empty()
        {
            wallet_info
        } else {
            ton_provider
                .get_address_information(&wallet_addr_friendly)
                .await
                .map_err(|e| {
                    TonError::TonCoreError(format!(
                        "Failed getAddressInformation: {}",
                        e
                    ))
                })?
        };

        tracing::info!("addr_info: {addr_info:?}");

        // Some backends return a top-level object, others nest under result
        let root = addr_info.get("result").unwrap_or(&addr_info);

        // If account state is explicitly uninitialized, use seqno=0
        let is_uninitialized = root
            .get("state")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case("uninitialized"))
            .or_else(|| {
                root.get("account_state")
                    .and_then(|v| v.as_str())
                    .map(|s| s.eq_ignore_ascii_case("uninitialized"))
            })
            .unwrap_or(false);

        // Prefer seqno directly if provided by wallet_information
        let seqno_direct =
            root.get("seqno").and_then(|v| v.as_u64()).or_else(|| {
                root.get("seqno")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u64>().ok())
            });

        let seqno_u64 = seqno_direct.or_else(|| {
            root.get("block_id")
                .and_then(|b| b.get("seqno"))
                .and_then(|v| v.as_u64())
                .or_else(|| {
                    root.get("block_id")
                        .and_then(|b| b.get("seqno"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<u64>().ok())
                })
                .or_else(|| {
                    root.get("last_block_id")
                        .and_then(|b| b.get("seqno"))
                        .and_then(|v| v.as_u64())
                })
                .or_else(|| {
                    root.get("last_block_id")
                        .and_then(|b| b.get("seqno"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<u64>().ok())
                })
                .or_else(|| {
                    root.get("blockId")
                        .and_then(|b| b.get("seqno"))
                        .and_then(|v| v.as_u64())
                })
                .or_else(|| {
                    root.get("blockId")
                        .and_then(|b| b.get("seqno"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<u64>().ok())
                })
                .or_else(|| {
                    root.get("lastBlockId")
                        .and_then(|b| b.get("seqno"))
                        .and_then(|v| v.as_u64())
                })
                .or_else(|| {
                    root.get("lastBlockId")
                        .and_then(|b| b.get("seqno"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<u64>().ok())
                })
                .or_else(|| root.get("seqno").and_then(|v| v.as_u64()))
                .or_else(|| {
                    root.get("seqno")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<u64>().ok())
                })
        });

        // Seqno: if uninitialized default to 0 (deploy path adds state_init),
        // otherwise require seqno from API
        let seqno = if is_uninitialized {
            0
        } else {
            seqno_u64.ok_or_else(|| TonError::TonCoreError(
                "Missing seqno (getWalletInformation/getAddressInformation)".into(),
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

        // If account is uninitialized, we must add state init so the wallet can be deployed
        let add_state_init = is_uninitialized;
        let ext_in_msg = wallet
            .create_ext_in_msg(
                vec![int_msg.to_cell_ref()?],
                seqno,
                valid_until,
                add_state_init,
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
        let boc_base64 = BASE64.encode(&boc);

        let response =
            ton_provider.send_boc(boc_base64.clone()).await.map_err(|e| {
                TonError::TonCoreError(format!("Failed to send BOC: {}", e))
            })?;

        // Extract result from response
        if let Some(error) = response.get("error") {
            return Err(TonError::TonCoreError(format!(
                "Blockchain API error: {}",
                error
            )));
        }

        // Return the BOC base64 used for sending (requested by client)
        Ok(boc_base64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_client() -> TonClient {
        TonClient::new(
            TonClientConfig { network_id: "ton:-239".to_string() },
            "test-project".into(),
            crate::pulse::get_pulse_metadata(),
        )
    }

    #[test]
    fn test_generate_keypair_from_ton_mnemonic_valid() {
        // Test with known TON mnemonic
        let client = create_test_client();
        let mnemonic = "dose ice enrich trigger test dove century still betray gas diet dune use other base gym mad law immense village world example praise game".to_string();

        let keypair =
            client.generate_keypair_from_ton_mnemonic(mnemonic).unwrap();

        assert!(!keypair.sk.is_empty());
        assert!(!keypair.pk.is_empty());

        // Verify the keypair can be used to generate an address
        let address_result = client.get_address_from_keypair(&keypair);
        assert!(address_result.is_ok());
    }

    #[test]
    fn test_generate_keypair_from_ton_mnemonic_invalid() {
        // Test error handling
        let client = create_test_client();

        // Test with invalid mnemonic (too short)
        let invalid_mnemonic = "abandon abandon abandon".to_string();
        let result =
            client.generate_keypair_from_ton_mnemonic(invalid_mnemonic);
        assert!(result.is_err());

        // Test with empty mnemonic
        let empty_mnemonic = "".to_string();
        let result = client.generate_keypair_from_ton_mnemonic(empty_mnemonic);
        assert!(result.is_err());

        // Test with invalid words
        let invalid_words = "invalid invalid invalid invalid invalid invalid invalid invalid invalid invalid invalid invalid".to_string();
        let result = client.generate_keypair_from_ton_mnemonic(invalid_words);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_keypair_from_bip39_mnemonic_valid() {
        // Test with known BIP39 mnemonic
        let client = create_test_client();
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        let result = client.generate_keypair_from_bip39_mnemonic(mnemonic);
        assert!(result.is_ok());

        let keypair = result.unwrap();
        assert!(!keypair.sk.is_empty());
        assert!(!keypair.pk.is_empty());

        // Verify the keypair can be used to generate an address
        let address_result = client.get_address_from_keypair(&keypair);
        assert!(address_result.is_ok());
    }

    #[test]
    fn test_generate_keypair_from_bip39_mnemonic_invalid() {
        // Test error handling
        let client = create_test_client();

        // Test with invalid mnemonic (too short)
        let invalid_mnemonic = "abandon abandon abandon";
        let result =
            client.generate_keypair_from_bip39_mnemonic(invalid_mnemonic);
        assert!(result.is_err());

        // Test with empty mnemonic
        let empty_mnemonic = "";
        let result =
            client.generate_keypair_from_bip39_mnemonic(empty_mnemonic);
        assert!(result.is_err());

        // Test with invalid words
        let invalid_words = "invalid invalid invalid invalid invalid invalid invalid invalid invalid invalid invalid invalid";
        let result = client.generate_keypair_from_bip39_mnemonic(invalid_words);
        assert!(result.is_err());
    }

    #[test]
    fn test_unbounceable_address_format_for_known_pubkey() {
        // Public key from repro example
        let pk_hex =
            "a323642d9cd5e4631368be4f3b15017427e4d1d15d97723a103f1c29609b7c14";
        let pk_bytes = hex::decode(pk_hex).expect("valid hex");
        let mut pk_array = [0u8; 32];
        pk_array.copy_from_slice(&pk_bytes);

        let address = TonAddress::new(
            0,
            ton_lib::ton_lib_core::cell::TonHash::from(pk_array),
        );

        // Unbounceable, URL-safe, mainnet
        let friendly_unbounceable = address.to_base64(true, false, true);
        assert_eq!(
            friendly_unbounceable,
            "UQCjI2QtnNXkYxNovk87FQF0J-TR0V2XcjoQPxwpYJt8FOUF"
        );

        // Bounceable (for contrast), URL-safe, mainnet
        let friendly_bounceable = address.to_base64(true, true, true);
        assert_eq!(
            friendly_bounceable,
            "EQCjI2QtnNXkYxNovk87FQF0J-TR0V2XcjoQPxwpYJt8FLjA"
        );
    }

    #[test]
    fn test_get_state_init_boc() {
        let client = create_test_client();
        let mnemonic = "test test test test test test test test test test \
            test junk";

        let keypair =
            client.generate_keypair_from_bip39_mnemonic(mnemonic).unwrap();
        let state_init_boc = client.get_state_init_boc(&keypair).unwrap();

        // Verify it's valid base64
        assert!(!state_init_boc.is_empty());
        assert!(BASE64.decode(state_init_boc.as_bytes()).is_ok());
    }

    #[test]
    fn test_get_session_properties() {
        let client = create_test_client();
        let mnemonic = "test test test test test test test test test test \
            test junk";

        let keypair =
            client.generate_keypair_from_bip39_mnemonic(mnemonic).unwrap();
        let props = client.get_session_properties(&keypair).unwrap();

        // Verify public_key matches keypair
        assert_eq!(props.public_key, keypair.pk);

        // Verify state_init is valid base64
        assert!(!props.state_init.is_empty());
        assert!(BASE64.decode(props.state_init.as_bytes()).is_ok());
    }

    #[test]
    fn test_session_properties_deterministic() {
        let client = create_test_client();
        let mnemonic = "test test test test test test test test test test \
            test junk";

        let keypair =
            client.generate_keypair_from_bip39_mnemonic(mnemonic).unwrap();

        // Get properties twice
        let props1 = client.get_session_properties(&keypair).unwrap();
        let props2 = client.get_session_properties(&keypair).unwrap();

        // Should be deterministic
        assert_eq!(props1.public_key, props2.public_key);
        assert_eq!(props1.state_init, props2.state_init);
    }
}
