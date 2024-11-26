use crate::{
    chain_abstraction::{
        api::{
            route::{BridgingError, RouteResponse, RouteResponseError},
            status::StatusResponse,
            Transaction,
        },
        client::Client,
        currency::Currency,
    },
    test_helpers::{
        private_faucet, use_account, use_faucet_gas, BRIDGE_ACCOUNT_1,
        BRIDGE_ACCOUNT_2,
    },
};
use alloy::{
    network::{Ethereum, EthereumWallet, TransactionBuilder},
    primitives::{address, Address, U256, U64},
    rlp::Encodable,
    rpc::types::TransactionRequest,
    signers::{k256::ecdsa::SigningKey, local::LocalSigner},
    sol,
    sol_types::SolCall,
    transports::http::Http,
};
use alloy_provider::{
    fillers::{
        BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill,
        NonceFiller, WalletFiller,
    },
    Identity, Provider, ProviderBuilder, ReqwestProvider, RootProvider,
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use ERC20::ERC20Instance;

const CHAIN_ID_OPTIMISM: &str = "eip155:10";
const CHAIN_ID_BASE: &str = "eip155:8453";
const CHAIN_ID_ARBITRUM: &str = "eip155:42161";
const USDC_CONTRACT_OPTIMISM: Address =
    address!("0b2c639c533813f4aa9d7837caf62653d097ff85");
const USDC_CONTRACT_BASE: Address =
    address!("833589fCD6eDb6E08f4c7C32D4f71b54bdA02913");
const USDC_CONTRACT_ARBITRUM: Address =
    address!("af88d065e77c8cC2239327C5EDb3A432268e5831");

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum Chain {
    Base,
    Optimism,
    Arbitrum,
}

impl Chain {
    fn eip155_chain_id(&self) -> &'static str {
        match self {
            Chain::Base => CHAIN_ID_BASE,
            Chain::Optimism => CHAIN_ID_OPTIMISM,
            Chain::Arbitrum => CHAIN_ID_ARBITRUM,
        }
    }

    #[allow(unused)]
    fn chain_id(&self) -> &'static str {
        self.eip155_chain_id().strip_prefix("eip155:").unwrap()
    }

    fn token_address(&self, token: &Token) -> Address {
        match self {
            Chain::Base => match token {
                Token::Usdc => USDC_CONTRACT_BASE,
            },
            Chain::Optimism => match token {
                Token::Usdc => USDC_CONTRACT_OPTIMISM,
            },
            Chain::Arbitrum => match token {
                Token::Usdc => USDC_CONTRACT_ARBITRUM,
            },
        }
    }

    fn from_eip155_chain_id(chain_id: &str) -> Chain {
        match chain_id {
            CHAIN_ID_BASE => Chain::Base,
            CHAIN_ID_OPTIMISM => Chain::Optimism,
            CHAIN_ID_ARBITRUM => Chain::Arbitrum,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum Token {
    Usdc,
}

sol! {
    #[sol(rpc)]
    contract ERC20 {
        function transfer(address to, uint256 amount);
        function approve(address spender, uint256 amount) public returns (bool);
        function balanceOf(address _owner) public view returns (uint256 balance);
    }
}

fn provider_for_chain(chain_id: &Chain) -> ReqwestProvider {
    // let project_id: ProjectId =
    //     std::env::var("REOWN_PROJECT_ID").unwrap().into();
    // let url = format!(
    //     "https://rpc.walletconnect.org/v1?chainId={}&projectId={project_id}",
    //     chain_id.eip155_chain_id()
    // )
    // .parse()
    // .unwrap();
    // https://reown-inc.slack.com/archives/C0816SK4877/p1732598903113679?thread_ts=1732562310.770219&cid=C0816SK4877
    let url = match chain_id {
        Chain::Base => "https://mainnet.base.org",
        Chain::Optimism => "https://mainnet.optimism.io",
        Chain::Arbitrum => "https://arbitrum.gateway.tenderly.co",
    }
    .parse()
    .unwrap();
    ProviderBuilder::new().on_http(url)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BridgeTokenParams {
    chain: Chain,
    account_address: Address,
    token: Token,
}

type BridgeTokenProvider = FillProvider<
    JoinFill<
        JoinFill<
            Identity,
            JoinFill<
                GasFiller,
                JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>,
            >,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<Http<reqwest::Client>>,
    Http<reqwest::Client>,
    Ethereum,
>;

#[derive(Clone)]
struct BridgeToken {
    params: BridgeTokenParams,
    token: ERC20Instance<Http<reqwest::Client>, BridgeTokenProvider, Ethereum>,
    provider: BridgeTokenProvider,
}

impl BridgeToken {
    fn new(
        params: BridgeTokenParams,
        account: LocalSigner<SigningKey>,
    ) -> BridgeToken {
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::new(account))
            .on_provider(provider_for_chain(&params.chain));

        let token_address = params.chain.token_address(&params.token);

        let token = ERC20::new(token_address, provider.clone());

        BridgeToken { params, token, provider }
    }

    async fn native_balance(&self) -> U256 {
        self.provider.get_balance(self.params.account_address).await.unwrap()
    }

    async fn token_balance(&self) -> U256 {
        self.token
            .balanceOf(self.params.account_address)
            .call()
            .await
            .unwrap()
            .balance
    }
}

#[tokio::test]
async fn bridging_routes_routes_available() {
    let faucet = private_faucet();
    println!("faucet: {}", faucet.address());

    // Accounts unique to this test fixture
    let account_1 = use_account(Some(BRIDGE_ACCOUNT_1));
    println!("account_1: {}", account_1.address());
    let account_2 = use_account(Some(BRIDGE_ACCOUNT_2));
    println!("account_2: {}", account_2.address());

    let wallet_lookup = [account_1.clone(), account_2.clone()]
        .into_iter()
        .map(|a| (a.address(), a))
        .collect::<HashMap<_, _>>();

    let token = Token::Usdc;

    let chain_1 = Chain::Base;
    let chain_2 = Chain::Optimism;

    let chain_1_provider = provider_for_chain(&chain_1);
    let chain_2_provider = provider_for_chain(&chain_2);

    let chain_1_address_1_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_1.to_owned(),
            account_address: account_1.address(),
            token,
        },
        account_1.clone(),
    );
    let chain_1_address_2_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_1.to_owned(),
            account_address: account_2.address(),
            token,
        },
        account_2.clone(),
    );
    let chain_2_address_1_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_2.to_owned(),
            account_address: account_1.address(),
            token,
        },
        account_1.clone(),
    );
    let chain_2_address_2_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_2.to_owned(),
            account_address: account_2.address(),
            token,
        },
        account_2.clone(),
    );

    println!("initial token balances:");
    println!(
        "chain_1_address_1_token: {}",
        chain_1_address_1_token.token_balance().await
    );
    println!(
        "chain_1_address_2_token: {}",
        chain_1_address_2_token.token_balance().await
    );
    println!(
        "chain_2_address_1_token: {}",
        chain_2_address_1_token.token_balance().await
    );
    println!(
        "chain_2_address_2_token: {}",
        chain_2_address_2_token.token_balance().await
    );

    struct Sources {
        chain_1_address_1_token: BridgeToken,
        #[allow(unused)]
        chain_1_address_2_token: BridgeToken,
        #[allow(unused)]
        chain_2_address_1_token: BridgeToken,
        chain_2_address_2_token: BridgeToken,
    }
    let sources = Sources {
        chain_1_address_1_token: chain_1_address_1_token.clone(),
        chain_1_address_2_token: chain_1_address_2_token.clone(),
        chain_2_address_1_token: chain_2_address_1_token.clone(),
        chain_2_address_2_token: chain_2_address_2_token.clone(),
    };

    #[derive(Debug)]
    enum Source {
        Left,
        Right,
    }

    impl Source {
        fn other(&self) -> Source {
            match self {
                Source::Left => Source::Right,
                Source::Right => Source::Left,
            }
        }

        fn bridge_token(&self, sources: &Sources) -> BridgeToken {
            match self {
                Source::Left => sources.chain_1_address_1_token.clone(),
                Source::Right => sources.chain_2_address_2_token.clone(),
            }
        }

        fn address(&self, sources: &Sources) -> Address {
            self.bridge_token(sources).params.account_address
        }

        async fn token_balance(&self, sources: &Sources) -> U256 {
            match self {
                Source::Left => {
                    sources.chain_1_address_1_token.token_balance().await
                }
                Source::Right => {
                    sources.chain_2_address_2_token.token_balance().await
                }
            }
        }
    }

    async fn estimate_total_fees(
        _wallet: &EthereumWallet,
        provider: &ReqwestProvider,
        txn: TransactionRequest,
    ) -> U256 {
        let gas = txn.gas.unwrap();
        let max_fee_per_gas = txn.max_fee_per_gas.unwrap();

        let provider_chain_id =
            format!("eip155:{}", provider.get_chain_id().await.unwrap());
        let l1_data_fee = if provider_chain_id == CHAIN_ID_BASE
            || provider_chain_id == CHAIN_ID_OPTIMISM
        {
            // https://docs.optimism.io/builders/app-developers/transactions/fees#l1-data-fee
            sol! {
                #[sol(rpc)]
                contract GasPriceOracle {
                    function getL1Fee(bytes memory _data) public view returns (uint256);
                }
            }
            // https://github.com/wevm/viem/blob/ae3b8aeab22d56b2cf6d3b05e4f9eeaab7cf81fe/src/op-stack/contracts.ts#L8
            let oracle_address =
                address!("420000000000000000000000000000000000000F");
            let oracle = GasPriceOracle::new(oracle_address, provider.clone());
            let x = txn.build_unsigned().unwrap();
            let txn = x.eip1559().unwrap();
            let mut buf = Vec::with_capacity(txn.length());
            txn.encode(&mut buf);
            // txn.build_unsigned().unwrap().eip1559().unwrap().
            // let built = txn.build(wallet).await.unwrap();
            // let mut buf = Vec::with_capacity(built.eip2718_encoded_length());
            // built.as_eip1559().unwrap().rlp_encode(&mut buf);
            let current_l1_fee =
                oracle.getL1Fee(buf.into()).call().await.unwrap()._0;
            // The fee can change a maximum of 12.5% per mainnet block: https://docs.optimism.io/builders/app-developers/transactions/fees#mechanism
            // Multiplying by 2 gives us 6 blocks of buffer, and also is simpler
            // to implement here w/ integers (vs floats)
            current_l1_fee * U256::from(2)

            // TODO also consider "blob fee" (max_fee_per_blob_gas): https://docs.optimism.io/builders/app-developers/transactions/fees#mechanism
        } else {
            U256::ZERO
        };
        println!("l1_data_fee: {l1_data_fee}");

        U256::from(max_fee_per_gas) * U256::from(gas) + l1_data_fee
    }

    async fn send_sponsored_txn(
        faucet: LocalSigner<SigningKey>,
        provider: &ReqwestProvider,
        wallet_lookup: &HashMap<Address, LocalSigner<SigningKey>>,
        txn: TransactionRequest,
    ) {
        let provider_chain_id =
            format!("eip155:{}", provider.get_chain_id().await.unwrap());
        let from_address = txn.from.unwrap();
        let wallet = EthereumWallet::new(
            wallet_lookup.get(&from_address).unwrap().clone(),
        );

        let txn = txn.with_nonce(
            provider.get_transaction_count(from_address).await.unwrap(),
        );

        let gas = provider.estimate_gas(&txn).await.unwrap();
        let fees = provider.estimate_eip1559_fees(None).await.unwrap();
        let txn = txn
            .gas_limit(gas)
            .max_fee_per_gas(fees.max_fee_per_gas)
            .max_priority_fee_per_gas(fees.max_priority_fee_per_gas);

        let total_fee =
            estimate_total_fees(&wallet, provider, txn.clone()).await;

        let balance = provider.get_balance(from_address).await.unwrap();
        if balance < total_fee {
            let additional_balance_required = total_fee - balance;
            println!(
                "additional_balance_required: {additional_balance_required}"
            );
            println!(
                "using faucet (2) for {}:{} at {}",
                provider_chain_id, from_address, additional_balance_required
            );
            use_faucet_gas(
                provider.clone(),
                faucet.clone(),
                U256::from(additional_balance_required),
                from_address,
                4,
            )
            .await;
            println!("funded");
        }

        let start = Instant::now();
        loop {
            println!("sending txn: {:?}", txn);
            let txn_sent = ProviderBuilder::new()
                .with_recommended_fillers()
                .wallet(wallet.clone())
                .on_provider(provider)
                .send_transaction(txn.clone())
                .await
                .unwrap()
                // .with_required_confirmations(3)
                .with_timeout(Some(Duration::from_secs(15)));
            println!(
                "txn hash: {} on chain {provider_chain_id}",
                txn_sent.tx_hash()
            );
            // if provider
            //     .get_transaction_by_hash(*txn_sent.tx_hash())
            //     .await
            //     .unwrap()
            //     .is_none()
            // {
            //     println!("get_transaction_by_hash returned None,
            // retrying...");     continue;
            // }
            let receipt = txn_sent.get_receipt().await;
            if let Ok(receipt) = receipt {
                assert!(receipt.status());
                break;
            }

            println!("error getting receipt: {:?}", receipt);
            if start.elapsed() > Duration::from_secs(30) {
                panic!("timed out");
            }
        }
    }

    // Consolidate balances if necessary to the source and destination accounts.
    // Vias should be 0 before rest of test is run
    let via1 = chain_1_address_2_token.token_balance().await;
    let via2 = chain_2_address_1_token.token_balance().await;
    println!("via balances: {} {}", via1, via2);
    if !via1.is_zero() {
        println!("via1 txn sending");
        send_sponsored_txn(
            faucet.clone(),
            &chain_1_provider,
            &wallet_lookup,
            TransactionRequest::default()
                .with_from(chain_1_address_2_token.params.account_address)
                .with_to(*chain_1_address_2_token.token.address())
                .with_input(
                    ERC20::transferCall {
                        to: chain_1_address_1_token.params.account_address,
                        amount: via1,
                    }
                    .abi_encode(),
                ),
        )
        .await;
        println!("via1 txn complete");
    }
    if !via2.is_zero() {
        println!("via2 txn sending");
        send_sponsored_txn(
            faucet.clone(),
            &chain_2_provider,
            &wallet_lookup,
            TransactionRequest::default()
                .with_from(chain_2_address_1_token.params.account_address)
                .with_to(*chain_2_address_1_token.token.address())
                .with_input(
                    ERC20::transferCall {
                        to: chain_2_address_2_token.params.account_address,
                        amount: via2,
                    }
                    .abi_encode(),
                ),
        )
        .await;
        println!("via2 txn complete");
    }
    assert!(chain_1_address_2_token.token_balance().await.is_zero());
    assert!(chain_2_address_1_token.token_balance().await.is_zero());

    println!("token balances after via removal:");
    println!(
        "chain_1_address_1_token: {}",
        chain_1_address_1_token.token_balance().await
    );
    println!(
        "chain_1_address_2_token: {}",
        chain_1_address_2_token.token_balance().await
    );
    println!(
        "chain_2_address_1_token: {}",
        chain_2_address_1_token.token_balance().await
    );
    println!(
        "chain_2_address_2_token: {}",
        chain_2_address_2_token.token_balance().await
    );

    let send_amount = U256::from(1_500_000); // 1.5 USDC (6 decimals)
    let required_amount =
        U256::from((send_amount.to::<u128>() as f64 * 1.05) as u128);

    let chain_1_balance = chain_1_address_1_token.token_balance().await;
    let chain_2_balance = chain_2_address_2_token.token_balance().await;
    let (faucet_required, source) = match (chain_1_balance, chain_2_balance) {
        (balance_1, _balance_2) if balance_1 >= required_amount => {
            (false, Source::Left)
        }
        (_balance_1, balance_2) if balance_2 >= required_amount => {
            (false, Source::Right)
        }
        _ => (true, Source::Left),
    };
    println!("source: {:?}", source);

    if faucet_required {
        assert!(required_amount < U256::from(2000000));
        println!(
            "using token faucet {} on chain {} for amount {required_amount} on token {:?} ({}). Send tokens to faucet at: {}",
            faucet.address(),
            chain_1_address_1_token.params.chain.eip155_chain_id(),
            token,
            chain_1_address_1_token.token.address(),
            faucet.address(),
        );
        let status = BridgeToken::new(
            chain_1_address_1_token.params.clone(),
            faucet.clone(),
        )
        .token
        .transfer(account_1.address(), required_amount)
        .send()
        .await
        .unwrap()
        .with_timeout(Some(Duration::from_secs(30)))
        .get_receipt()
        .await
        .unwrap()
        .status();
        assert!(status);
    }
    assert!(source.token_balance(&sources).await >= required_amount);

    let transaction = Transaction {
        from: source.address(&sources),
        to: *source.other().bridge_token(&sources).token.address(),
        value: U256::ZERO,
        // gas: U64::ZERO,
        // https://reown-inc.slack.com/archives/C0816SK4877/p1731962527043399
        gas: U64::from(50000), // until Blockchain API estimates this
        data: ERC20::transferCall {
            to: source.other().address(&sources),
            amount: send_amount,
        }
        .abi_encode()
        .into(),
        nonce: U64::from({
            let token = match source {
                Source::Left => &chain_2_address_1_token,
                Source::Right => &chain_1_address_2_token,
            };
            token
                .provider
                .get_transaction_count(token.params.account_address)
                .await
                .unwrap()
        }),
        chain_id: source
            .other()
            .bridge_token(&sources)
            .params
            .chain
            .eip155_chain_id()
            .to_owned(),
        gas_price: U256::ZERO,
        max_fee_per_gas: U256::ZERO,
        max_priority_fee_per_gas: U256::ZERO,
    };
    println!("input transaction: {:?}", transaction);

    let project_id = std::env::var("REOWN_PROJECT_ID").unwrap().into();
    let client = Client::new(project_id);
    let mut result = client
        .route(transaction.clone())
        .await
        .unwrap()
        .into_result()
        .unwrap()
        .into_option()
        .unwrap();
    println!("route result: {:?}", result);

    assert_eq!(result.transactions.len(), 2);
    result.transactions[0].gas = U64::from(60000 /* 55437 */); // until Blockchain API estimates this
    result.transactions[1].gas = U64::from(140000 /* 107394 */); // until Blockchain API estimates this

    println!(
        "output result: {:?}",
        client
            .get_route_ui_fields(
                result.clone(),
                transaction.clone(),
                Currency::Usd,
                "normal".to_owned()
            )
            .await
            .unwrap()
    );

    fn map_transaction(txn: Transaction) -> TransactionRequest {
        TransactionRequest::default()
            .with_from(txn.from)
            .with_to(txn.to)
            .with_value(txn.value)
            .with_gas_limit(txn.gas.to())
            .with_input(txn.data)
            .with_nonce(txn.nonce.to())
            .with_chain_id(
                txn.chain_id
                    .strip_prefix("eip155:")
                    .unwrap()
                    .parse::<U64>()
                    .unwrap()
                    .to(),
            )
    }

    let mut total_fees = HashMap::new();
    let mut transactions_with_fees = vec![];
    for txn in
        result.transactions.into_iter().chain(std::iter::once(transaction))
    {
        let provider =
            provider_for_chain(&Chain::from_eip155_chain_id(&txn.chain_id));
        let fees = provider.estimate_eip1559_fees(None).await.unwrap();
        let txn = map_transaction(txn)
            .with_max_fee_per_gas(fees.max_fee_per_gas)
            .with_max_priority_fee_per_gas(fees.max_priority_fee_per_gas);
        let fee = estimate_total_fees(
            &EthereumWallet::new(
                wallet_lookup.get(&txn.from.unwrap()).unwrap().clone(),
            ),
            &provider,
            txn.clone(),
        )
        .await;
        println!("total fee: {} for txn {:?}", fee, txn);
        total_fees
            .entry((
                format!("eip155:{}", txn.chain_id.unwrap()),
                txn.from.unwrap(),
            ))
            .and_modify(|f| *f += fee)
            .or_insert(fee);
        transactions_with_fees.push(txn);
    }

    for ((chain_id, address), total_fee) in total_fees {
        let provider =
            provider_for_chain(&Chain::from_eip155_chain_id(&chain_id));
        let balance = provider.get_balance(address).await.unwrap();
        if total_fee > balance {
            let additional_balance_required = total_fee - balance;
            println!("using faucet (1) for {chain_id}:{address} at {additional_balance_required}");
            use_faucet_gas(
                provider,
                faucet.clone(),
                additional_balance_required,
                address,
                4,
            )
            .await;
            println!("funded");
        }
    }

    let original_source_balance = source.token_balance(&sources).await;
    let original_destination_balance =
        source.other().bridge_token(&sources).token_balance().await;

    let (bridge, original) =
        transactions_with_fees.split_at(transactions_with_fees.len() - 1);
    assert_eq!(bridge.len(), 2);
    assert_eq!(original.len(), 1);
    let original = original.first().unwrap();

    let status = client.status(result.orchestration_id.clone()).await.unwrap();
    assert!(matches!(status, StatusResponse::Pending(_)));

    let approval_start = Instant::now();

    let mut pending_bridge_txn_hashes = Vec::with_capacity(bridge.len());
    for txn in bridge {
        let provider = provider_for_chain(&Chain::from_eip155_chain_id(
            &format!("eip155:{}", txn.chain_id.unwrap()),
        ));
        let start = Instant::now();
        loop {
            println!("sending txn: {:?}", txn);
            let txn_sent = ProviderBuilder::new()
                .wallet(EthereumWallet::new(
                    wallet_lookup.get(&txn.from.unwrap()).unwrap().clone(),
                ))
                .on_provider(provider.clone())
                .send_transaction(txn.clone())
                .await
                .unwrap()
                // .with_required_confirmations(3)
                .with_timeout(Some(Duration::from_secs(15)));

            let tx_hash = *txn_sent.tx_hash();
            println!(
                "txn hash: {} on chain {}",
                tx_hash,
                txn.chain_id.unwrap()
            );
            // if provider
            //     .get_transaction_by_hash(tx_hash)
            //     .await
            //     .unwrap()
            //     .is_none()
            // {
            //     println!("get_transaction_by_hash returned None,
            // retrying...");     continue;
            // }
            let receipt = txn_sent.get_receipt().await;
            if let Ok(receipt) = receipt {
                assert!(receipt.status());
                pending_bridge_txn_hashes.push((provider.clone(), tx_hash));
                break;
            }

            println!("error getting receipt: {:?}", receipt);
            if start.elapsed() > Duration::from_secs(30) {
                panic!("timed out");
            }

            // let receipt = pending_txn
            //     .provider()
            //     .get_transaction_receipt(*hash)
            //     .await
            //     .unwrap()
            //     .unwrap();
            // let status = receipt.status();
            // assert!(status);
        }
    }

    let status = client
        .wait_for_success(
            result.orchestration_id,
            Duration::from_millis(result.metadata.check_in),
        )
        .await
        .unwrap();
    println!("status: {:?}", status);
    println!("bridge success in {:?}", approval_start.elapsed());

    for (provider, pending_txn_hash) in pending_bridge_txn_hashes {
        let receipt = provider
            .get_transaction_receipt(pending_txn_hash)
            .await
            .unwrap()
            .unwrap();
        println!("txn hash: {}", receipt.transaction_hash);
        let status = receipt.status();
        assert!(status);
    }
    println!("confirmed receipts in {:?}", approval_start.elapsed());

    let provider = provider_for_chain(&Chain::from_eip155_chain_id(&format!(
        "eip155:{}",
        original.chain_id.unwrap()
    )));

    let start = Instant::now();
    loop {
        println!("sending txn: {:?}", original);
        let txn_sent = match ProviderBuilder::new()
            .wallet(EthereumWallet::new(
                wallet_lookup.get(&original.from.unwrap()).unwrap().clone(),
            ))
            .on_provider(provider.clone())
            .send_transaction(original.clone())
            .await
        {
            Ok(txn_sent) => {
                txn_sent
                    // .with_required_confirmations(3)
                    .with_timeout(Some(Duration::from_secs(15)))
            }
            Err(e) => {
                println!("error sending txn: {:?}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };
        println!(
            "txn hash: {} on chain {}",
            txn_sent.tx_hash(),
            original.chain_id.unwrap()
        );
        // if provider
        //     .get_transaction_by_hash(*txn_sent.tx_hash())
        //     .await
        //     .unwrap()
        //     .is_none()
        // {
        //     println!("get_transaction_by_hash returned None, retrying...");
        //     continue;
        // }
        let receipt = txn_sent.get_receipt().await;
        if let Ok(receipt) = receipt {
            assert!(receipt.status());
            break;
        }

        println!("error getting receipt: {:?}", receipt);
        if start.elapsed() > Duration::from_secs(30) {
            panic!("timed out");
        }
    }
    println!("original txn finished in {:?}", approval_start.elapsed());

    println!("final token balances:");
    println!(
        "chain_1_address_1_token: {}",
        chain_1_address_1_token.token_balance().await
    );
    println!(
        "chain_1_address_2_token: {}",
        chain_1_address_2_token.token_balance().await
    );
    println!(
        "chain_2_address_1_token: {}",
        chain_2_address_1_token.token_balance().await
    );
    println!(
        "chain_2_address_2_token: {}",
        chain_2_address_2_token.token_balance().await
    );

    let new_source_balance = source.token_balance(&sources).await;
    let new_destination_balance =
        source.other().bridge_token(&sources).token_balance().await;

    assert!(new_destination_balance > original_destination_balance);
    assert!(new_source_balance < original_source_balance);
    assert_eq!(
        new_destination_balance,
        original_destination_balance + send_amount
    );
}

#[tokio::test]
async fn bridging_routes_routes_insufficient_funds() {
    let account_1 = LocalSigner::random();
    println!("account_1: {}", account_1.address());
    let account_2 = LocalSigner::random();
    println!("account_2: {}", account_2.address());

    let token = Token::Usdc;

    let chain_1 = Chain::Base;
    let chain_2 = Chain::Optimism;

    let chain_1_address_1_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_1.to_owned(),
            account_address: account_1.address(),
            token,
        },
        account_1.clone(),
    );
    let chain_1_address_2_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_1.to_owned(),
            account_address: account_2.address(),
            token,
        },
        account_2.clone(),
    );
    let chain_2_address_1_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_2.to_owned(),
            account_address: account_1.address(),
            token,
        },
        account_1.clone(),
    );
    let chain_2_address_2_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_2.to_owned(),
            account_address: account_2.address(),
            token,
        },
        account_2.clone(),
    );
    assert_eq!(chain_1_address_1_token.token_balance().await, U256::ZERO);
    assert_eq!(chain_1_address_2_token.token_balance().await, U256::ZERO);
    assert_eq!(chain_2_address_1_token.token_balance().await, U256::ZERO);
    assert_eq!(chain_2_address_2_token.token_balance().await, U256::ZERO);
    assert_eq!(chain_1_address_1_token.native_balance().await, U256::ZERO);
    assert_eq!(chain_1_address_2_token.native_balance().await, U256::ZERO);
    assert_eq!(chain_2_address_1_token.native_balance().await, U256::ZERO);
    assert_eq!(chain_2_address_2_token.native_balance().await, U256::ZERO);

    let send_amount = U256::from(1_500_000); // 1.5 USDC (6 decimals)

    let transaction = Transaction {
        from: account_1.address(),
        to: *chain_1_address_2_token.token.address(),
        value: U256::ZERO,
        gas: U64::ZERO,
        data: ERC20::transferCall {
            to: account_2.address(),
            amount: send_amount,
        }
        .abi_encode()
        .into(),
        nonce: U64::ZERO,
        chain_id: chain_1.eip155_chain_id().to_owned(),
        gas_price: U256::ZERO,
        max_fee_per_gas: U256::ZERO,
        max_priority_fee_per_gas: U256::ZERO,
    };
    println!("input transaction: {:?}", transaction);

    let project_id = std::env::var("REOWN_PROJECT_ID").unwrap().into();
    let client = Client::new(project_id);
    let result = client.route(transaction.clone()).await.unwrap();
    assert_eq!(
        result,
        RouteResponse::Error(RouteResponseError {
            error: BridgingError::InsufficientFunds,
        })
    );
}
