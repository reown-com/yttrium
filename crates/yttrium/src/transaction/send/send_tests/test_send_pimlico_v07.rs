#[cfg(test)]
mod tests {
    use crate::{
        bundler::{
            client::BundlerClient, config::BundlerConfig,
            pimlico::client::BundlerClient as PimlicoBundlerClient,
            pimlico::paymaster::client::PaymasterClient,
        },
        entry_point::get_sender_address::get_sender_address_v07,
        smart_accounts::simple_account::{
            create_account::SimpleAccountCreate, SimpleAccountExecute,
        },
        user_operation::UserOperationV07,
    };
    use alloy::{
        network::EthereumWallet,
        primitives::{Address, Bytes, U256},
        providers::ProviderBuilder,
        signers::local::{coins_bip39::English, MnemonicBuilder},
    };
    use std::str::FromStr;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    const SIMPLE_ACCOUNT_FACTORY_ADDRESS: &str =
        "0x91E60e0613810449d098b0b5Ec8b51A0FE8c8985";

    const MNEMONIC_PHRASE: &str =
        "test test test test test test test test test test test junk";

    #[tokio::test]
    #[ignore = "TODO: rewrite against local infrastructure"]
    async fn test_send_transaction_pimlico_v07() -> eyre::Result<()> {
        let expected_factory_data_hex = "0x5fbfb9cf000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb922660000000000000000000000000000000000000000000000000000000000000000";

        let expected_call_data_hex = "0xb61d27f6000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa9604500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000568656c6c6f000000000000000000000000000000000000000000000000000000";

        let config = crate::config::Config::pimlico();

        // 3. Create the clients

        let bundler_base_url = config.endpoints.bundler.base_url;

        let bundler_client =
            BundlerClient::new(BundlerConfig::new(bundler_base_url.clone()));

        let pimlico_client: PimlicoBundlerClient = PimlicoBundlerClient::new(
            BundlerConfig::new(bundler_base_url.clone()),
        );

        let phrase = MNEMONIC_PHRASE;
        let index: u32 = 0;
        let chain = crate::chain::Chain::ETHEREUM_SEPOLIA_V07;
        let entry_point_config = chain.entry_point_config();

        let chain_id = chain.id.eip155_chain_id();

        let wallet = MnemonicBuilder::<English>::default()
            .phrase(phrase)
            .index(index)?
            .build()?;

        let alloy_signer =
            alloy::signers::local::PrivateKeySigner::from(wallet.clone());

        let ethereum_wallet = EthereumWallet::from(wallet.clone());

        let mnemonic = phrase.to_string();

        let sign_service = crate::sign_service::SignService::new_with_mnemonic(
            mnemonic.clone(),
        );

        let rpc_url = config.endpoints.rpc.base_url;

        // Create a provider with the wallet.
        let rpc_url: reqwest::Url = rpc_url.parse()?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(ethereum_wallet.clone())
            .on_http(rpc_url);

        // 4.Generate the factory and factoryData

        let _simple_account_factory_address: Address =
            SIMPLE_ACCOUNT_FACTORY_ADDRESS.parse()?;
        let simple_account_factory_address =
            crate::smart_accounts::simple_account::factory::FactoryAddress::new(
                _simple_account_factory_address,
            );

        let entry_point_address = entry_point_config.address();

        let owner = ethereum_wallet.clone().default_signer();
        let owner_address = owner.address();

        let factory_data_call = SimpleAccountCreate::new_u64(owner_address, 0);

        let factory_data_value = factory_data_call.encode();

        let factory_data_value_hex = hex::encode(factory_data_value.clone());

        let factory_data_value_hex_prefixed =
            format!("0x{}", factory_data_value_hex);

        assert_eq!(
            factory_data_value_hex_prefixed.clone(),
            expected_factory_data_hex,
            "Factory data value hex does not match expected factory data hex"
        );

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

        // 6. Generate the callData

        let to: Address =
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".parse()?; // vitalik
        let value = alloy::primitives::Uint::<256, 4>::from(0);

        let data: Bytes =
            Bytes::from_str("0x68656c6c6f".strip_prefix("0x").unwrap())?; // "hello" encoded to utf-8 bytes

        let call_data = SimpleAccountExecute::new(to, value, data);
        let call_data_encoded = call_data.encode();
        let call_data_value_hex = hex::encode(call_data_encoded.clone());
        let call_data_value_hex_prefixed = format!("0x{}", call_data_value_hex);

        println!("Generated callData: {:?}", call_data_value_hex_prefixed);

        assert_eq!(
            call_data_value_hex_prefixed, expected_call_data_hex,
            "Call data value hex does not match expected call data hex"
        );

        // 7. Fill out remaining UserOperation values

        let gas_price =
            pimlico_client.estimate_user_operation_gas_price().await?;

        assert!(gas_price.fast.max_fee_per_gas > U256::from(1));

        println!("Gas price: {:?}", gas_price);

        let nonce = crate::smart_accounts::nonce::get_nonce(
            &provider,
            &crate::smart_accounts::simple_account::SimpleAccountAddress::new(
                sender_address,
            ),
            &entry_point_address,
        )
        .await?;

        let user_op = {
            let user_op = UserOperationV07 {
                sender: sender_address,
                nonce: U256::from(nonce),
                factory: None,
                factory_data: None,
                call_data: Bytes::from_str(&call_data_value_hex).unwrap(),
                call_gas_limit: U256::from(0),
                verification_gas_limit: U256::from(0),
                pre_verification_gas: U256::from(0),
                max_fee_per_gas: gas_price.fast.max_fee_per_gas,
                max_priority_fee_per_gas: gas_price
                    .fast
                    .max_priority_fee_per_gas,
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

            user_op
        };

        // 8. Request Pimlico verifying paymaster sponsorship

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

        // 9. Sign the UserOperation

        let signed_user_op =
            crate::signer::sign_user_operation_v07_with_ecdsa_and_sign_service(
                &sponsored_user_op.clone(),
                &entry_point_address.to_address(),
                chain_id,
                alloy_signer,
                &Arc::new(Mutex::new(sign_service)),
            )?;

        println!("Generated signature: {:?}", signed_user_op.signature);

        // 10. Submit the UserOperation to be bundled

        let user_operation_hash = bundler_client
            .send_user_operation(
                entry_point_address.to_address(),
                signed_user_op.clone(),
            )
            .await?;

        println!("Received User Operation hash: {:?}", user_operation_hash);

        // let's also wait for the userOperation to be included, by continually
        // querying for the receipts

        // println!("Querying for receipts...");

        // let receipt = bundler_client
        //     .wait_for_user_operation_receipt(user_operation_hash)
        //     .await?;

        // let tx_hash = receipt.receipt.transaction_hash;
        // println!(
        //     "UserOperation included: https://sepolia.etherscan.io/tx/{}",
        //     tx_hash
        // );

        Ok(())
    }
}
