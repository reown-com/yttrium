use {
    crate::sign::storage::{Storage, StorageError},
    jsonwebtoken::{jwk::Jwk, Algorithm, DecodingKey, Validation},
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    url::Url,
};

const PUBLIC_KEY_URL: &str = "https://verify.walletconnect.org/v3/public-key";
const PUBLIC_KEY: &str = include_str!("verify-public.jwk");

#[derive(Debug, thiserror::Error)]
pub enum GetPublicKeyError {
    #[error("get key from storage: {0}")]
    GetFromStorage(StorageError),

    #[error("network: {0}")]
    Network(reqwest::Error),

    #[error("not success: {0}")]
    NotSuccess(reqwest::StatusCode),

    #[error("json: {0}")]
    Json(reqwest::Error),

    #[error("recv: {0}")]
    Recv(tokio::sync::oneshot::error::RecvError),

    #[error("set key to storage: {0}")]
    SetToStorage(StorageError),
}

pub async fn get_optimistic_public_key(
    http_client: reqwest::Client,
    storage: Arc<dyn Storage>,
) -> Result<Jwk, GetPublicKeyError> {
    get_optimistic_public_key_impl(http_client, storage, PUBLIC_KEY).await
}

pub async fn get_optimistic_public_key_impl(
    http_client: reqwest::Client,
    storage: Arc<dyn Storage>,
    hardcoded_public_key: &str,
) -> Result<Jwk, GetPublicKeyError> {
    let public_key = storage
        .get_verify_public_key()
        .map_err(GetPublicKeyError::GetFromStorage)?;
    if let Some(public_key) = public_key {
        Ok(public_key)
    } else {
        match serde_json::from_str(hardcoded_public_key) {
            Ok(public_key) => Ok(public_key),
            Err(e) => {
                tracing::error!("verify parse hardcoded public key: {e}");
                get_latest_public_key(http_client, storage).await
            }
        }
    }
}

pub async fn get_latest_public_key(
    http_client: reqwest::Client,
    storage: Arc<dyn Storage>,
) -> Result<Jwk, GetPublicKeyError> {
    // spawn() to support WASM environments where the `send()` future is not Send
    // TODO consider removing when compiling for native platforms
    let (tx, rx) = tokio::sync::oneshot::channel();
    crate::spawn::spawn(async move {
        let result = async {
            let response = http_client
                .get(PUBLIC_KEY_URL)
                .send()
                .await
                .map_err(GetPublicKeyError::Network)?;
            if !response.status().is_success() {
                return Err(GetPublicKeyError::NotSuccess(response.status()));
            }
            let public_key = response
                .json::<VerifyPublicKey>()
                .await
                .map_err(GetPublicKeyError::Json)?;
            Ok(public_key.public_key)
        }
        .await;
        let _ = tx.send(result);
    });
    let public_key = rx.await.map_err(GetPublicKeyError::Recv)??;
    storage
        .set_verify_public_key(public_key.clone())
        .map_err(GetPublicKeyError::SetToStorage)?;
    Ok(public_key)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VerifyPublicKey {
    pub public_key: Jwk,
    pub expires_at: u64,
}

pub fn decode_attestation_into_verify_context(
    app_metadata_url: &str,
    attestation: &str,
    public_key: &Jwk,
    encrypted_id: &str,
) -> Result<VerifyContext, jsonwebtoken::errors::Error> {
    let attestation = match jsonwebtoken::decode::<Attestation>(
        attestation,
        &DecodingKey::from_jwk(public_key)?,
        &Validation::new(Algorithm::ES256),
    ) {
        Ok(token_data) => token_data.claims,
        Err(e) => {
            tracing::error!("verify decode attestation: {e}");
            return Ok(VerifyContext {
                origin: None,
                validation: VerifyValidation::Unknown,
                is_scam: false,
            });
        }
    };

    let app_origin = match Url::parse(app_metadata_url) {
        Ok(url) => url.origin().ascii_serialization(),
        Err(e) => {
            tracing::error!("verify parse app metadata url: {e}");
            return Ok(VerifyContext {
                origin: None,
                validation: VerifyValidation::Unknown,
                is_scam: attestation.is_scam,
            });
        }
    };

    if attestation.id != encrypted_id {
        return Ok(VerifyContext {
            origin: None,
            validation: VerifyValidation::Unknown,
            is_scam: attestation.is_scam,
        });
    }

    if !attestation.is_verified {
        return Ok(VerifyContext {
            origin: None,
            validation: VerifyValidation::Unknown,
            is_scam: attestation.is_scam,
        });
    }

    Ok(VerifyContext {
        validation: if attestation.origin == app_origin {
            VerifyValidation::Valid
        } else {
            VerifyValidation::Invalid
        },
        origin: Some(attestation.origin),
        is_scam: attestation.is_scam,
    })
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attestation {
    pub exp: u64,
    pub id: String,
    pub origin: String,
    pub is_scam: bool,
    pub is_verified: bool,
}

#[derive(Clone, Debug)]
pub struct VerifyContext {
    pub origin: Option<String>,
    pub validation: VerifyValidation,
    pub is_scam: bool,
}

#[derive(Clone, Debug)]
pub enum VerifyValidation {
    Unknown,
    Valid,
    Invalid,
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::sign::{
            client_types::{Session, TransportType},
            storage::StoragePairing,
        },
        relay_rpc::domain::Topic,
    };

    #[tokio::test]
    async fn test_get_optimistic_cached_public_key() {
        const MOCK_JWK: &str = r#"{"crv":"P-256","ext":true,"key_ops":["verify"],"kty":"EC","x":"CbL4DOYOb1ntd-8OmExO-oS0DWCMC00DntrymJoB8tk","y":"KTFwjHtQxGTDR91VsOypcdBfvbo6sAMj5p4Wb-9hRA1"}"#;
        struct MockStorage;
        impl Storage for MockStorage {
            fn get_verify_public_key(
                &self,
            ) -> Result<Option<Jwk>, StorageError> {
                Ok(Some(serde_json::from_str(MOCK_JWK).unwrap()))
            }
            fn set_verify_public_key(
                &self,
                _jwk: Jwk,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn add_session(
                &self,
                _session: Session,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn delete_session(
                &self,
                _topic: Topic,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn get_session(
                &self,
                _topic: Topic,
            ) -> Result<Option<Session>, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn get_all_sessions(&self) -> Result<Vec<Session>, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn get_all_topics(&self) -> Result<Vec<Topic>, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn get_decryption_key_for_topic(
                &self,
                _topic: Topic,
            ) -> Result<Option<[u8; 32]>, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn save_pairing(
                &self,
                _topic: Topic,
                _rpc_id: u64,
                _sym_key: [u8; 32],
                _self_key: [u8; 32],
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn get_pairing(
                &self,
                _topic: Topic,
                _rpc_id: u64,
            ) -> Result<Option<StoragePairing>, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn save_partial_session(
                &self,
                _topic: Topic,
                _sym_key: [u8; 32],
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn insert_json_rpc_history(
                &self,
                _request_id: u64,
                _topic: String,
                _method: String,
                _body: String,
                _transport_type: Option<TransportType>,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn update_json_rpc_history_response(
                &self,
                _request_id: u64,
                _response: String,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn delete_json_rpc_history_by_topic(
                &self,
                _topic: String,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn does_json_rpc_exist(
                &self,
                _request_id: u64,
            ) -> Result<bool, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
        }
        let http_client = reqwest::Client::new();
        let storage = Arc::new(MockStorage);
        let public_key =
            get_optimistic_public_key_impl(http_client, storage, MOCK_JWK)
                .await;
        let public_key = public_key.unwrap();
        assert_eq!(public_key, serde_json::from_str(MOCK_JWK).unwrap());
        assert_ne!(public_key, serde_json::from_str(PUBLIC_KEY).unwrap());
    }

    #[tokio::test]
    async fn test_get_optimistic_hardcoded_public_key() {
        struct MockStorage;
        impl Storage for MockStorage {
            fn get_verify_public_key(
                &self,
            ) -> Result<Option<Jwk>, StorageError> {
                Ok(None)
            }
            fn set_verify_public_key(
                &self,
                _jwk: Jwk,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn add_session(
                &self,
                _session: Session,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn delete_session(
                &self,
                _topic: Topic,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn get_session(
                &self,
                _topic: Topic,
            ) -> Result<Option<Session>, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn get_all_sessions(&self) -> Result<Vec<Session>, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn get_all_topics(&self) -> Result<Vec<Topic>, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn get_decryption_key_for_topic(
                &self,
                _topic: Topic,
            ) -> Result<Option<[u8; 32]>, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn save_pairing(
                &self,
                _topic: Topic,
                _rpc_id: u64,
                _sym_key: [u8; 32],
                _self_key: [u8; 32],
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn get_pairing(
                &self,
                _topic: Topic,
                _rpc_id: u64,
            ) -> Result<Option<StoragePairing>, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn save_partial_session(
                &self,
                _topic: Topic,
                _sym_key: [u8; 32],
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn insert_json_rpc_history(
                &self,
                _request_id: u64,
                _topic: String,
                _method: String,
                _body: String,
                _transport_type: Option<TransportType>,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn update_json_rpc_history_response(
                &self,
                _request_id: u64,
                _response: String,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn delete_json_rpc_history_by_topic(
                &self,
                _topic: String,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn does_json_rpc_exist(
                &self,
                _request_id: u64,
            ) -> Result<bool, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
        }
        let mock_jwk = r#"{"crv":"P-256","ext":true,"key_ops":["verify"],"kty":"EC","x":"CbL4DOYOb1ntd-8OmExO-oS0DWCMC00DntrymJoB8tk","y":"KTFwjHtQxGTDR91VsOypcdBfvbo6sAMj5p4Wb-9hRA1"}"#;
        let http_client = reqwest::Client::new();
        let storage = Arc::new(MockStorage);
        let public_key =
            get_optimistic_public_key_impl(http_client, storage, mock_jwk)
                .await;
        let public_key = public_key.unwrap();
        assert_eq!(public_key, serde_json::from_str(mock_jwk).unwrap());
        assert_ne!(public_key, serde_json::from_str(PUBLIC_KEY).unwrap());
    }

    #[tokio::test]
    async fn test_get_optimistic_invalid_hardcoded_public_key() {
        const MOCK_JWK: &str = r#"{"crv":"P-256","ext":true,"key_ops":["verify"],"kty":"EC","x":"CbL4DOYOb1ntd-8OmExO-oS0DWCMC00DntrymJoB8tk","y":"KTFwjHtQxGTDR91VsOypcdBfvbo6sAMj5p4Wb-9hRA1"}"#;
        const INVALID_JWK: &str = r#"{"crv":"P-256","ext":true,"key_ops":["verify"],"kty":"EC","x ":"CbL4DOYOb1ntd-8OmExO-oS0DWCMC00DntrymJoB8tk","y":"KTFwjHtQxGTDR91VsOypcdBfvbo6sAMj5p4Wb-9hRA1"}"#;
        struct MockStorage;
        impl Storage for MockStorage {
            fn get_verify_public_key(
                &self,
            ) -> Result<Option<Jwk>, StorageError> {
                Ok(Some(serde_json::from_str(MOCK_JWK).unwrap()))
            }
            fn set_verify_public_key(
                &self,
                _jwk: Jwk,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn add_session(
                &self,
                _session: Session,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn delete_session(
                &self,
                _topic: Topic,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn get_session(
                &self,
                _topic: Topic,
            ) -> Result<Option<Session>, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn get_all_sessions(&self) -> Result<Vec<Session>, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn get_all_topics(&self) -> Result<Vec<Topic>, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn get_decryption_key_for_topic(
                &self,
                _topic: Topic,
            ) -> Result<Option<[u8; 32]>, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn save_pairing(
                &self,
                _topic: Topic,
                _rpc_id: u64,
                _sym_key: [u8; 32],
                _self_key: [u8; 32],
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn get_pairing(
                &self,
                _topic: Topic,
                _rpc_id: u64,
            ) -> Result<Option<StoragePairing>, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn save_partial_session(
                &self,
                _topic: Topic,
                _sym_key: [u8; 32],
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn insert_json_rpc_history(
                &self,
                _request_id: u64,
                _topic: String,
                _method: String,
                _body: String,
                _transport_type: Option<TransportType>,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn update_json_rpc_history_response(
                &self,
                _request_id: u64,
                _response: String,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn delete_json_rpc_history_by_topic(
                &self,
                _topic: String,
            ) -> Result<(), StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
            fn does_json_rpc_exist(
                &self,
                _request_id: u64,
            ) -> Result<bool, StorageError> {
                Err(StorageError::Runtime("unimplemented".to_string()))
            }
        }
        let http_client = reqwest::Client::new();
        let storage = Arc::new(MockStorage);
        let public_key =
            get_optimistic_public_key_impl(http_client, storage, INVALID_JWK)
                .await;
        let public_key = serde_json::to_string(&public_key.unwrap()).unwrap();
        assert_ne!(public_key, INVALID_JWK);
        assert_ne!(public_key, PUBLIC_KEY);
    }
}
