use {
    crate::{
        sign::{
            client_types::{Session, TransportType},
            protocol_types::ProtocolRpcId,
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
    fn delete_session(&self, topic: Topic) -> Result<(), StorageError>;
    fn get_session(
        &self,
        topic: Topic,
    ) -> Result<Option<SessionFfi>, StorageError>;
    fn get_all_sessions(&self) -> Result<Vec<SessionFfi>, StorageError>;
    fn get_all_topics(&self) -> Result<Vec<Topic>, StorageError>;
    fn get_decryption_key_for_topic(
        &self,
        topic: Topic,
    ) -> Result<Option<Vec<u8>>, StorageError>;
    fn save_pairing(
        &self,
        topic: Topic,
        rpc_id: ProtocolRpcId,
        sym_key: Vec<u8>,
        self_key: Vec<u8>,
        expiry: u64,
    ) -> Result<(), StorageError>;
    fn get_pairing(
        &self,
        topic: Topic,
        rpc_id: ProtocolRpcId,
    ) -> Result<Option<PairingFfi>, StorageError>;
    fn get_all_pairings(&self) -> Result<Vec<PairingFfi>, StorageError>;
    fn delete_pairing(&self, topic: Topic) -> Result<(), StorageError>;
    fn save_partial_session(
        &self,
        topic: Topic,
        sym_key: Vec<u8>,
    ) -> Result<(), StorageError>;
    fn get_verify_public_key(&self) -> Result<Option<String>, StorageError>;
    fn set_verify_public_key(&self, jwk: String) -> Result<(), StorageError>;

    // JSON-RPC History
    fn insert_json_rpc_history(
        &self,
        request_id: ProtocolRpcId,
        topic: Topic,
        method: String,
        body: String,
        transport_type: Option<TransportType>,
        insertion_timestamp: u64,
    ) -> Result<(), StorageError>;

    fn update_json_rpc_history_response(
        &self,
        request_id: ProtocolRpcId,
        response: String,
    ) -> Result<(), StorageError>;

    fn does_json_rpc_exist(
        &self,
        request_id: ProtocolRpcId,
    ) -> Result<bool, StorageError>;
    fn get_all_json_rpc_with_timestamps(
        &self,
    ) -> Result<Vec<JsonRpcHistoryEntryFfi>, StorageError>;
    fn delete_json_rpc_history_by_id(
        &self,
        request_id: ProtocolRpcId,
    ) -> Result<(), StorageError>;
}

pub struct StorageFfiProxy(pub Arc<dyn StorageFfi>);

impl Storage for StorageFfiProxy {
    fn add_session(&self, session: Session) -> Result<(), StorageError> {
        self.0.add_session(session.into())
    }

    fn delete_session(&self, topic: Topic) -> Result<(), StorageError> {
        self.0.delete_session(topic)
    }

    fn get_session(
        &self,
        topic: Topic,
    ) -> Result<Option<Session>, StorageError> {
        Ok(self.0.get_session(topic)?.map(|s| s.into()))
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
            .get_decryption_key_for_topic(topic)?
            .map(|s| s.try_into().unwrap()))
    }

    fn save_pairing(
        &self,
        topic: Topic,
        rpc_id: ProtocolRpcId,
        sym_key: [u8; 32],
        self_key: [u8; 32],
        expiry: u64,
    ) -> Result<(), StorageError> {
        self.0.save_pairing(
            topic,
            rpc_id,
            sym_key.to_vec(),
            self_key.to_vec(),
            expiry,
        )
    }

    fn get_pairing(
        &self,
        topic: Topic,
        rpc_id: ProtocolRpcId,
    ) -> Result<Option<StoragePairing>, StorageError> {
        Ok(self.0.get_pairing(topic, rpc_id)?.map(|pairing| StoragePairing {
            expiry: pairing.expiry,
            sym_key: pairing.sym_key.try_into().unwrap(),
            self_key: pairing.self_key.try_into().unwrap(),
        }))
    }

    fn get_all_pairings(
        &self,
    ) -> Result<Vec<(Topic, ProtocolRpcId, u64)>, StorageError> {
        Ok(self
            .0
            .get_all_pairings()?
            .into_iter()
            .map(|pairing| (pairing.topic, pairing.rpc_id, pairing.expiry))
            .collect())
    }

    fn delete_pairing(&self, topic: Topic) -> Result<(), StorageError> {
        self.0.delete_pairing(topic)
    }

    fn save_partial_session(
        &self,
        topic: Topic,
        sym_key: [u8; 32],
    ) -> Result<(), StorageError> {
        self.0.save_partial_session(topic, sym_key.to_vec())
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

    fn insert_json_rpc_history(
        &self,
        request_id: ProtocolRpcId,
        topic: Topic,
        method: String,
        body: String,
        transport_type: Option<TransportType>,
        insertion_timestamp: u64,
    ) -> Result<(), StorageError> {
        self.0.insert_json_rpc_history(
            request_id,
            topic,
            method,
            body,
            transport_type,
            insertion_timestamp,
        )
    }

    fn update_json_rpc_history_response(
        &self,
        request_id: ProtocolRpcId,
        response: String,
    ) -> Result<(), StorageError> {
        self.0.update_json_rpc_history_response(request_id, response)
    }

    fn does_json_rpc_exist(
        &self,
        request_id: ProtocolRpcId,
    ) -> Result<bool, StorageError> {
        self.0.does_json_rpc_exist(request_id)
    }

    fn get_all_json_rpc_with_timestamps(
        &self,
    ) -> Result<Vec<(ProtocolRpcId, Topic, u64)>, StorageError> {
        Ok(self
            .0
            .get_all_json_rpc_with_timestamps()?
            .into_iter()
            .map(|entry| {
                (entry.request_id, entry.topic, entry.insertion_timestamp)
            })
            .collect())
    }

    fn delete_json_rpc_history_by_id(
        &self,
        request_id: ProtocolRpcId,
    ) -> Result<(), StorageError> {
        self.0.delete_json_rpc_history_by_id(request_id)
    }
}

#[derive(uniffi::Record)]
pub struct PairingFfi {
    topic: Topic,
    expiry: u64,
    rpc_id: ProtocolRpcId,
    sym_key: Vec<u8>,
    self_key: Vec<u8>,
}

#[derive(uniffi::Record)]
pub struct JsonRpcHistoryEntryFfi {
    request_id: ProtocolRpcId,
    topic: Topic,
    insertion_timestamp: u64,
}

impl From<UnexpectedUniFFICallbackError> for StorageError {
    fn from(error: UnexpectedUniFFICallbackError) -> Self {
        StorageError::Runtime(format!("UnexpectedUniFFICallbackError: {error}"))
    }
}
