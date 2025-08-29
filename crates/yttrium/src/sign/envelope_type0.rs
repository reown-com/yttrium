const IV_LENGTH: usize = 12;

#[derive(Debug)]
pub struct EnvelopeType0 {
    pub iv: [u8; IV_LENGTH],
    pub sb: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
#[error("Envelope type 0 error: {0}")]
pub enum EnvelopeType0Error {
    #[error("Too short")]
    EnvelopeTooShort,

    #[error("Not type 0 envelope")]
    NotType0,

    #[error("IV array conversion failed")]
    IvArrayTryIntoFailed,
}

pub fn deserialize_envelope_type0(
    envelope: &[u8],
) -> Result<EnvelopeType0, EnvelopeType0Error> {
    let type_byte =
        envelope.first().ok_or(EnvelopeType0Error::EnvelopeTooShort)?;
    if *type_byte != 0 {
        return Err(EnvelopeType0Error::NotType0);
    }
    Ok(EnvelopeType0 {
        iv: envelope
            .get(1..1 + IV_LENGTH)
            .ok_or(EnvelopeType0Error::EnvelopeTooShort)?
            .try_into()
            .map_err(|_| EnvelopeType0Error::IvArrayTryIntoFailed)?,
        sb: envelope
            .get(1 + IV_LENGTH..)
            .ok_or(EnvelopeType0Error::EnvelopeTooShort)?
            .to_vec(),
    })
}

pub fn encode_envelope_type0(envelope: &EnvelopeType0) -> Vec<u8> {
    let mut result = vec![0];
    result.extend_from_slice(&envelope.iv);
    result.extend_from_slice(&envelope.sb);
    result
}

    /// Decrypt a type-0 envelope and return the decrypted bytes.
    pub fn decrypt_type0_envelope(
        sym_key: [u8; 32],
        message_b64: &str,
    ) -> Result<Vec<u8>, PairError> {
        let decoded = BASE64
            .decode(message_b64.as_bytes())
            .map_err(|e| PairError::Internal(format!(
                "Failed to decode message: {e}"
            )))?;

        let envelope = envelope_type0::deserialize_envelope_type0(&decoded)
            .map_err(|e| PairError::Internal(e.to_string()))?;

        let key = ChaCha20Poly1305::new(&sym_key.into());
        let decrypted = key
            .decrypt(&Nonce::from(envelope.iv), envelope.sb.as_slice())
            .map_err(|e| PairError::Internal(e.to_string()))?;
        Ok(decrypted)
    }
    /// Decode a type-0 envelope proposal message coming from IRN and return the parsed JSON-RPC request
    /// (method: wc_sessionPropose).
    pub fn decode_type0_encrypted_proposal_message(
        sym_key: [u8; 32],
        message_b64: &str,
    ) -> Result<ProposalJsonRpc, PairError> {
        let decrypted = Self::decrypt_type0_envelope(sym_key, message_b64)?;

        let request = serde_json::from_slice::<ProposalJsonRpc>(&decrypted)
            .map_err(|e| {
                PairError::Internal(format!(
                    "Failed to parse decrypted message: {e}"
                ))
            })?;
        Ok(request)
    }
