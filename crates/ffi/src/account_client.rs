use super::ffi;
use ffi::FFIAccountClientConfig;
use ffi::FFIError;
use sign_service::SignService;
use yttrium::account_client::AccountClient;
use yttrium::error::YttriumError;
use yttrium::sign_service;
use yttrium::sign_service::address_from_string;
use yttrium::transaction::Transaction;

pub struct FFIAccountClient {
    pub owner_address: String,
    pub chain_id: i64,
    account_client: yttrium::account_client::AccountClient,
}

impl FFIAccountClient {
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
        let chain_id = config.chain_id.clone();
        let signer_id = format!("{}-{}", owner_address, chain_id);

        let sign_fn = Box::new(move |message: String| {
            let signer_service = ffi::SignerServiceFFI::new(signer_id.clone());
            let sign = signer_service.sign(message);
            let result = match sign {
                ffi::FFIStringResult::Ok(signed_message) => Ok(signed_message),
                ffi::FFIStringResult::Err(error) => {
                    Err(YttriumError { message: error })
                }
            };
            result
        });

        let owner = address_from_string(&owner_address).unwrap();

        let signer = SignService::new(sign_fn, owner);

        let account_client = AccountClient::new(
            config.owner_address.clone(),
            config.chain_id.clone(),
            config.config.into(),
            signer,
        );

        Self {
            owner_address: config.owner_address.clone(),
            chain_id: config.chain_id,
            account_client,
        }
    }

    pub fn chain_id(&self) -> i64 {
        self.chain_id
    }

    pub async fn get_address(&self) -> Result<String, FFIError> {
        // self.account_client
        //     .get_address()
        //     .await
        //     .map_err(|e| FFIError::Unknown(e.to_string()))
        // TODO: Implement get_address
        Ok("EXPECTED_ADDRESS".to_string())
    }

    pub async fn send_transaction(
        &self,
        transaction: ffi::FFITransaction,
    ) -> Result<String, FFIError> {
        let transaction = Transaction::from(transaction);
        self.account_client
            .send_transaction(transaction)
            .await
            .map_err(|e| FFIError::Unknown(e.to_string()))
    }
}

impl From<ffi::FFITransaction> for Transaction {
    fn from(transaction: ffi::FFITransaction) -> Self {
        Self {
            to: transaction._to,
            value: transaction._value,
            data: transaction._data,
        }
    }
}
