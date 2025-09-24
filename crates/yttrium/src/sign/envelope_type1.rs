#![allow(unused)]

const IV_LENGTH: usize = 12;
const PK_LENGTH: usize = 32;

#[derive(Debug)]
pub struct EnvelopeType1 {
    pub iv: [u8; IV_LENGTH],
    pub pk: [u8; PK_LENGTH],
    pub sb: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
#[error("Envelope type 1 error: {0}")]
pub enum EnvelopeType1Error {
    #[error("Too short")]
    EnvelopeTooShort,

    #[error("Not type 1 envelope")]
    NotType1,

    #[error("PK array conversion failed")]
    PkArrayTryIntoFailed,

    #[error("IV array conversion failed")]
    IvArrayTryIntoFailed,
}

pub fn deserialize_envelope_type1(
    envelope: &[u8],
) -> Result<EnvelopeType1, EnvelopeType1Error> {
    let type_byte =
        envelope.first().ok_or(EnvelopeType1Error::EnvelopeTooShort)?;
    if *type_byte != 1 {
        return Err(EnvelopeType1Error::NotType1);
    }
    Ok(EnvelopeType1 {
        pk: envelope
            .get(1..1 + PK_LENGTH)
            .ok_or(EnvelopeType1Error::EnvelopeTooShort)?
            .try_into()
            .map_err(|_| EnvelopeType1Error::PkArrayTryIntoFailed)?,
        iv: envelope
            .get(1 + PK_LENGTH..1 + PK_LENGTH + IV_LENGTH)
            .ok_or(EnvelopeType1Error::EnvelopeTooShort)?
            .try_into()
            .map_err(|_| EnvelopeType1Error::IvArrayTryIntoFailed)?,
        sb: envelope
            .get(1 + PK_LENGTH + IV_LENGTH..)
            .ok_or(EnvelopeType1Error::EnvelopeTooShort)?
            .to_vec(),
    })
}

pub fn encode_envelope_type1(
    envelope: &EnvelopeType1,
) -> Result<Vec<u8>, EnvelopeType1Error> {
    let mut result = vec![1];
    result.extend_from_slice(&envelope.pk);
    result.extend_from_slice(&envelope.iv);
    result.extend_from_slice(&envelope.sb);
    Ok(result)
}
