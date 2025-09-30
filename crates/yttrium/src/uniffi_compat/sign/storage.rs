use {
    crate::{
        sign::{
            client_types::{Session, TransportType},
            storage::{Storage, StorageError, StoragePairing},
        },
        uniffi_compat::sign::ffi_types::SessionFfi,
    },
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

    // JSON-RPC History
    fn insert_json_rpc_history(
        &self,
        request_id: u64,
        topic: String,
        method: String,
        body: String,
        transport_type: Option<TransportType>,
    ) -> Result<(), StorageError>;

    fn update_json_rpc_history_response(
        &self,
        request_id: u64,
        response: String,
    ) -> Result<(), StorageError>;

    fn delete_json_rpc_history_by_topic(
        &self,
        topic: String,
    ) -> Result<(), StorageError>;

    fn does_json_rpc_exist(
        &self,
        request_id: u64,
    ) -> Result<bool, StorageError>;
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

    fn insert_json_rpc_history(
        &self,
        request_id: u64,
        topic: String,
        method: String,
        body: String,
        transport_type: Option<TransportType>,
    ) -> Result<(), StorageError> {
        self.0.insert_json_rpc_history(
            request_id,
            topic,
            method,
            body,
            transport_type,
        )
    }

    fn update_json_rpc_history_response(
        &self,
        request_id: u64,
        response: String,
    ) -> Result<(), StorageError> {
        self.0.update_json_rpc_history_response(request_id, response)
    }

    fn delete_json_rpc_history_by_topic(
        &self,
        topic: String,
    ) -> Result<(), StorageError> {
        self.0.delete_json_rpc_history_by_topic(topic)
    }

    fn does_json_rpc_exist(
        &self,
        request_id: u64,
    ) -> Result<bool, StorageError> {
        self.0.does_json_rpc_exist(request_id)
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
