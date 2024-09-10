use crate::error::YttriumError;
use alloy::primitives::Address;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type PrivateKeyFN =
    dyn Fn() -> Result<String, YttriumError> + Send + 'static;

pub type BoxPrivateKeyFN = Box<PrivateKeyFN>;

pub struct PrivateKeyService {
    private_key_fn: Arc<Mutex<BoxPrivateKeyFN>>,
    owner: Address,
}

impl PrivateKeyService {
    pub fn new(private_key_fn: BoxPrivateKeyFN, owner: Address) -> Self {
        PrivateKeyService {
            private_key_fn: Arc::new(Mutex::new(private_key_fn)),
            owner,
        }
    }

    pub fn owner(&self) -> Address {
        self.owner
    }

    pub fn private_key(&self) -> Result<String, YttriumError> {
        let private_key_fn = self.private_key_fn.clone();
        let private_key_fn = private_key_fn
            .try_lock()
            .map_err(|e| YttriumError { message: e.to_string() })?;
        (private_key_fn)()
    }
}
