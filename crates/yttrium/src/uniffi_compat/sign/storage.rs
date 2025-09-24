use {
    crate::{
        sign::{
            client_types::Session,
            storage::{Storage, StorageError, StoragePairing},
        },
        uniffi_compat::sign::ffi_types::SessionFfi,
    },
    jsonwebtoken::jwk::Jwk,
    relay_rpc::domain::Topic,
    std::sync::Arc,
    uniffi::UnexpectedUniFFICallbackError,
};

#[uniffi::export(with_foreign)]
pub trait StorageFfi: Send + Sync {
    fn add_session(&self, session: SessionFfi) -> Result<(), StorageError>;
    fn delete_session(&self, topic: String) -> Result<(), StorageError>;
    fn get_session(
        &self,
        topic: String,
    ) -> Result<Option<SessionFfi>, StorageError>;
    fn get_all_sessions(&self) -> Result<Vec<SessionFfi>, StorageError>;
    fn get_all_topics(&self) -> Result<Vec<Topic>, StorageError>;
    fn get_decryption_key_for_topic(
        &self,
        topic: String,
    ) -> Result<Option<Vec<u8>>, StorageError>;
    fn save_pairing(
        &self,
        topic: String,
        rpc_id: u64,
        sym_key: Vec<u8>,
        self_key: Vec<u8>,
    ) -> Result<(), StorageError>;
    fn get_pairing(
        &self,
        topic: String,
        rpc_id: u64,
    ) -> Result<Option<PairingFfi>, StorageError>;
    fn save_partial_session(
        &self,
        topic: String,
        sym_key: Vec<u8>,
    ) -> Result<(), StorageError>;
    fn get_verify_public_key(&self) -> Result<Option<String>, StorageError>;
    fn set_verify_public_key(&self, jwk: String) -> Result<(), StorageError>;
}

pub struct StorageFfiProxy(pub Arc<dyn StorageFfi>);

impl Storage for StorageFfiProxy {
    fn add_session(&self, session: Session) -> Result<(), StorageError> {
        self.0.add_session(session.into())
    }

    fn delete_session(&self, topic: Topic) -> Result<(), StorageError> {
        self.0.delete_session(topic.to_string())
    }

    fn get_session(
        &self,
        topic: Topic,
    ) -> Result<Option<Session>, StorageError> {
        Ok(self.0.get_session(topic.to_string())?.map(|s| s.into()))
    }

    fn get_all_sessions(&self) -> Result<Vec<Session>, StorageError> {
        Ok(self.0.get_all_sessions()?.into_iter().map(|s| s.into()).collect())
    }

    fn get_all_topics(&self) -> Result<Vec<Topic>, StorageError> {
        self.0.get_all_topics()
    }

    fn get_decryption_key_for_topic(
        &self,
        topic: Topic,
    ) -> Result<Option<[u8; 32]>, StorageError> {
        Ok(self
            .0
            .get_decryption_key_for_topic(topic.to_string())?
            .map(|s| s.try_into().unwrap()))
    }

    fn save_pairing(
        &self,
        topic: Topic,
        rpc_id: u64,
        sym_key: [u8; 32],
        self_key: [u8; 32],
    ) -> Result<(), StorageError> {
        self.0.save_pairing(
            topic.to_string(),
            rpc_id,
            sym_key.to_vec(),
            self_key.to_vec(),
        )
    }

    fn get_pairing(
        &self,
        topic: Topic,
        rpc_id: u64,
    ) -> Result<Option<StoragePairing>, StorageError> {
        Ok(self.0.get_pairing(topic.to_string(), rpc_id)?.map(|pairing| {
            StoragePairing {
                sym_key: pairing.sym_key.try_into().unwrap(),
                self_key: pairing.self_key.try_into().unwrap(),
            }
        }))
    }

    fn save_partial_session(
        &self,
        topic: Topic,
        sym_key: [u8; 32],
    ) -> Result<(), StorageError> {
        self.0.save_partial_session(topic.to_string(), sym_key.to_vec())
    }

    fn get_verify_public_key(&self) -> Result<Option<Jwk>, StorageError> {
        let jwk = self.0.get_verify_public_key()?;
        if let Some(jwk) = jwk {
            serde_json::from_str(&jwk)
                .map_err(|e| StorageError::Runtime(e.to_string()))
        } else {
            Ok(None)
        }
    }

    fn set_verify_public_key(&self, jwk: Jwk) -> Result<(), StorageError> {
        serde_json::to_string(&jwk)
            .map_err(|e| StorageError::Runtime(e.to_string()))
            .and_then(|jwk| self.0.set_verify_public_key(jwk))
    }
}

#[derive(uniffi::Record)]
pub struct PairingFfi {
    rpc_id: u64,
    sym_key: Vec<u8>,
    self_key: Vec<u8>,
}

impl From<UnexpectedUniFFICallbackError> for StorageError {
    fn from(error: UnexpectedUniFFICallbackError) -> Self {
        StorageError::Runtime(format!("UnexpectedUniFFICallbackError: {error}"))
    }
}
