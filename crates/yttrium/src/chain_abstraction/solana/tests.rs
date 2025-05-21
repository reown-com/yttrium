use {
    crate::{
        blockchain_api::{BLOCKCHAIN_API_URL_PROD, WALLET_ENDPOINT_PATH},
        call::Call,
        chain_abstraction::{
            amount::Amount,
            client::Client,
            currency::Currency,
            pulse::PULSE_SDK_TYPE,
            solana::usdc_mint,
            tests::{
                get_pulse_metadata, Chain, SolanaChain, USDC_CONTRACT_BASE,
            },
            ui_fields::RouteSig,
        },
        erc20::{Token, ERC20},
        test_helpers::{
            private_faucet, use_account, use_solana_account,
            BRIDGE_ACCOUNT_SOLANA_1, BRIDGE_ACCOUNT_SOLANA_2,
        },
        wallet_service_api::{
            AddressOrNative, Asset, GetAssetsFilters, GetAssetsParams,
        },
    },
    alloy::{
        network::{EthereumWallet, TransactionBuilder},
        primitives::{
            utils::{ParseUnits, Unit},
            U128, U256, U64,
        },
        rpc::types::TransactionRequest,
        signers::SignerSync,
        sol_types::SolCall,
    },
    alloy_provider::{Provider, ProviderBuilder},
    eyre::eyre,
    relay_rpc::domain::ProjectId,
    serde::Deserialize,
    serde_json::json,
    serial_test::serial,
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{commitment_config::CommitmentConfig, signer::Signer},
    spl_associated_token_account::get_associated_token_address,
    std::time::Duration,
    url::Url,
};

#[test_log::test(tokio::test)]
#[serial(happy_path)]
async fn solana_happy_path() {
    let project_id: ProjectId =
        std::env::var("REOWN_PROJECT_ID").unwrap().into();
    let blockchain_api_url = std::env::var("BLOCKCHAIN_API_URL")
        .unwrap_or(BLOCKCHAIN_API_URL_PROD.to_string())
        .parse::<Url>()
        .unwrap();
    let client = Client::with_blockchain_api_url(
        project_id.clone(),
        get_pulse_metadata(),
        blockchain_api_url.clone(),
    );
    let sol_rpc = "https://api.mainnet-beta.solana.com";
    let client_sol = RpcClient::new_with_commitment(
        sol_rpc.to_string(),
        CommitmentConfig::confirmed(), // TODO what commitment level should we use?
    );

    let faucet = private_faucet();
    println!("faucet: {}", faucet.address());

    // Accounts unique to this test fixture
    let account_eth = use_account(Some(BRIDGE_ACCOUNT_SOLANA_1));
    println!("account_eth: {}", account_eth.address());
    let account_solana = use_solana_account(BRIDGE_ACCOUNT_SOLANA_2);
    let faucet_solana = use_solana_account(0);
    println!("account_solana: {}", account_solana.pubkey());

    let chain_eth = Chain::Base;
    let chain_solana = SolanaChain::Mainnet;

    let token = Token::Usdc;
    let usdc_erc20_faucet = ERC20::new(
        chain_eth.token_address(&token),
        ProviderBuilder::new()
            .wallet(EthereumWallet::new(faucet.clone()))
            .on_provider(
                client.provider_pool.get_provider(chain_eth.caip2()).await,
            ),
    );
    let usdc_erc20_account_eth = ERC20::new(
        chain_eth.token_address(&token),
        ProviderBuilder::new()
            .wallet(EthereumWallet::new(account_eth.clone()))
            .on_provider(
                client.provider_pool.get_provider(chain_eth.caip2()).await,
            ),
    );

    let send_amount = U256::from(1_500_000); // 1.5 USDC (6 decimals)

    let initial_transaction = Call {
        to: chain_eth.token_address(&token),
        value: U256::ZERO,
        input: ERC20::transferCall {
            // Spend amount = $1.5 USDC -> is sent to faucet account
            _to: faucet.address(),
            _value: send_amount,
        }
        .abi_encode()
        .into(),
    };

    let account_sol_token_account =
        get_associated_token_address(&account_solana.pubkey(), &usdc_mint());
    let account_sol_usdc_balance_result =
        client_sol.get_token_account_balance(&account_sol_token_account).await;
    let account_sol_usdc_balance_ui_amount =
        match account_sol_usdc_balance_result {
            Ok(balance) => balance.ui_amount.unwrap(),
            Err(e) => {
                if e.to_string()
                    .contains("Invalid param: could not find account")
                {
                    0.
                } else {
                    panic!("Error getting token account balance: {:?}", e);
                }
            }
        };
    let faucet_usdc_balance = usdc_erc20_faucet
        .balanceOf(faucet.address())
        .call()
        .await
        .unwrap()
        .balance;
    let needs_usdc = account_sol_usdc_balance_ui_amount < 2.0;
    if needs_usdc {
        // Check if sender has enough USDC
        if faucet_usdc_balance < send_amount * U256::from(2) {
            let unit = Unit::new(
                usdc_erc20_faucet.decimals().call().await.unwrap()._0,
            )
            .unwrap();
            let want_amount = ParseUnits::from(send_amount).format_units(unit);
            let result = reqwest::Client::new().post("https://faucetbot.dev/api/faucet-request")
                .json(&serde_json::json!({
                    "key": std::env::var("FAUCET_REQUEST_API_KEY").unwrap(),
                    "text": format!("Yttrium tests running low on USDC. Please send {want_amount} USDC to {} on {}", faucet.address(), chain_eth.caip2()),
                }))
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();
            println!("requested funds from faucetbot: {result}");
        }

        let quote = reqwest::Client::new()
            .get("https://li.quest/v1/quote/toAmount")
            .query(&json!({
                "fromChain": "BAS",
                "toChain": "SOL",
                "fromToken": USDC_CONTRACT_BASE.to_string(),
                "toToken": usdc_mint().to_string(),
                "toAmount": 2_000_000,
                "fromAddress": faucet.address().to_string(),
                "toAddress": account_solana.pubkey().to_string(),
            }))
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        println!("Quote: {}", serde_json::to_string_pretty(&quote).unwrap());

        let route = quote["action"].clone();
        assert_eq!(
            route["fromAddress"].as_str().unwrap(),
            faucet.address().to_string()
        );
        assert_eq!(route["fromChainId"].as_u64().unwrap(), 8453);
        assert!(
            route["fromAmount"].as_str().unwrap().parse::<u64>().unwrap()
                >= 2000000
        );
        let from_token = route["fromToken"].clone();
        assert_eq!(
            from_token["address"].as_str().unwrap(),
            USDC_CONTRACT_BASE.to_string()
        );
        assert_eq!(from_token["chainId"].as_u64().unwrap(), 8453);
        assert_eq!(from_token["symbol"].as_str().unwrap(), "USDC");
        assert_eq!(from_token["decimals"].as_u64().unwrap(), 6);

        let to_token = route["toToken"].clone();
        assert_eq!(
            to_token["address"].as_str().unwrap(),
            usdc_mint().to_string()
        );
        assert_eq!(to_token["chainId"].as_u64().unwrap(), 1151111081099710);
        assert_eq!(to_token["symbol"].as_str().unwrap(), "USDC");
        assert_eq!(to_token["decimals"].as_u64().unwrap(), 6);

        let transaction_request = quote["transactionRequest"].clone();

        let bridge_contract =
            transaction_request["to"].as_str().unwrap().parse().unwrap();

        let allowance = usdc_erc20_faucet
            .allowance(faucet.address(), bridge_contract)
            .call()
            .await
            .unwrap()
            .remaining;
        println!("Allowance: {}", allowance);
        if allowance < send_amount * U256::from(2) {
            assert!(usdc_erc20_faucet
                .approve(
                    bridge_contract,
                    send_amount.checked_mul(U256::from(2)).unwrap()
                )
                .send()
                .await
                .unwrap()
                .with_timeout(Some(Duration::from_secs(60)))
                .get_receipt()
                .await
                .unwrap()
                .status());
        }

        let transaction_request = TransactionRequest::default()
            .with_chain_id(transaction_request["chainId"].as_u64().unwrap())
            .with_from(
                transaction_request["from"].as_str().unwrap().parse().unwrap(),
            )
            .with_to(bridge_contract)
            // .with_value(
            //     transaction_request["value"].as_str().unwrap().parse().unwrap(),
            // )
            .with_input(
                hex::decode(
                    transaction_request["data"]
                        .as_str()
                        .unwrap()
                        .strip_prefix("0x")
                        .unwrap(),
                )
                .unwrap(),
            )
            .with_gas_price(
                transaction_request["gasPrice"]
                    .as_str()
                    .unwrap()
                    .parse::<U128>()
                    .unwrap()
                    .to(),
            )
            .with_gas_limit(
                transaction_request["gasLimit"]
                    .as_str()
                    .unwrap()
                    .parse::<U64>()
                    .unwrap()
                    .to(),
            );
        let receipt = ProviderBuilder::new()
            .wallet(EthereumWallet::new(faucet.clone()))
            .on_provider(
                client.provider_pool.get_provider(chain_eth.caip2()).await,
            )
            .send_transaction(transaction_request)
            .await
            .unwrap()
            .with_timeout(Some(Duration::from_secs(60)))
            .get_receipt()
            .await
            .unwrap();
        println!(
            "Receipt: {}",
            serde_json::to_string_pretty(&receipt).unwrap()
        );
        assert!(receipt.status());

        await_status(receipt.transaction_hash.to_string()).await;

        async fn await_status(tx_hash: String) {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            tokio::time::timeout(
                tokio::time::Duration::from_secs(60),
                async move {
                    while get_status(tx_hash.clone()).await.is_err() {
                        tokio::time::sleep(tokio::time::Duration::from_secs(5))
                            .await;
                    }
                },
            )
            .await
            .unwrap();
        }

        async fn get_status(tx_hash: String) -> eyre::Result<()> {
            let status = reqwest::Client::new()
                .get("https://li.quest/v1/status")
                .query(&json!({
                    "txHash": tx_hash,
                }))
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;
            println!("Status: {}", serde_json::to_string_pretty(&status)?);
            let status = serde_json::from_value::<Status>(status)?;
            if status.status == "DONE" {
                Ok(())
            } else {
                Err(eyre!("Status: {}", status.status))
            }
        }

        #[derive(Deserialize)]
        struct Status {
            status: String,
        }
    }
    let new_account_sol_usdc_balance = client_sol
        .get_token_account_balance(&account_sol_token_account)
        .await
        .unwrap();
    println!(
        "new_account_sol_usdc_balance: {}",
        new_account_sol_usdc_balance.ui_amount.unwrap()
    );
    assert!(new_account_sol_usdc_balance.ui_amount.unwrap() >= 2.0);

    let faucet_sol_sol_balance =
        client_sol.get_balance(&faucet_solana.pubkey()).await.unwrap();
    let account_sol_sol_balance =
        client_sol.get_balance(&account_solana.pubkey()).await.unwrap();
    let data_len = client_sol
        .get_account(&account_solana.pubkey())
        .await
        .unwrap()
        .data
        .len();
    println!("data_len: {}", data_len);
    let minimum_balance_for_rent_exemption = client_sol
        .get_minimum_balance_for_rent_exemption(data_len)
        .await
        .unwrap();
    println!(
        "minimum_balance_for_rent_exemption: {}",
        minimum_balance_for_rent_exemption
    );
    let min_balance = minimum_balance_for_rent_exemption * 2;

    if account_sol_sol_balance < min_balance {
        println!("funding from faucet: {}", faucet_solana.pubkey());

        let faucet_amount = min_balance;
        #[allow(clippy::zero_prefixed_literal)]
        if faucet_sol_sol_balance - faucet_amount < 0_001_000_000 {
            // 0.001 SOL = ~$0.15
            panic!(
                "!!!!! Faucet doesn't have enough SOL. Please send at least 0.001 SOL to {}",
                faucet_solana.pubkey()
            );
        }

        println!(
            "Preparing transfer transaction... faucet_amount: {}",
            faucet_amount
        );
        // Create transfer instruction
        let transfer_ix = solana_sdk::system_instruction::transfer(
            &faucet_solana.pubkey(),
            &account_solana.pubkey(),
            faucet_amount,
        );

        // Get recent blockhash
        let recent_blockhash = client_sol.get_latest_blockhash().await.unwrap();

        // Create and sign transaction
        let transaction =
            solana_sdk::transaction::Transaction::new_signed_with_payer(
                &[transfer_ix],
                Some(&faucet_solana.pubkey()),
                &[&faucet_solana],
                recent_blockhash,
            );

        println!("Sending transaction...");
        // Send and confirm transaction
        let signature = client_sol
            .send_and_confirm_transaction(&transaction)
            .await
            .unwrap();
        println!("signature: {signature}");
    }
    let new_account_sol_sol_balance =
        client_sol.get_balance(&account_solana.pubkey()).await.unwrap();
    println!("new_account_sol_sol_balance: {}", new_account_sol_sol_balance);
    assert!(new_account_sol_sol_balance >= min_balance);

    let min_eth_balance = U256::from(5_000_000_000_000u64); // $0.01 @ $2k ETH price
    let account_eth_eth_balance = client
        .provider_pool
        .get_provider(chain_eth.caip2())
        .await
        .get_balance(account_eth.address())
        .await
        .unwrap();
    if account_eth_eth_balance < min_eth_balance {
        assert!(ProviderBuilder::new()
            .wallet(EthereumWallet::new(faucet.clone()))
            .on_provider(
                client.provider_pool.get_provider(chain_eth.caip2()).await
            )
            .send_transaction(
                TransactionRequest::default()
                    .with_chain_id(chain_eth.chain_id().parse().unwrap())
                    .with_from(faucet.address())
                    .with_to(account_eth.address())
                    .with_value(min_eth_balance),
            )
            .await
            .unwrap()
            .with_timeout(Some(Duration::from_secs(60)))
            .get_receipt()
            .await
            .unwrap()
            .status());
    }

    // Assert balance of EVM account USDC < $1
    // If higher, send to faucet address
    let account_eth_usdc_balance = usdc_erc20_account_eth
        .balanceOf(account_eth.address())
        .call()
        .await
        .unwrap()
        .balance;
    if account_eth_usdc_balance >= send_amount.div_ceil(U256::from(2)) {
        assert!(usdc_erc20_account_eth
            .transfer(faucet.address(), account_eth_usdc_balance)
            .send()
            .await
            .unwrap()
            .with_timeout(Some(Duration::from_secs(60)))
            .get_receipt()
            .await
            .unwrap()
            .status());
    }

    let account_sol_usdc_balance = client_sol
        .get_token_account_balance(&account_sol_token_account)
        .await
        .unwrap();
    println!(
        "account_sol_usdc_balance: {}",
        account_sol_usdc_balance.ui_amount.unwrap()
    );
    assert!(account_sol_usdc_balance.ui_amount.unwrap() >= 2.);

    // TODO assert total spendable amount of USDC (EIP-7811) is enough to cover the spend amount: $1

    let mut wallet_service_url =
        blockchain_api_url.join(WALLET_ENDPOINT_PATH).unwrap();
    wallet_service_url
        .query_pairs_mut()
        .append_pair("projectId", project_id.as_ref())
        .append_pair(
            "sessionId",
            client.provider_pool.session_id.to_string().as_str(),
        )
        .append_pair("st", PULSE_SDK_TYPE)
        .append_pair("sv", get_pulse_metadata().sdk_version.as_str())
        .append_pair(
            "accounts",
            &[chain_solana.get_caip10(account_solana.pubkey())].join(","),
        );

    // Wait for cache invalidation on balance call
    tokio::time::sleep(Duration::from_secs(30)).await;
    let assets = client
        .provider_pool
        .get_wallet_provider(None, Some(wallet_service_url))
        .await
        .wallet_get_assets(GetAssetsParams {
            account: account_eth.address(),
            filters: GetAssetsFilters {
                asset_filter: None,
                asset_type_filter: None,
                chain_filter: None,
            },
        })
        .await
        .unwrap();
    println!("assets: {:?}", assets);
    let asset = assets
        .get(&U64::from(chain_eth.chain_id().parse::<u64>().unwrap()))
        .unwrap()
        .iter()
        .find_map(|asset| match asset {
            Asset::Erc20 { data } => {
                if data.address
                    == AddressOrNative::AddressVariant(
                        chain_eth.token_address(&token),
                    )
                {
                    Some(data)
                } else {
                    None
                }
            }
            _ => None,
        });
    if let Some(asset) = asset {
        // TODO finish this get_assets test
        println!("asset: {:?}", asset);
        let amount = Amount::new(
            asset.metadata.symbol.clone(),
            asset.balance,
            Unit::try_from(asset.metadata.decimals).unwrap(),
        );
        // TODO fix asset test
        println!("amount: {:?}", amount);
        // assert!(amount.as_float_inaccurate() >= 1.0);
        // assert!(asset.balance >= send_amount);
    }

    let result = client
        .prepare_detailed(
            chain_eth.caip2().to_string(),
            account_eth.address(),
            initial_transaction.clone(),
            vec![chain_solana.get_caip10(account_solana.pubkey())],
            Currency::Usd,
            false,
        )
        .await
        .unwrap()
        .into_result()
        .unwrap()
        .into_option()
        .unwrap();
    println!("route result: {:?}", result);

    let route_txn_sigs = result
        .route
        .iter()
        .map(|txn| {
            let solana_txn = txn.as_solana().unwrap();
            RouteSig::Solana(
                solana_txn
                    .iter()
                    .map(|txn| {
                        account_solana
                            .sign_message(&txn.transaction_hash_to_sign)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();

    let original_faucet_balance = usdc_erc20_faucet
        .balanceOf(faucet.address())
        .call()
        .await
        .unwrap()
        .balance;
    let expected_faucet_balance = original_faucet_balance + send_amount;

    let initial_txn_sigs = account_eth
        .sign_hash_sync(&result.initial.transaction_hash_to_sign)
        .unwrap();
    let execute_result =
        client.execute(result, route_txn_sigs, initial_txn_sigs).await.unwrap();
    println!("execute_result: {:?}", execute_result);

    // Ensure the faucet account received the USDC
    let new_faucet_balance = usdc_erc20_faucet
        .balanceOf(faucet.address())
        .call()
        .await
        .unwrap()
        .balance;
    assert_eq!(new_faucet_balance, expected_faucet_balance);

    // Ensure the EVM account has less than $1 - non-normative, if we bridge more than necessary amount this may fail
    let new_account_eth_balance = usdc_erc20_account_eth
        .balanceOf(account_eth.address())
        .call()
        .await
        .unwrap()
        .balance;
    assert!(new_account_eth_balance < U256::from(1_000_000));
}
