use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

pub fn decode_attestation(
    attestation: &str,
) -> Result<(), jsonwebtoken::errors::Error> {
    jsonwebtoken::decode::<Attestation>(
        attestation,
        &DecodingKey::from_secret("secret".as_ref()),
        &Validation::new(Algorithm::EdDSA),
    )?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attestation {
    pub exp: u64,
    pub id: String,
    pub origin: String,
    pub is_scam: bool,
    pub is_verified: bool,
}
