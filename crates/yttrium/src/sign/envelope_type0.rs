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
