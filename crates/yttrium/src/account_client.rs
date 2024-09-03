use crate::config::Config;
use crate::sign_service::SignService;
use crate::transaction::{send::send_transaction, Transaction};
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

    pub fn chain_id(&self) -> i64 {
        self.chain_id
    }

    pub async fn get_address(&self) -> eyre::Result<String> {
        todo!("Implement get_address")
    }

    pub async fn sign_message(&self, message: String) -> eyre::Result<String> {
        todo!("Implement sign_message: {}", message)
    }

    pub async fn send_batch_transaction(
        &self,
        batch: Vec<Transaction>,
    ) -> eyre::Result<String> {
        todo!("Implement send_batch_transaction: {:?}", batch)
    }

    pub async fn send_transaction(
        &self,
        transaction: Transaction,
    ) -> eyre::Result<String> {
        send_transaction(self.sign_service.clone(), transaction).await
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
