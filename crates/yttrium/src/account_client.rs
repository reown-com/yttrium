use crate::bundler::models::user_operation_receipt::UserOperationReceipt;
use crate::bundler::{client::BundlerClient, config::BundlerConfig};
use crate::config::Config;
use crate::private_key_service::PrivateKeyService;
use crate::sign_service::SignService;
use crate::transaction::send::safe_test;
use crate::transaction::{send::send_transactions, Transaction};
use alloy::primitives::{Address, B256};
use alloy::signers::local::PrivateKeySigner;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub enum SignerType {
    PrivateKey,
    Native,
}

impl SignerType {
    pub fn from(string: String) -> eyre::Result<Self> {
        match string.as_str() {
            "PrivateKey" => Ok(SignerType::PrivateKey),
            "Native" => Ok(SignerType::Native),
            _ => Err(eyre::Report::msg("Invalid signer type")),
        }
    }
}

#[derive(Clone)]
pub enum Signer {
    PrivateKey(Arc<Mutex<PrivateKeyService>>),
    Native(Arc<Mutex<SignService>>),
}

impl Signer {
    pub fn new_with_sign_service(sign_service: SignService) -> Self {
        Self::Native(Arc::new(Mutex::new(sign_service)))
    }

    pub fn new_with_private_key_service(
        private_key_service: PrivateKeyService,
    ) -> Self {
        Self::PrivateKey(Arc::new(Mutex::new(private_key_service)))
    }
}

#[allow(dead_code)]
pub struct AccountClient {
    owner: String,
    chain_id: u64,
    config: Config,
    signer: Signer,
    safe: bool,
}

impl AccountClient {
    pub fn new_with_sign_service(
        owner: String,
        chain_id: u64,
        config: Config,
        sign_service: SignService,
    ) -> Self {
        Self {
            owner,
            chain_id,
            config: config.clone(),
            signer: Signer::Native(Arc::new(Mutex::new(sign_service))),
            safe: false,
        }
    }

    pub fn new_with_private_key_service(
        owner: String,
        chain_id: u64,
        config: Config,
        private_key_service: PrivateKeyService,
        safe: bool,
    ) -> Self {
        Self {
            owner,
            chain_id,
            config: config.clone(),
            signer: Signer::PrivateKey(Arc::new(Mutex::new(
                private_key_service,
            ))),
            safe,
        }
    }

    pub fn new_with_private_key(
        owner: String,
        chain_id: u64,
        config: Config,
        private_key: String,
    ) -> Self {
        let owner_address: Address = owner.parse().unwrap();
        let private_key_service = PrivateKeyService::new(
            Box::new(move || Ok(private_key.clone())),
            owner_address,
        );
        Self {
            owner,
            chain_id,
            config: config.clone(),
            signer: Signer::PrivateKey(Arc::new(Mutex::new(
                private_key_service,
            ))),
            safe: false,
        }
    }

    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    pub async fn get_address(&self) -> eyre::Result<String> {
        get_address_with_signer(
            self.owner.clone(),
            self.chain_id,
            self.config.clone(),
            self.signer.clone(),
            self.safe,
        )
        .await
    }

    pub async fn sign_message(&self, message: String) -> eyre::Result<String> {
        todo!("Implement sign_message: {}", message)
    }

    pub async fn send_transactions(
        &self,
        transaction: Vec<Transaction>,
    ) -> eyre::Result<B256> {
        send_transactions(
            transaction,
            self.owner.clone(),
            self.chain_id,
            self.config.clone(),
            self.signer.clone(),
            self.safe,
        )
        .await
    }

    pub fn sign_message_with_mnemonic(
        &self,
        message: String,
        mnemonic: String,
    ) -> eyre::Result<String> {
        let sign_service = crate::sign_service::SignService::new_with_mnemonic(
            mnemonic.clone(),
        );

        let signature = sign_service.sign(message)?;

        Ok(signature)
    }

    pub async fn wait_for_user_operation_receipt(
        &self,
        user_operation_hash: B256,
    ) -> eyre::Result<UserOperationReceipt> {
        println!("Querying for receipts...");

        let bundler_base_url = self.config.clone().endpoints.bundler.base_url;

        let bundler_client =
            BundlerClient::new(BundlerConfig::new(bundler_base_url.clone()));
        let receipt = bundler_client
            .wait_for_user_operation_receipt(user_operation_hash)
            .await?;

        println!("Received User Operation receipt: {:?}", receipt);

        let tx_hash = receipt.clone().receipt.transaction_hash;
        println!(
            "UserOperation included: https://sepolia.etherscan.io/tx/{}",
            tx_hash
        );
        Ok(receipt)
    }
}

impl AccountClient {
    pub fn mock() -> Self {
        AccountClient {
            owner: "".to_string(),
            chain_id: 0,
            config: Config::local(),
            signer: Signer::Native(Arc::new(Mutex::new(SignService::mock()))),
            safe: false,
        }
    }
}

pub async fn get_address_with_signer(
    owner: String,
    chain_id: u64,
    config: Config,
    signer: Signer,
    safe: bool,
) -> eyre::Result<String> {
    match signer {
        Signer::PrivateKey(private_key_service) => {
            let private_key_service = private_key_service.clone();
            let private_key_service = private_key_service.lock().await;
            let private_key_signer_key =
                private_key_service.private_key().unwrap();
            let private_key_signer: PrivateKeySigner =
                private_key_signer_key.parse().unwrap();
            get_address_with_private_key_signer(
                owner,
                chain_id,
                config,
                private_key_signer,
                safe,
            )
            .await
        }
        Signer::Native(_sign_service) => {
            todo!("Implement native signer support")
        }
    }
}

pub async fn get_address_with_private_key_signer(
    _owner: String,
    chain_id: u64,
    config: Config,
    signer: PrivateKeySigner,
    safe: bool,
) -> eyre::Result<String> {
    use crate::smart_accounts::simple_account::sender_address::get_sender_address_with_signer;

    let sender_address = if safe {
        safe_test::get_address(signer, config).await?
    } else {
        get_sender_address_with_signer(config, chain_id, signer).await?
    };

    Ok(sender_address.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::private_key_service::PrivateKeyService;

    // mnemonic:`"test test test test test test test test test test test junk"`
    // derived at `m/44'/60'/0'/0/0`
    const PRIVATE_KEY_HEX: &str =
        "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    #[tokio::test]
    async fn test_send_transaction_local() -> eyre::Result<()> {
        let config = Config::local();

        let owner_address =
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string();
        let chain_id = 11155111;

        let private_key_hex = PRIVATE_KEY_HEX.to_string();

        let private_key_service = PrivateKeyService::new(
            Box::new(move || Ok(private_key_hex.clone())),
            owner_address.parse().unwrap(),
        );

        let account_client = AccountClient::new_with_private_key_service(
            owner_address,
            chain_id,
            config,
            private_key_service,
            false,
        );

        let transaction = Transaction::new_from_strings(
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
            "0".to_string(),
            "0x68656c6c6f".to_string(),
        )?;

        let user_operation_hash =
            account_client.send_transactions(vec![transaction]).await?;

        println!("user_operation_hash: {:?}", user_operation_hash);

        let receipt = account_client
            .wait_for_user_operation_receipt(user_operation_hash)
            .await?;

        println!("receipt: {:?}", receipt);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_address_local() -> eyre::Result<()> {
        let expected_address =
            "0x75BD33d92EEAC5Fe41446fcF5953050d691E7fc9".to_string();

        let config = Config::local();

        let owner_address =
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string();
        let chain_id = 11155111;

        let private_key_hex = PRIVATE_KEY_HEX.to_string();

        let private_key_service = PrivateKeyService::new(
            Box::new(move || Ok(private_key_hex.clone())),
            owner_address.parse().unwrap(),
        );

        let account_client = AccountClient::new_with_private_key_service(
            owner_address,
            chain_id,
            config,
            private_key_service,
            false,
        );

        let sender_address = account_client.get_address().await?;

        println!("sender_address: {:?}", sender_address);

        eyre::ensure!(
            sender_address == expected_address,
            "Sender address {} does not match expected address {}",
            sender_address,
            expected_address
        );

        Ok(())
    }
}
