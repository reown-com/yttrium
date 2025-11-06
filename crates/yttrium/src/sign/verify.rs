use {
    crate::sign::{
        storage::{Storage, StorageError},
        utils::{DecryptedHash, EncryptedHash},
    },
    jsonwebtoken::{jwk::Jwk, Algorithm, DecodingKey, Validation},
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    tracing::Instrument,
    url::Url,
};

pub const VERIFY_SERVER_URL: &str = "https://verify.walletconnect.org";
const PUBLIC_KEY_ENDPOINT: &str = "/v3/public-key";
const ATTESTATION_ENDPOINT: &str = "/attestation/";
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
    verify_server_url: String,
    http_client: reqwest::Client,
    storage: Arc<dyn Storage>,
    probe_group: Option<String>,
) -> Result<Jwk, GetPublicKeyError> {
    get_optimistic_public_key_impl(
        verify_server_url,
        http_client,
        storage,
        PUBLIC_KEY,
        probe_group.clone(),
    )
    .await
}

pub async fn get_optimistic_public_key_impl(
    verify_server_url: String,
    http_client: reqwest::Client,
    storage: Arc<dyn Storage>,
    hardcoded_public_key: &str,
    probe_group: Option<String>,
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
                get_latest_public_key(
                    verify_server_url,
                    http_client,
                    storage,
                    probe_group.clone(),
                )
                .await
            }
        }
    }
}

pub async fn get_latest_public_key(
    verify_server_url: String,
    http_client: reqwest::Client,
    storage: Arc<dyn Storage>,
    probe_group: Option<String>,
) -> Result<Jwk, GetPublicKeyError> {
    // spawn() to support WASM environments where the `send()` future is not Send
    // TODO consider removing when compiling for native platforms
    let (tx, rx) = tokio::sync::oneshot::channel();
    crate::spawn::spawn(
        async move {
            let result = async {
                let response = http_client
                    .get(format!("{verify_server_url}{PUBLIC_KEY_ENDPOINT}"))
                    .send()
                    .await
                    .map_err(GetPublicKeyError::Network)?;
                if !response.status().is_success() {
                    return Err(GetPublicKeyError::NotSuccess(
                        response.status(),
                    ));
                }
                let public_key = response
                    .json::<VerifyPublicKey>()
                    .await
                    .map_err(GetPublicKeyError::Json)?;
                Ok(public_key.public_key)
            }
            .await;
            let _ = tx.send(result);
        }
        .instrument(tracing::debug_span!(
            "get_latest_public_key",
            group = probe_group.clone()
        )),
    );
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

#[allow(clippy::too_many_arguments)]
async fn decode_attestation_into_verify_context(
    verify_server_url: String,
    app_metadata_url: &str,
    attestation: &str,
    public_key: &Jwk,
    encrypted_id: EncryptedHash,
    http_client: reqwest::Client,
    storage: Arc<dyn Storage>,
    probe_group: Option<String>,
) -> VerifyContext {
    let decoding_key = match DecodingKey::from_jwk(public_key) {
        Ok(decoding_key) => decoding_key,
        Err(e) => {
            tracing::error!("decode_attestation_into_verify_context: DecodingKey::from_jwk: {e}");
            return VerifyContext {
                origin: None,
                validation: VerifyValidation::Unknown,
                is_scam: false,
            };
        }
    };
    let attestation = match jsonwebtoken::decode::<Attestation>(
        attestation,
        &decoding_key,
        &Validation::new(Algorithm::ES256),
    ) {
        Ok(token_data) => token_data.claims,
        Err(e) => {
            if e.kind() == &jsonwebtoken::errors::ErrorKind::InvalidSignature {
                tracing::debug!("decode_attestation_into_verify_context: invalid signature, fetching latest key");
                let public_key = match get_latest_public_key(
                    verify_server_url,
                    http_client,
                    storage,
                    probe_group.clone(),
                )
                .await
                {
                    Ok(public_key) => public_key,
                    Err(e) => {
                        tracing::error!("decode_attestation_into_verify_context: get_latest_public_key: {e}");
                        return VerifyContext {
                            origin: None,
                            validation: VerifyValidation::Unknown,
                            is_scam: false,
                        };
                    }
                };
                let decoding_key = match DecodingKey::from_jwk(&public_key) {
                    Ok(decoding_key) => decoding_key,
                    Err(e) => {
                        tracing::error!("decode_attestation_into_verify_context: DecodingKey::from_jwk: {e}");
                        return VerifyContext {
                            origin: None,
                            validation: VerifyValidation::Unknown,
                            is_scam: false,
                        };
                    }
                };
                match jsonwebtoken::decode::<Attestation>(
                    attestation,
                    &decoding_key,
                    &Validation::new(Algorithm::ES256),
                ) {
                    Ok(token_data) => token_data.claims,
                    Err(e) => {
                        tracing::error!("decode_attestation_into_verify_context: decode attestation (2): {e}");
                        return VerifyContext {
                            origin: None,
                            validation: VerifyValidation::Unknown,
                            is_scam: false,
                        };
                    }
                }
            } else {
                tracing::error!("decode_attestation_into_verify_context: decode attestation (not invalid signature): {e}");
                return VerifyContext {
                    origin: None,
                    validation: VerifyValidation::Unknown,
                    is_scam: false,
                };
            }
        }
    };

    let app_origin = match Url::parse(app_metadata_url) {
        Ok(url) => url.origin().ascii_serialization(),
        Err(e) => {
            tracing::error!("decode_attestation_into_verify_context: parse app metadata url: {e}");
            return VerifyContext {
                origin: None,
                validation: VerifyValidation::Unknown,
                is_scam: attestation.is_scam.unwrap_or(false),
            };
        }
    };

    if attestation.id != encrypted_id.as_str() {
        tracing::debug!(
            "decode_attestation_into_verify_context: expected attestation id: {}, got: {}",
            encrypted_id.as_str(),
            attestation.id
        );
        return VerifyContext {
            origin: None,
            validation: VerifyValidation::Unknown,
            is_scam: attestation.is_scam.unwrap_or(false),
        };
    }

    if !attestation.is_verified {
        tracing::debug!("decode_attestation_into_verify_context: attestation is not verified");
        return VerifyContext {
            origin: None,
            validation: VerifyValidation::Unknown,
            is_scam: attestation.is_scam.unwrap_or(false),
        };
    }

    VerifyContext {
        validation: if attestation.origin == app_origin {
            tracing::debug!("decode_attestation_into_verify_context: attestation origin is valid");
            VerifyValidation::Valid
        } else {
            tracing::debug!("decode_attestation_into_verify_context: attestation origin is invalid");
            VerifyValidation::Invalid
        },
        origin: Some(attestation.origin),
        is_scam: attestation.is_scam.unwrap_or(false),
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attestation {
    pub exp: u64,
    pub id: String,
    pub origin: String,
    pub is_scam: Option<bool>,
    pub is_verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VerifyAttestation {
    attestation_id: String,
    origin: String,
    is_scam: Option<bool>,
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_verify(
    verify_server_url: String,
    decrypted_hash: DecryptedHash,
    http_client: reqwest::Client,
    storage: Arc<dyn Storage>,
    attestation: Option<Arc<str>>,
    encrypted_hash: EncryptedHash,
    app_metadata_url: String,
    probe_group: Option<String>,
) -> VerifyContext {
    if let Some(attestation) = attestation {
        if attestation.is_empty() {
            // Handling deprecated path just-in-case. Mostly should be null or JWT w/ isVerified=false
            tracing::debug!("handle_verify: attestation is empty");
            return VerifyContext {
                origin: None,
                validation: VerifyValidation::Unknown,
                is_scam: false,
            };
        }

        let verify_public_key = match get_optimistic_public_key(
            verify_server_url.clone(),
            http_client.clone(),
            storage.clone(),
            probe_group.clone(),
        )
        .await
        {
            Ok(public_key) => public_key,
            Err(e) => {
                tracing::error!("get verify public key: {e}");
                return VerifyContext {
                    origin: None,
                    validation: VerifyValidation::Unknown,
                    is_scam: false,
                };
            }
        };
        decode_attestation_into_verify_context(
            verify_server_url,
            &app_metadata_url,
            attestation.as_ref(),
            &verify_public_key,
            encrypted_hash.clone(),
            http_client,
            storage,
            probe_group.clone(),
        )
        .await
    } else {
        let attestation_result = {
            let url = format!(
                "{verify_server_url}{ATTESTATION_ENDPOINT}{decrypted_hash}?v2Supported=true",
                decrypted_hash = decrypted_hash.as_str()
            );
            // spawn() to support WASM environments where the `reqwest::send()` future is not Send
            // TODO consider removing when compiling for native platforms
            let (tx, rx) = tokio::sync::oneshot::channel();
            crate::spawn::spawn(
                async move {
                    let result = async {
                        let response = http_client
                            .get(url)
                            .send()
                            .await
                            .map_err(GetPublicKeyError::Network)?;
                        if !response.status().is_success() {
                            return Err(GetPublicKeyError::NotSuccess(
                                response.status(),
                            ));
                        }
                        let attestation = response
                            .json::<VerifyAttestation>()
                            .await
                            .map_err(GetPublicKeyError::Json)?;
                        Ok(attestation)
                    }
                    .await;
                    let _ = tx.send(result);
                }
                .instrument(tracing::debug_span!(
                    "get_attestation",
                    group = probe_group.clone()
                )),
            );
            rx.await.map_err(GetPublicKeyError::Recv)
        };
        let attestation = match attestation_result {
            Ok(Ok(public_key)) => public_key,
            Ok(Err(e)) => {
                tracing::error!("get attestation: {e}");
                return VerifyContext {
                    origin: None,
                    validation: VerifyValidation::Unknown,
                    is_scam: false,
                };
            }
            Err(e) => {
                tracing::error!("get attestation thread: {e}");
                return VerifyContext {
                    origin: None,
                    validation: VerifyValidation::Unknown,
                    is_scam: false,
                };
            }
        };

        let app_origin = match Url::parse(&app_metadata_url) {
            Ok(url) => url.origin().ascii_serialization(),
            Err(e) => {
                tracing::error!("verify parse app metadata url: {e}");
                return VerifyContext {
                    origin: None,
                    validation: VerifyValidation::Unknown,
                    is_scam: attestation.is_scam.unwrap_or(false),
                };
            }
        };

        if attestation.attestation_id != decrypted_hash.as_str() {
            tracing::debug!("handle_verify: attestation id mismatch");
            return VerifyContext {
                origin: None,
                validation: VerifyValidation::Unknown,
                is_scam: attestation.is_scam.unwrap_or(false),
            };
        }

        VerifyContext {
            validation: if attestation.origin == app_origin {
                tracing::debug!("handle_verify: attestation origin is valid");
                VerifyValidation::Valid
            } else {
                tracing::debug!("handle_verify: attestation origin is invalid");
                VerifyValidation::Invalid
            },
            origin: Some(attestation.origin),
            is_scam: attestation.is_scam.unwrap_or(false),
        }
    }
}

#[derive(Clone, Debug)]
pub struct VerifyContext {
    pub origin: Option<String>,
    pub validation: VerifyValidation,
    pub is_scam: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VerifyValidation {
    Unknown,
    Valid,
    Invalid,
}

#[cfg(target_arch = "wasm32")]
pub async fn create_attestation(
    encrypted_id: EncryptedHash,
    decrypted_id: DecryptedHash,
    project_id: relay_rpc::domain::ProjectId,
) -> Result<String, ()> {
    match crate::sign::verify_attestation::create_attestation(
        encrypted_id,
        decrypted_id,
        project_id,
    )
    .await
    {
        Ok(Some(attestation)) => Ok(attestation),
        Ok(None) => {
            tracing::warn!(
                "Verify V3 attestation returned None (timeout or no response)"
            );
            Ok(String::new())
        }
        Err(e) => {
            tracing::error!("Verify V3 attestation error: {e}");
            Ok(String::new())
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn create_attestation(
    _encrypted_id: EncryptedHash,
    _decrypted_id: DecryptedHash,
    _project_id: relay_rpc::domain::ProjectId,
) -> Result<String, ()> {
    tracing::debug!("Verify V3 not supported on non-WASM platforms, returning empty attestation");
    Ok(String::new())
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
        sha2::Digest,
        wiremock::{
            matchers::{method, path},
            Mock, MockServer, ResponseTemplate,
        },
    };

    #[tokio::test]
    async fn test_get_optimistic_cached_public_key() {
        let mock_server = MockServer::start().await;
        let verify_url = mock_server.uri();
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
        let public_key = get_optimistic_public_key_impl(
            verify_url,
            http_client,
            storage,
            MOCK_JWK,
            None,
        )
        .await;
        let public_key = public_key.unwrap();
        assert_eq!(public_key, serde_json::from_str(MOCK_JWK).unwrap());
        assert_ne!(public_key, serde_json::from_str(PUBLIC_KEY).unwrap());
    }

    #[tokio::test]
    async fn test_get_optimistic_hardcoded_public_key() {
        let mock_server = MockServer::start().await;
        let verify_url = mock_server.uri();
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
        let public_key = get_optimistic_public_key_impl(
            verify_url,
            http_client,
            storage,
            mock_jwk,
            None,
        )
        .await;
        let public_key = public_key.unwrap();
        assert_eq!(public_key, serde_json::from_str(mock_jwk).unwrap());
        assert_ne!(public_key, serde_json::from_str(PUBLIC_KEY).unwrap());
    }

    #[tokio::test]
    async fn test_get_optimistic_invalid_hardcoded_public_key() {
        let mock_server = MockServer::start().await;
        let verify_url = mock_server.uri();
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
        let public_key = get_optimistic_public_key_impl(
            verify_url,
            http_client,
            storage,
            INVALID_JWK,
            None,
        )
        .await;
        let public_key = serde_json::to_string(&public_key.unwrap()).unwrap();
        assert_ne!(public_key, INVALID_JWK);
        assert_ne!(public_key, PUBLIC_KEY);
    }

    #[tokio::test]
    async fn test_handle_verify_empty_attestation() {
        let mock_server = MockServer::start().await;
        let verify_url = mock_server.uri();
        let http_client = reqwest::Client::new();
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
        let storage = Arc::new(MockStorage);
        let verify_context = handle_verify(
            verify_url,
            DecryptedHash(hex::encode(sha2::Sha256::digest([0; 32]))),
            http_client,
            storage,
            Some("".to_string().into()),
            EncryptedHash(hex::encode(sha2::Sha256::digest([1; 32]))),
            "https://app.walletconnect.org".to_string(),
            None,
        )
        .await;

        // Test that empty attestation returns Unknown validation
        assert!(verify_context.origin.is_none());
        assert!(matches!(verify_context.validation, VerifyValidation::Unknown));
        assert!(!verify_context.is_scam);
    }

    // Helper function to create a simple mock storage that returns the embedded public key
    fn create_mock_storage_with_public_key() -> Arc<impl Storage> {
        const MOCK_JWK: &str = r#"{"crv":"P-256","ext":true,"key_ops":["verify"],"kty":"EC","x":"CbL4DOYOb1ntd-8OmExO-oS0DWCMC00DntrymJoB8tk","y":"KTFwjHtQxGTDR91VsOypcdBfvbo6sAMj5p4Wb-9hRA1"}"#;

        struct MockStorageWithKey;
        impl Storage for MockStorageWithKey {
            fn get_verify_public_key(
                &self,
            ) -> Result<Option<Jwk>, StorageError> {
                Ok(Some(serde_json::from_str(MOCK_JWK).unwrap()))
            }
            fn set_verify_public_key(
                &self,
                _jwk: Jwk,
            ) -> Result<(), StorageError> {
                Ok(())
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
        Arc::new(MockStorageWithKey)
    }

    #[tokio::test]
    async fn test_handle_verify_no_attestation_valid_v2_response() {
        let mock_server = MockServer::start().await;
        let verify_url = mock_server.uri();
        let http_client = reqwest::Client::new();
        let storage = create_mock_storage_with_public_key();

        let decrypted_hash =
            DecryptedHash(hex::encode(sha2::Sha256::digest([1; 32])));
        let app_origin = "https://app.walletconnect.org";
        let attestation_origin = app_origin;

        // Mock the v2 attestation endpoint
        Mock::given(method("GET"))
            .and(path(format!(
                "{}/{}",
                ATTESTATION_ENDPOINT.trim_end_matches('/'),
                decrypted_hash.as_str()
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                VerifyAttestation {
                    attestation_id: decrypted_hash.as_str().to_string(),
                    origin: attestation_origin.to_string(),
                    is_scam: Some(false),
                },
            ))
            .mount(&mock_server)
            .await;

        let verify_context = handle_verify(
            verify_url,
            decrypted_hash,
            http_client,
            storage,
            None,
            EncryptedHash(hex::encode(sha2::Sha256::digest([0; 32]))),
            app_origin.to_string(),
            None,
        )
        .await;

        assert_eq!(verify_context.origin.as_deref(), Some(attestation_origin));
        assert!(matches!(verify_context.validation, VerifyValidation::Valid));
        assert!(!verify_context.is_scam);
    }

    #[tokio::test]
    async fn test_handle_verify_no_attestation_network_error() {
        let mock_server = MockServer::start().await;
        let verify_url = mock_server.uri();
        let http_client = reqwest::Client::new();
        let storage = create_mock_storage_with_public_key();

        let decrypted_hash =
            DecryptedHash(hex::encode(sha2::Sha256::digest([1; 32])));
        let app_origin = "https://app.walletconnect.org";

        // Mock the v2 attestation endpoint to return 500 error
        Mock::given(method("GET"))
            .and(path(format!(
                "{}/{}",
                ATTESTATION_ENDPOINT.trim_end_matches('/'),
                decrypted_hash.as_str()
            )))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let verify_context = handle_verify(
            verify_url,
            decrypted_hash,
            http_client,
            storage,
            None,
            EncryptedHash(hex::encode(sha2::Sha256::digest([0; 32]))),
            app_origin.to_string(),
            None,
        )
        .await;

        assert!(verify_context.origin.is_none());
        assert!(matches!(verify_context.validation, VerifyValidation::Unknown));
        assert!(!verify_context.is_scam);
    }

    #[tokio::test]
    async fn test_handle_verify_no_attestation_mismatched_origin() {
        let mock_server = MockServer::start().await;
        let verify_url = mock_server.uri();
        let http_client = reqwest::Client::new();
        let storage = create_mock_storage_with_public_key();

        let decrypted_hash =
            DecryptedHash(hex::encode(sha2::Sha256::digest([1; 32])));
        let app_origin = "https://app.walletconnect.org";
        let malicious_origin = "https://malicious.example.com";

        // Mock the v2 attestation endpoint with different origin
        Mock::given(method("GET"))
            .and(path(format!(
                "{}/{}",
                ATTESTATION_ENDPOINT.trim_end_matches('/'),
                decrypted_hash.as_str()
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                VerifyAttestation {
                    attestation_id: decrypted_hash.as_str().to_string(),
                    origin: malicious_origin.to_string(),
                    is_scam: Some(false),
                },
            ))
            .mount(&mock_server)
            .await;

        let verify_context = handle_verify(
            verify_url,
            decrypted_hash,
            http_client,
            storage,
            None,
            EncryptedHash(hex::encode(sha2::Sha256::digest([0; 32]))),
            app_origin.to_string(),
            None,
        )
        .await;

        assert_eq!(verify_context.origin.as_deref(), Some(malicious_origin));
        assert!(matches!(verify_context.validation, VerifyValidation::Invalid));
        assert!(!verify_context.is_scam);
    }

    #[tokio::test]
    async fn test_handle_verify_no_attestation_id_mismatch() {
        let mock_server = MockServer::start().await;
        let verify_url = mock_server.uri();
        let http_client = reqwest::Client::new();
        let storage = create_mock_storage_with_public_key();

        let decrypted_hash =
            DecryptedHash(hex::encode(sha2::Sha256::digest([1; 32])));
        let app_origin = "https://app.walletconnect.org";

        // Mock the v2 attestation endpoint with wrong ID
        Mock::given(method("GET"))
            .and(path(format!(
                "{}/{}",
                ATTESTATION_ENDPOINT.trim_end_matches('/'),
                decrypted_hash.as_str()
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                VerifyAttestation {
                    attestation_id: "wrong_id".to_string(), // Mismatched ID
                    origin: app_origin.to_string(),
                    is_scam: Some(false),
                },
            ))
            .mount(&mock_server)
            .await;

        let verify_context = handle_verify(
            verify_url,
            decrypted_hash,
            http_client,
            storage,
            None,
            EncryptedHash(hex::encode(sha2::Sha256::digest([0; 32]))),
            app_origin.to_string(),
            None,
        )
        .await;

        assert!(verify_context.origin.is_none());
        assert!(matches!(verify_context.validation, VerifyValidation::Unknown));
        assert!(!verify_context.is_scam);
    }

    #[tokio::test]
    async fn test_handle_verify_no_attestation_scam_detected() {
        let mock_server = MockServer::start().await;
        let verify_url = mock_server.uri();
        let http_client = reqwest::Client::new();
        let storage = create_mock_storage_with_public_key();

        let decrypted_hash =
            DecryptedHash(hex::encode(sha2::Sha256::digest([1; 32])));
        let app_origin = "https://app.walletconnect.org";

        // Mock the v2 attestation endpoint with scam flag
        Mock::given(method("GET"))
            .and(path(format!(
                "{}/{}",
                ATTESTATION_ENDPOINT.trim_end_matches('/'),
                decrypted_hash.as_str()
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                VerifyAttestation {
                    attestation_id: decrypted_hash.as_str().to_string(),
                    origin: app_origin.to_string(),
                    is_scam: Some(true), // Scam detected
                },
            ))
            .mount(&mock_server)
            .await;

        let verify_context = handle_verify(
            verify_url,
            decrypted_hash,
            http_client,
            storage,
            None,
            EncryptedHash(hex::encode(sha2::Sha256::digest([0; 32]))),
            app_origin.to_string(),
            None,
        )
        .await;

        assert_eq!(verify_context.origin.as_deref(), Some(app_origin));
        assert!(matches!(verify_context.validation, VerifyValidation::Valid));
        assert!(verify_context.is_scam); // Scam flag should be true
    }
}
