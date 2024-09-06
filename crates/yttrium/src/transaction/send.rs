use crate::{
    sign_service::SignService, transaction::Transaction,
    user_operation::UserOperationV07,
};
use core::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;

mod safe_test;
mod simple_account_test;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserOperationEstimated(UserOperationV07);

impl Into<UserOperationV07> for UserOperationEstimated {
    fn into(self) -> UserOperationV07 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct SignedUserOperation(UserOperationV07);

impl Into<UserOperationV07> for SignedUserOperation {
    fn into(self) -> UserOperationV07 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SentUserOperationHash(String);

impl From<SentUserOperationHash> for String {
    fn from(user_operation_hash: SentUserOperationHash) -> Self {
        user_operation_hash.0
    }
}

impl fmt::Display for SentUserOperationHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub mod send_tests;

pub async fn send_transaction(
    sign_service: Arc<Mutex<SignService>>,
    transaction: Transaction,
) -> eyre::Result<String> {
    let _ = sign_service.try_lock()?;
    todo!("Calling send_transaction with transaction: {transaction:?}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bundler::{
            client::BundlerClient,
            config::BundlerConfig,
            pimlico::{
                client::BundlerClient as PimlicoBundlerClient,
                paymaster::client::PaymasterClient,
            },
        },
        entry_point::get_sender_address::get_sender_address_v07,
        signer::sign_user_operation_v07_with_ecdsa,
        smart_accounts::{
            nonce::get_nonce,
            simple_account::{
                create_account::SimpleAccountCreate, factory::FactoryAddress,
                SimpleAccountAddress, SimpleAccountExecute,
            },
        },
        user_operation::UserOperationV07,
    };
    use alloy::{
        network::EthereumWallet,
        primitives::{Address, Bytes, U256},
        providers::ProviderBuilder,
        signers::local::{
            coins_bip39::English, MnemonicBuilder, PrivateKeySigner,
        },
    };
    use std::str::FromStr;

    const MNEMONIC_PHRASE: &str =
        "test test test test test test test test test test test junk";

    async fn send_transaction(
        sign_service: Arc<Mutex<SignService>>,
        transaction: Transaction,
    ) -> eyre::Result<String> {
        let sign_service = sign_service.clone();
        let sign_service = sign_service.lock().await;

        let config = crate::config::Config::pimlico();

        let bundler_base_url = config.endpoints.bundler.base_url;

        let bundler_client =
            BundlerClient::new(BundlerConfig::new(bundler_base_url.clone()));

        let pimlico_client: PimlicoBundlerClient = PimlicoBundlerClient::new(
            BundlerConfig::new(bundler_base_url.clone()),
        );

        let chain = crate::chain::Chain::ETHEREUM_SEPOLIA_V07;
        let entry_point_config = chain.entry_point_config();

        let chain_id = chain.id.eip155_chain_id()?;

        let entry_point_address = entry_point_config.address();

        let rpc_url = config.endpoints.rpc.base_url;

        // Create a provider

        let (ethereum_wallet, alloy_signer) = {
            let phrase = MNEMONIC_PHRASE.to_string();
            let index: u32 = 0;

            let local_signer = {
                let local_signer_result = MnemonicBuilder::<English>::default()
                    .phrase(phrase.clone())
                    .index(index)?
                    .build();
                let local_signer = match local_signer_result {
                    Ok(signer) => signer,
                    Err(e) => {
                        println!("Error creating signer: {}", e);
                        let local_signer: PrivateKeySigner = phrase.parse()?;
                        local_signer
                    }
                };
                local_signer
            };
            let ethereum_wallet = EthereumWallet::from(local_signer.clone());
            (ethereum_wallet, local_signer)
        };

        let rpc_url: reqwest::Url = rpc_url.parse()?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(ethereum_wallet.clone())
            .on_http(rpc_url);

        let simple_account_factory_address_primitives: Address =
            "0x91E60e0613810449d098b0b5Ec8b51A0FE8c8985".parse()?;
        let simple_account_factory_address =
            FactoryAddress::new(simple_account_factory_address_primitives);

        let owner = ethereum_wallet.clone().default_signer();
        let owner_address = owner.address();
        let sign_service_owner = sign_service.owner();
        assert_eq!(
            owner_address, sign_service_owner,
            "Owner addresses don't match, should be {:?}, is {:?}",
            owner_address, sign_service_owner
        );

        let factory_data_call = SimpleAccountCreate::new_u64(owner_address, 0);

        let factory_data_value = factory_data_call.encode();

        let factory_data_value_hex = hex::encode(factory_data_value.clone());

        let factory_data_value_hex_prefixed =
            format!("0x{}", factory_data_value_hex);

        println!(
            "Generated factory_data: {:?}",
            factory_data_value_hex_prefixed.clone()
        );

        // 5. Calculate the sender address

        let sender_address = get_sender_address_v07(
            &provider,
            simple_account_factory_address.clone().into(),
            factory_data_value.clone().into(),
            entry_point_address.clone(),
        )
        .await?;

        println!("Calculated sender address: {:?}", sender_address);

        let to: Address = transaction.to.parse()?;
        let value: alloy::primitives::Uint<256, 4> =
            transaction.value.parse()?;
        let data_hex = transaction.data.strip_prefix("0x").unwrap();
        let data: Bytes = Bytes::from_str(data_hex)?;

        let call_data = SimpleAccountExecute::new(to, value, data);
        let call_data_encoded = call_data.encode();
        let call_data_value_hex = hex::encode(call_data_encoded.clone());
        let call_data_value_hex_prefixed = format!("0x{}", call_data_value_hex);

        println!("Generated callData: {:?}", call_data_value_hex_prefixed);

        // Fill out remaining UserOperation values

        let gas_price =
            pimlico_client.estimate_user_operation_gas_price().await?;

        assert!(gas_price.fast.max_fee_per_gas > U256::from(1));

        println!("Gas price: {:?}", gas_price);

        let nonce = get_nonce(
            &provider,
            &SimpleAccountAddress::new(sender_address),
            &entry_point_address,
        )
        .await?;

        let user_op = UserOperationV07 {
            sender: sender_address,
            nonce: U256::from(nonce),
            factory: None,
            factory_data: None,
            call_data: Bytes::from_str(&call_data_value_hex)?,
            call_gas_limit: U256::from(0),
            verification_gas_limit: U256::from(0),
            pre_verification_gas: U256::from(0),
            max_fee_per_gas: gas_price.fast.max_fee_per_gas,
            max_priority_fee_per_gas: gas_price.fast.max_priority_fee_per_gas,
            paymaster: None,
            paymaster_verification_gas_limit: None,
            paymaster_post_op_gas_limit: None,
            paymaster_data: None,
            signature: Bytes::from_str(
                crate::smart_accounts::simple_account::DUMMY_SIGNATURE_HEX
                    .strip_prefix("0x")
                    .unwrap(),
            )?,
        };

        let paymaster_client =
            PaymasterClient::new(BundlerConfig::new(bundler_base_url.clone()));

        let sponsor_user_op_result = paymaster_client
            .sponsor_user_operation_v07(
                &user_op.clone().into(),
                &entry_point_address,
                None,
            )
            .await?;

        println!("sponsor_user_op_result: {:?}", sponsor_user_op_result);

        let sponsored_user_op = {
            let s = sponsor_user_op_result.clone();
            let mut op = user_op.clone();

            op.call_gas_limit = s.call_gas_limit;
            op.verification_gas_limit = s.verification_gas_limit;
            op.pre_verification_gas = s.pre_verification_gas;
            op.paymaster = Some(s.paymaster);
            op.paymaster_verification_gas_limit =
                Some(s.paymaster_verification_gas_limit);
            op.paymaster_post_op_gas_limit =
                Some(s.paymaster_post_op_gas_limit);
            op.paymaster_data = Some(s.paymaster_data);

            op
        };

        println!("Received paymaster sponsor result: {:?}", sponsored_user_op);

        // Sign the UserOperation

        let signed_user_op = sign_user_operation_v07_with_ecdsa(
            &sponsored_user_op.clone(),
            &entry_point_address.to_address(),
            chain_id,
            alloy_signer,
        )?;

        println!("Generated signature: {:?}", signed_user_op.signature);

        let user_operation_hash = bundler_client
            .send_user_operation(
                entry_point_address.to_address(),
                signed_user_op.clone(),
            )
            .await?;

        println!("Received User Operation hash: {:?}", user_operation_hash);

        // let receipt = bundler_client
        //     .get_user_operation_receipt(user_operation_hash.clone())
        //     .await?;

        // println!("Received User Operation receipt: {:?}", receipt);

        // println!("Querying for receipts...");

        // let receipt = bundler_client
        //     .wait_for_user_operation_receipt(user_operation_hash.clone())
        //     .await?;

        // let tx_hash = receipt.receipt.transaction_hash;
        // println!(
        //     "UserOperation included: https://sepolia.etherscan.io/tx/{}",
        //     tx_hash
        // );

        Ok(user_operation_hash)
    }

    #[tokio::test]
    #[ignore = "TODO: rewrite against local infrastructure"]
    async fn test_send_transaction() -> eyre::Result<()> {
        let transaction = Transaction::mock();

        let mnemonic = MNEMONIC_PHRASE.to_string();

        let sign_service =
            crate::sign_service::SignService::mock_with_mnemonic(
                mnemonic.clone(),
            )
            .await;

        let sign_service_arc = Arc::new(Mutex::new(sign_service));

        let transaction_hash =
            send_transaction(sign_service_arc, transaction).await?;

        println!("Transaction sent: {}", transaction_hash);

        Ok(())
    }
}
