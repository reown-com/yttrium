use {
    crate::sign::storage::{Storage, StorageError},
    jsonwebtoken::{jwk::Jwk, Algorithm, DecodingKey, Validation},
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    url::Url,
};

const PUBLIC_KEY_URL: &str = "https://verify.walletconnect.org/v3/public-key";

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

pub async fn get_public_key(
    http_client: reqwest::Client,
    storage: Arc<dyn Storage>,
) -> Result<Jwk, GetPublicKeyError> {
    let public_key = storage
        .get_verify_public_key()
        .map_err(GetPublicKeyError::GetFromStorage)?;
    if let Some(public_key) = public_key {
        Ok(public_key)
    } else {
        let (tx, rx) = tokio::sync::oneshot::channel();
        crate::spawn::spawn(async move {
            let result = async {
                let response = http_client
                    .get(PUBLIC_KEY_URL)
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
        });
        let public_key = rx.await.map_err(GetPublicKeyError::Recv)??;
        storage
            .set_verify_public_key(public_key.clone())
            .map_err(GetPublicKeyError::SetToStorage)?;
        Ok(public_key)
    }
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
