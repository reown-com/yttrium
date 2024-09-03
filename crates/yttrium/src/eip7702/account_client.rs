use crate::config::Config;
use crate::sign_service::SignService;
use crate::transaction::Transaction;
use std::sync::Arc;
use tokio::sync::Mutex;

#[allow(dead_code)]
pub struct AccountClient {
    owner: String,
    chain_id: i64,
    config: Config,
    sign_service: Arc<Mutex<SignService>>,
}

impl AccountClient {
    pub fn new(
        owner: String,
        chain_id: i64,
        config: Config,
        sign_service: SignService,
    ) -> Self {
        Self {
            owner,
            chain_id,
            config: config.clone(),
            sign_service: Arc::new(Mutex::new(sign_service)),
        }
    }

    pub async fn send_batch_transaction(
        &self,
        _batch: Vec<Transaction>,
    ) -> eyre::Result<String> {
        todo!()
    }
}

impl AccountClient {
    pub fn mock() -> Self {
        AccountClient {
            owner: "".to_string(),
            chain_id: 0,
            config: Config::local(),
            sign_service: Arc::new(Mutex::new(SignService::mock())),
        }
    }
}
