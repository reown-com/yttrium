//! What the platform provides: durable storage and the event listener.

use {
    crate::client::types::SignEvent,
    std::{collections::BTreeMap, sync::Mutex},
};

#[derive(Debug, Clone, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum StorageError {
    #[error("storage failed: {message}")]
    Failed { message: String },
}

/// Async key-value storage. Keys under `secret/` must go to the platform's secure store.
/// Each `set` must be durable once it returns; no transactions are required.
#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(feature = "uniffi", uniffi::export(with_foreign))]
#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    async fn get(&self, key: String) -> Result<Option<String>, StorageError>;
    async fn set(&self, key: String, value: String)
    -> Result<(), StorageError>;
    async fn remove(&self, key: String) -> Result<(), StorageError>;
    async fn keys(&self, prefix: String) -> Result<Vec<String>, StorageError>;
}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
pub trait Storage {
    async fn get(&self, key: String) -> Result<Option<String>, StorageError>;
    async fn set(&self, key: String, value: String)
    -> Result<(), StorageError>;
    async fn remove(&self, key: String) -> Result<(), StorageError>;
    async fn keys(&self, prefix: String) -> Result<Vec<String>, StorageError>;
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(feature = "uniffi", uniffi::export(with_foreign))]
pub trait Listener: Send + Sync {
    fn on_event(&self, event: SignEvent);
}

#[cfg(target_arch = "wasm32")]
pub trait Listener {
    fn on_event(&self, event: SignEvent);
}

/// In-memory storage for tests and examples.
#[derive(Default)]
pub struct MemoryStorage {
    map: Mutex<BTreeMap<String, String>>,
}

impl MemoryStorage {
    pub fn snapshot(&self) -> BTreeMap<String, String> {
        self.map.lock().expect("lock").clone()
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl Storage for MemoryStorage {
    async fn get(&self, key: String) -> Result<Option<String>, StorageError> {
        Ok(self.map.lock().expect("lock").get(&key).cloned())
    }
    async fn set(
        &self,
        key: String,
        value: String,
    ) -> Result<(), StorageError> {
        self.map.lock().expect("lock").insert(key, value);
        Ok(())
    }
    async fn remove(&self, key: String) -> Result<(), StorageError> {
        self.map.lock().expect("lock").remove(&key);
        Ok(())
    }
    async fn keys(&self, prefix: String) -> Result<Vec<String>, StorageError> {
        Ok(self
            .map
            .lock()
            .expect("lock")
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect())
    }
}
