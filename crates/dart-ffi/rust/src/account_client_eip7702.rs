use super::dart_ffi;
use dart_ffi::FFIAccountClientConfig;
use dart_ffi::FFIError;
use sign_service::SignService;
use yttrium::eip7702::account_client::AccountClient;
use yttrium::error::YttriumError;
use yttrium::sign_service;
use yttrium::sign_service::address_from_string;
use yttrium::transaction::Transaction;

pub struct FFI7702AccountClient {
    pub owner_address: String,
    pub chain_id: u64,
    account_client: yttrium::eip7702::account_client::AccountClient,
}

impl FFI7702AccountClient {
    pub fn new(config: FFIAccountClientConfig) -> Self {
        #[cfg(target_os = "ios")]
        match crate::log::init_os_logger() {
            Ok(_) => {
                log::debug!("log::debug! Logging setup successfully");
            }
            Err(err) => {
                println!("Logging setup failure e: {:?}", err.to_string());
            }
        }

        let owner_address = config.owner_address.clone();
        let chain_id = config.chain_id;
        let signer_id = format!("{}-{}", owner_address, chain_id);

        let sign_fn = Box::new(move |message: String| {
            let signer_service = dart_ffi::NativeSignerFFI::new(signer_id.clone());
            let sign = signer_service.sign(message);
            match sign {
                dart_ffi::FFIStringResult::Ok(signed_message) => Ok(signed_message),
                dart_ffi::FFIStringResult::Err(error) => {
                    Err(YttriumError { message: error })
                }
            }
        });

        let owner = address_from_string(&owner_address).unwrap();

        let signer = SignService::new(sign_fn, owner);

        let account_client = AccountClient::new(
            config.owner_address.clone(),
            config.chain_id,
            config.config.into(),
            signer,
        );

        Self {
            owner_address: config.owner_address.clone(),
            chain_id: config.chain_id,
            account_client,
        }
    }

    pub async fn send_batch_transaction(
        &self,
        batch: String,
    ) -> Result<String, FFIError> {
        let batch: Vec<dart_ffi::FFITransaction> =
            serde_json::from_str(&batch).unwrap();
        let batch_transaction: Vec<Transaction> =
            batch.into_iter().map(Into::into).collect();
        self.account_client
            .send_batch_transaction(batch_transaction)
            .await
            .map_err(|e| FFIError::Unknown(e.to_string()))
    }
}
