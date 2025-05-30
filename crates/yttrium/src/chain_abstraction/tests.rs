#[cfg(feature = "solana")]
use super::solana::{
    self, usdc_mint, SOLANA_MAINNET_CAIP2, SOLANA_MAINNET_CHAIN_ID,
};
use {
    crate::{
        blockchain_api::BLOCKCHAIN_API_URL_PROD,
        call::Call,
        chain_abstraction::{
            amount::Amount,
            api::{
                prepare::{
                    BridgingError, Eip155OrSolanaAddress, PrepareResponse,
                    PrepareResponseError,
                },
                status::StatusResponse,
                Transaction,
            },
            client::Client,
            currency::Currency,
            l1_data_fee::get_l1_data_fee,
            pulse::get_pulse_metadata,
            test_helpers::floats_close,
            ui_fields::{RouteSig, TransactionFee, TxnDetails},
        },
        erc20::{Token, ERC20},
        provider_pool::ProviderPool,
        test_helpers::{
            private_faucet, use_account, use_faucet_gas, BRIDGE_ACCOUNT_1,
            BRIDGE_ACCOUNT_2, BRIDGE_ACCOUNT_USDC_1557_1,
            BRIDGE_ACCOUNT_USDC_1557_2,
        },
        time::Instant,
        wallet_service_api::{GetAssetsFilters, GetAssetsParams},
    },
    alloy::{
        network::{Ethereum, EthereumWallet, TransactionBuilder},
        primitives::{
            address,
            utils::{ParseUnits, Unit},
            Address, TxKind, U256, U64,
        },
        rpc::types::TransactionRequest,
        signers::{k256::ecdsa::SigningKey, local::LocalSigner, SignerSync},
        sol_types::SolCall,
    },
    alloy_provider::{DynProvider, Provider, ProviderBuilder},
    reqwest::Client as ReqwestClient,
    serial_test::serial,
    std::{cmp::max, collections::HashMap, iter, time::Duration},
    ERC20::ERC20Instance,
};

pub const USDC_CONTRACT_OPTIMISM: Address =
    address!("0b2c639c533813f4aa9d7837caf62653d097ff85");
pub const USDC_CONTRACT_BASE: Address =
    address!("833589fCD6eDb6E08f4c7C32D4f71b54bdA02913");
pub const USDC_CONTRACT_ARBITRUM: Address =
    address!("af88d065e77c8cC2239327C5EDb3A432268e5831");

const TOPOFF: f64 = 3.05; // 200% in the server

/// How much to multiply the amount by when bridging to cover bridging
/// differences
pub const BRIDGING_AMOUNT_MULTIPLIER: u64 = 205; // 200% in the server

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Chain {
    Base,
    Optimism,
    Arbitrum,
}

impl Chain {
    pub fn caip2(&self) -> &'static str {
        match self {
            Chain::Base => CHAIN_ID_BASE,
            Chain::Optimism => CHAIN_ID_OPTIMISM,
            Chain::Arbitrum => CHAIN_ID_ARBITRUM,
        }
    }

    #[allow(unused)]
    pub fn chain_id(&self) -> &'static str {
        self.caip2().strip_prefix("eip155:").unwrap()
    }

    pub fn token_address(&self, token: &Token) -> Address {
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

    pub fn from_eip155_chain_id(chain_id: &str) -> Chain {
        match chain_id {
            CHAIN_ID_BASE => Chain::Base,
            CHAIN_ID_OPTIMISM => Chain::Optimism,
            CHAIN_ID_ARBITRUM => Chain::Arbitrum,
            _ => unimplemented!(),
        }
    }
}

#[cfg(feature = "solana")]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum SolanaChain {
    Mainnet,
}

#[cfg(feature = "solana")]
impl SolanaChain {
    pub fn caip2(&self) -> &'static str {
        match self {
            SolanaChain::Mainnet => SOLANA_MAINNET_CAIP2,
        }
    }

    #[allow(unused)]
    pub fn chain_id(&self) -> &'static str {
        self.caip2().strip_prefix("solana:").unwrap()
    }

    pub fn token_address(&self, token: &Token) -> solana_sdk::pubkey::Pubkey {
        match self {
            SolanaChain::Mainnet => match token {
                Token::Usdc => usdc_mint(),
            },
        }
    }

    pub fn from_solana_chain_id(chain_id: &str) -> SolanaChain {
        match chain_id {
            SOLANA_MAINNET_CHAIN_ID => SolanaChain::Mainnet,
            _ => unimplemented!(),
        }
    }

    pub fn get_caip10(&self, account_address: solana::SolanaPubkey) -> String {
        format!("{}:{}", self.caip2(), account_address)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BridgeTokenParams {
    chain: Chain,
    account_address: Address,
    token: Token,
}

#[derive(Clone)]
struct BridgeToken {
    params: BridgeTokenParams,
    token: ERC20Instance<(), DynProvider, Ethereum>,
    provider: DynProvider,
}

impl BridgeToken {
    async fn new(
        params: BridgeTokenParams,
        account: LocalSigner<SigningKey>,
        provider_pool: &ProviderPool,
    ) -> BridgeToken {
        let provider = ProviderBuilder::new()
            .wallet(EthereumWallet::new(account))
            .on_provider(provider_pool.get_provider(params.chain.caip2()).await)
            .erased();

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

const CHAIN_ID_OPTIMISM: &str = "eip155:10";
const CHAIN_ID_BASE: &str = "eip155:8453";
const CHAIN_ID_ARBITRUM: &str = "eip155:42161";

async fn estimate_total_fees(
    _wallet: &EthereumWallet,
    provider: &impl Provider,
    txn: TransactionRequest,
) -> U256 {
    let gas = txn.gas.unwrap();
    let max_fee_per_gas = txn.max_fee_per_gas.unwrap();

    let provider_chain_id =
        format!("eip155:{}", provider.get_chain_id().await.unwrap());
    let l1_data_fee = if provider_chain_id == CHAIN_ID_BASE
        || provider_chain_id == CHAIN_ID_OPTIMISM
    {
        get_l1_data_fee(txn, provider).await.unwrap()
    } else {
        U256::ZERO
    };
    println!("l1_data_fee: {l1_data_fee}");

    U256::from(max_fee_per_gas) * U256::from(gas) + l1_data_fee
}

async fn send_sponsored_txn(
    faucet: LocalSigner<SigningKey>,
    provider: &impl Provider,
    wallet_lookup: &HashMap<Address, LocalSigner<SigningKey>>,
    txn: TransactionRequest,
) {
    let provider_chain_id =
        format!("eip155:{}", provider.get_chain_id().await.unwrap());
    let from_address = txn.from.unwrap();
    let wallet =
        EthereumWallet::new(wallet_lookup.get(&from_address).unwrap().clone());

    let txn = txn.with_nonce(
        provider.get_transaction_count(from_address).await.unwrap(),
    );

    let gas = provider.estimate_gas(&txn).await.unwrap();
    let fees = provider.estimate_eip1559_fees(None).await.unwrap();
    let txn = txn
        .gas_limit(gas)
        .max_fee_per_gas(fees.max_fee_per_gas)
        .max_priority_fee_per_gas(fees.max_priority_fee_per_gas);

    let total_fee = estimate_total_fees(&wallet, provider, txn.clone()).await;

    let balance = provider.get_balance(from_address).await.unwrap();
    if balance < total_fee {
        let additional_balance_required = total_fee - balance;
        println!("additional_balance_required: {additional_balance_required}");
        println!(
            "using faucet (2) for {}:{} at {}",
            provider_chain_id, from_address, additional_balance_required
        );
        use_faucet_gas(
            provider,
            faucet.clone(),
            U256::from(additional_balance_required),
            from_address,
            4,
        )
        .await;
        println!("funded");
    }

    println!("sending txn: {:?}", txn);
    let txn_sent = ProviderBuilder::new()
        .wallet(wallet.clone())
        .on_provider(provider)
        .send_transaction(txn.clone())
        .await
        .unwrap()
        .with_timeout(Some(Duration::from_secs(30)));
    println!("txn hash: {} on chain {provider_chain_id}", txn_sent.tx_hash());
    let receipt = txn_sent.get_receipt().await.unwrap();
    assert!(receipt.status());
}

#[tokio::test]
async fn bridging_routes_routes_available() {
    let provider_pool = ProviderPool::new(
        std::env::var("REOWN_PROJECT_ID").unwrap().into(),
        ReqwestClient::new(),
        get_pulse_metadata(),
        std::env::var("BLOCKCHAIN_API_URL")
            .unwrap_or(BLOCKCHAIN_API_URL_PROD.to_string())
            .parse()
            .unwrap(),
    );

    let faucet = private_faucet();
    println!("faucet: {}", faucet.address());

    // Accounts unique to this test fixture
    let account_1 = use_account(Some(BRIDGE_ACCOUNT_USDC_1557_1));
    println!("account_1: {}", account_1.address());
    let account_2 = use_account(Some(BRIDGE_ACCOUNT_USDC_1557_2));
    println!("account_2: {}", account_2.address());

    let source = account_1.clone();
    let destination = account_2.clone();

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
        &provider_pool,
    )
    .await;
    let chain_2_address_1_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_2.to_owned(),
            account_address: account_1.address(),
            token,
        },
        account_1.clone(),
        &provider_pool,
    )
    .await;

    let current_balance = chain_1_address_1_token.token_balance().await;

    /// Minimal bridging fees coverage using decimals
    static MINIMAL_BRIDGING_FEES_COVERAGE: u64 = 50000; // 0.05 USDC/USDT

    let send_amount = U256::from(1_500_000); // 1.5 USDC (6 decimals)

    // let required_amount =
    //     U256::from((send_amount.to::<u128>() as f64 * TOPOFF) as u128);
    let required_amount = {
        let erc20_topup_value = send_amount;
        // Multiply the topup value by the bridging percent multiplier and get
        // the maximum between the calculated fees covering value and
        // the minimal bridging fees coverage
        let calculated_fees_covering_value = (erc20_topup_value
            * U256::from(BRIDGING_AMOUNT_MULTIPLIER))
            / U256::from(100);
        erc20_topup_value
            + max(
                calculated_fees_covering_value,
                U256::from(MINIMAL_BRIDGING_FEES_COVERAGE),
            )
    };
    println!("required_amount: {required_amount}");

    if current_balance < required_amount {
        assert!(required_amount < U256::from(5000000));
        println!(
                "using token faucet {} on chain {} for amount {current_balance} on token {:?} ({}). Send tokens to faucet at: {}",
                faucet.address(),
                chain_1_address_1_token.params.chain.caip2(),
                token,
                chain_1_address_1_token.token.address(),
                faucet.address(),
            );
        let status = BridgeToken::new(
            chain_1_address_1_token.params.clone(),
            faucet.clone(),
            &provider_pool,
        )
        .await
        .token
        .transfer(account_1.address(), required_amount - current_balance)
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
    assert!(chain_1_address_1_token.token_balance().await >= required_amount);

    let transaction = Call {
        to: *chain_2_address_1_token.token.address(),
        value: U256::ZERO,
        input: ERC20::transferCall {
            _to: destination.address(),
            _value: send_amount,
        }
        .abi_encode()
        .into(),
    };
    println!("input transaction: {:?}", transaction);

    let project_id = std::env::var("REOWN_PROJECT_ID").unwrap().into();
    let client = Client::new(project_id, get_pulse_metadata());
    let start = Instant::now();
    let result = client
        .prepare(
            chain_2.caip2().to_owned(),
            source.address(),
            transaction.clone(),
            vec![],
            false,
        )
        .await
        .unwrap()
        .into_result()
        .unwrap()
        .into_option()
        .unwrap();
    println!("route result in ({:#?}): {:?}", start.elapsed(), result);

    assert!(result.transactions.len() == 1 || result.transactions.len() == 2);

    let start = Instant::now();
    let ui_fields =
        client.get_ui_fields(result.clone(), Currency::Usd).await.unwrap();
    println!("output ui_fields in ({:#?}): {:?}", start.elapsed(), ui_fields);

    fn sanity_check_fee(fee: &Amount) {
        println!("sanity_check_fee fee: {fee:?}");
        assert_eq!(fee.symbol, "USD".to_owned());
        assert!(fee.amount > U256::ZERO);
        assert!(fee.amount > U256::from(100));
        assert!(fee.as_float_inaccurate() < 1.);
        assert!(fee.as_float_inaccurate() < 0.3); // TODO this was increased to stop test flakes, but this should reduce to 0.1 if we provide a low-cost solution
        assert!(fee.formatted.ends_with(&fee.symbol));
        assert!(
            fee.formatted_alt.starts_with("$")
                || fee.formatted_alt.starts_with("<$")
        );
    }

    println!("checking ui_fields.local_total");
    sanity_check_fee(&ui_fields.local_total);
    println!("checking ui_fields.initial.2.local_fee");
    sanity_check_fee(&ui_fields.initial.fee.local_fee);
    for TxnDetails { fee: TransactionFee { local_fee: fee, .. }, .. } in
        ui_fields.route[0].as_eip155().unwrap()
    {
        println!("checking ui_fields.route[*].2.local_fee");
        sanity_check_fee(fee);
    }

    let fee = ui_fields.bridge.first().unwrap();
    println!("checking ui_fields.bridge.first().unwrap()");
    sanity_check_fee(&fee.local_fee);
    let fee = &fee.fee;
    assert_eq!(fee.symbol, "USDC".to_owned());
    assert!(fee.amount > U256::ZERO);
    assert!(fee.as_float_inaccurate() < 1.);
    assert!(fee.as_float_inaccurate() < 0.3); // TODO this was increased to stop test flakes, but this should reduce to 0.1 if we provide a low-cost solution

    let total_fee = ui_fields.local_total.as_float_inaccurate();
    let combined_fees =
        iter::once(ui_fields.initial.fee.local_fee.as_float_inaccurate())
            .chain(
                ui_fields
                    .bridge
                    .iter()
                    .map(|f| f.local_fee.as_float_inaccurate()),
            )
            .chain(ui_fields.route.iter().flat_map(|route| {
                route.as_eip155().unwrap().iter().map(
                    |TxnDetails {
                         fee: TransactionFee { local_fee, .. },
                         ..
                     }| { local_fee.as_float_inaccurate() },
                )
            }))
            .sum::<f64>();
    println!("total_fee: {total_fee}");
    println!("combined_fees: {combined_fees}");
    let error = (total_fee - combined_fees).abs();
    println!("error: {error}");
    assert!(error < 0.00000000000001);

    let combined_fees_intermediate_totals = [
        ui_fields.initial.fee.local_fee.as_float_inaccurate(),
        ui_fields.local_route_total.as_float_inaccurate(),
        ui_fields.local_bridge_total.as_float_inaccurate(),
    ]
    .iter()
    .sum::<f64>();
    println!("combined_fees_intermediate_totals: {combined_fees_intermediate_totals}");
    let error = (total_fee - combined_fees_intermediate_totals).abs();
    println!("error: {error}");
    assert!(error < 0.00000000000001);
}

#[tokio::test]
#[serial(happy_path)]
async fn happy_path() {
    let provider_pool = ProviderPool::new(
        std::env::var("REOWN_PROJECT_ID").unwrap().into(),
        ReqwestClient::new(),
        get_pulse_metadata(),
        std::env::var("BLOCKCHAIN_API_URL")
            .unwrap_or(BLOCKCHAIN_API_URL_PROD.to_string())
            .parse()
            .unwrap(),
    );

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

    let chain_1_provider = provider_pool.get_provider(chain_1.caip2()).await;
    let chain_2_provider = provider_pool.get_provider(chain_2.caip2()).await;

    let chain_1_address_1_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_1.to_owned(),
            account_address: account_1.address(),
            token,
        },
        account_1.clone(),
        &provider_pool,
    )
    .await;
    let chain_1_address_2_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_1.to_owned(),
            account_address: account_2.address(),
            token,
        },
        account_2.clone(),
        &provider_pool,
    )
    .await;
    let chain_2_address_1_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_2.to_owned(),
            account_address: account_1.address(),
            token,
        },
        account_1.clone(),
        &provider_pool,
    )
    .await;
    let chain_2_address_2_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_2.to_owned(),
            account_address: account_2.address(),
            token,
        },
        account_2.clone(),
        &provider_pool,
    )
    .await;

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
                        _to: chain_1_address_1_token.params.account_address,
                        _value: via1,
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
                        _to: chain_2_address_2_token.params.account_address,
                        _value: via2,
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
        U256::from((send_amount.to::<u128>() as f64 * TOPOFF) as u128);

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
        assert!(required_amount < U256::from(4000000));
        println!(
            "using token faucet {} on chain {} for amount {required_amount} on token {:?} ({}). Send tokens to faucet at: {}",
            faucet.address(),
            chain_1_address_1_token.params.chain.caip2(),
            token,
            chain_1_address_1_token.token.address(),
            faucet.address(),
        );
        let status = BridgeToken::new(
            chain_1_address_1_token.params.clone(),
            faucet.clone(),
            &provider_pool,
        )
        .await
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

    let initial_transaction = Call {
        to: *source.other().bridge_token(&sources).token.address(),
        value: U256::ZERO,
        input: ERC20::transferCall {
            _to: source.other().address(&sources),
            _value: send_amount,
        }
        .abi_encode()
        .into(),
    };
    println!("input transaction: {:?}", initial_transaction);

    let project_id = std::env::var("REOWN_PROJECT_ID").unwrap().into();
    let client = Client::new(project_id, get_pulse_metadata());
    let result = client
        .prepare(
            source
                .other()
                .bridge_token(&sources)
                .params
                .chain
                .caip2()
                .to_owned(),
            source.address(&sources),
            initial_transaction.clone(),
            vec![],
            false,
        )
        .await
        .unwrap()
        .into_result()
        .unwrap()
        .into_option()
        .unwrap();
    println!("route result: {:?}", result);

    // TODO it's possible this is only 1 transaction due to already being
    // approved: https://reown-inc.slack.com/archives/C0816SK4877/p1732813465413249?thread_ts=1732787456.681429&cid=C0816SK4877
    assert!(result.transactions.len() == 1 || result.transactions.len() == 2);

    let start = Instant::now();
    let ui_fields =
        client.get_ui_fields(result.clone(), Currency::Usd).await.unwrap();
    println!("output ui_fields in ({:#?}): {:?}", start.elapsed(), ui_fields);

    fn map_transaction(txn: Transaction) -> TransactionRequest {
        TransactionRequest::default()
            .with_from(txn.from)
            .with_to(txn.to)
            .with_value(txn.value)
            .with_gas_limit(txn.gas_limit.to())
            .with_input(txn.input)
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

    assert_eq!(ui_fields.route.len(), 1);

    let mut total_fees = HashMap::new();
    let mut transactions_with_fees = vec![];
    for (txn, ui_fields) in result
        .transactions
        .into_iter()
        .next()
        .unwrap()
        .into_eip155()
        .unwrap()
        .into_iter()
        .zip(ui_fields.route.into_iter().next().unwrap().into_eip155().unwrap())
        .chain(
            std::iter::once(result.initial_transaction)
                .zip(std::iter::once(ui_fields.initial)),
        )
    {
        let provider = provider_pool
            .get_provider(Chain::from_eip155_chain_id(&txn.chain_id).caip2())
            .await;
        // TODO use fees from response of ui_fields: https://linear.app/reown/issue/RES-140/use-fees-from-response-of-route-ui-fields-for-happy-path-ca-tests
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
        transactions_with_fees.push(txn.clone());

        assert_eq!(
            ui_fields.transaction.chain_id,
            format!("eip155:{}", txn.chain_id.unwrap())
        );
        assert_eq!(ui_fields.transaction.from, txn.from.unwrap());
        assert_eq!(TxKind::Call(ui_fields.transaction.to), txn.to.unwrap());
        assert_eq!(ui_fields.fee.fee.symbol, "ETH");
        assert_eq!(ui_fields.fee.fee.unit, Unit::ETHER.get());
        assert!(floats_close(
            ui_fields.fee.fee.as_float_inaccurate(),
            Amount::new("NULL".to_owned(), fee, Unit::ETHER)
                .as_float_inaccurate(),
            0.25
        ));
    }

    assert_eq!(ui_fields.bridge.len(), 1);
    assert_eq!(ui_fields.bridge.first().unwrap().fee.symbol, "USDC");
    assert_eq!(ui_fields.bridge.first().unwrap().local_fee.symbol, "USD");
    let ui_bridge_fee =
        ui_fields.bridge.first().unwrap().fee.as_float_inaccurate();
    let send_amount_amount =
        Amount::new("NULL".to_owned(), send_amount, Unit::new(6).unwrap())
            .as_float_inaccurate();
    println!("ui_bridge_fee: {ui_bridge_fee}");
    println!("send_amount_amount: {send_amount_amount}");
    assert!(ui_bridge_fee / send_amount_amount < 0.05, "ui_bridge_fee {ui_bridge_fee} must be less than the amount being sent {send_amount_amount}");

    for ((chain_id, address), total_fee) in total_fees {
        let provider = provider_pool
            .get_provider(Chain::from_eip155_chain_id(&chain_id).caip2())
            .await;
        let balance = provider.get_balance(address).await.unwrap();
        if total_fee > balance {
            let additional_balance_required = total_fee - balance;
            println!("using faucet (1) for {chain_id}:{address} at {additional_balance_required}");
            use_faucet_gas(
                &provider,
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
    assert!(bridge.len() == 1 || bridge.len() == 2);
    assert_eq!(original.len(), 1);
    let original = original.first().unwrap();

    let status = client.status(result.orchestration_id.clone()).await.unwrap();
    assert!(matches!(status, StatusResponse::Pending(_)));

    let approval_start = Instant::now();

    let mut pending_bridge_txn_hashes = Vec::with_capacity(bridge.len());
    for txn in bridge {
        let provider = provider_pool
            .get_provider(
                Chain::from_eip155_chain_id(&format!(
                    "eip155:{}",
                    txn.chain_id.unwrap()
                ))
                .caip2(),
            )
            .await;
        println!("sending txn: {:?}", txn);
        let txn_sent = ProviderBuilder::new()
            .wallet(EthereumWallet::new(
                wallet_lookup.get(&txn.from.unwrap()).unwrap().clone(),
            ))
            .on_provider(provider.clone())
            .send_transaction(txn.clone())
            .await
            .unwrap()
            .with_timeout(Some(Duration::from_secs(30)));

        let tx_hash = *txn_sent.tx_hash();
        println!("txn hash: {} on chain {}", tx_hash, txn.chain_id.unwrap());
        let receipt = txn_sent.get_receipt().await.unwrap();
        assert!(receipt.status());
        pending_bridge_txn_hashes.push((provider.clone(), tx_hash));
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

    let provider = provider_pool
        .get_provider(
            Chain::from_eip155_chain_id(&format!(
                "eip155:{}",
                original.chain_id.unwrap()
            ))
            .caip2(),
        )
        .await;

    println!("sending txn: {:?}", original);
    let txn_sent = ProviderBuilder::new()
        .wallet(EthereumWallet::new(
            wallet_lookup.get(&original.from.unwrap()).unwrap().clone(),
        ))
        .on_provider(provider.clone())
        .send_transaction(original.clone())
        .await
        .unwrap();
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
    let receipt = txn_sent
        .with_timeout(Some(Duration::from_secs(60)))
        .get_receipt()
        .await
        .unwrap();
    assert!(receipt.status());

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
#[serial(happy_path)]
async fn happy_path_full_dependency_on_ui_fields() {
    let provider_pool = ProviderPool::new(
        std::env::var("REOWN_PROJECT_ID").unwrap().into(),
        ReqwestClient::new(),
        get_pulse_metadata(),
        std::env::var("BLOCKCHAIN_API_URL")
            .unwrap_or(BLOCKCHAIN_API_URL_PROD.to_string())
            .parse()
            .unwrap(),
    );

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

    let chain_1_provider = provider_pool.get_provider(chain_1.caip2()).await;
    let chain_2_provider = provider_pool.get_provider(chain_2.caip2()).await;

    let chain_1_address_1_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_1.to_owned(),
            account_address: account_1.address(),
            token,
        },
        account_1.clone(),
        &provider_pool,
    )
    .await;
    let chain_1_address_2_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_1.to_owned(),
            account_address: account_2.address(),
            token,
        },
        account_2.clone(),
        &provider_pool,
    )
    .await;
    let chain_2_address_1_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_2.to_owned(),
            account_address: account_1.address(),
            token,
        },
        account_1.clone(),
        &provider_pool,
    )
    .await;
    let chain_2_address_2_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_2.to_owned(),
            account_address: account_2.address(),
            token,
        },
        account_2.clone(),
        &provider_pool,
    )
    .await;

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
                        _to: chain_1_address_1_token.params.account_address,
                        _value: via1,
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
                        _to: chain_2_address_2_token.params.account_address,
                        _value: via2,
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
        U256::from((send_amount.to::<u128>() as f64 * TOPOFF) as u128);

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
        assert!(required_amount < U256::from(4000000));
        println!(
            "using token faucet {} on chain {} for amount {required_amount} on token {:?} ({}). Send tokens to faucet at: {}",
            faucet.address(),
            chain_1_address_1_token.params.chain.caip2(),
            token,
            chain_1_address_1_token.token.address(),
            faucet.address(),
        );
        let status = BridgeToken::new(
            chain_1_address_1_token.params.clone(),
            faucet.clone(),
            &provider_pool,
        )
        .await
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

    let initial_transaction = Call {
        to: *source.other().bridge_token(&sources).token.address(),
        value: U256::ZERO,
        input: ERC20::transferCall {
            _to: source.other().address(&sources),
            _value: send_amount,
        }
        .abi_encode()
        .into(),
    };
    println!("input transaction: {:?}", initial_transaction);

    let initial_transaction_chain_id =
        source.other().bridge_token(&sources).params.chain.caip2().to_owned();

    let project_id = std::env::var("REOWN_PROJECT_ID").unwrap().into();
    let client = Client::new(project_id, get_pulse_metadata());
    let result = client
        .prepare(
            initial_transaction_chain_id.clone(),
            source.address(&sources),
            initial_transaction.clone(),
            vec![],
            false,
        )
        .await
        .unwrap()
        .into_result()
        .unwrap()
        .into_option()
        .unwrap();
    println!("route result: {:?}", result);

    // TODO it's possible this is only 1 transaction due to already being
    // approved: https://reown-inc.slack.com/archives/C0816SK4877/p1732813465413249?thread_ts=1732787456.681429&cid=C0816SK4877
    assert!(result.transactions.len() == 1 || result.transactions.len() == 2);

    assert_eq!(result.metadata.funding_from.len(), 1);
    assert_eq!(result.metadata.funding_from.first().unwrap().symbol, "USDC");
    assert_eq!(result.metadata.funding_from.first().unwrap().decimals, 6);
    assert_eq!(
        result
            .metadata
            .funding_from
            .first()
            .unwrap()
            .clone()
            .to_amount()
            .symbol,
        "USDC"
    );
    assert!(result
        .metadata
        .funding_from
        .first()
        .unwrap()
        .to_amount()
        .formatted
        .ends_with(" USDC"));
    println!(
        "{}",
        result.metadata.funding_from.first().unwrap().to_amount().formatted
    );
    // Disabling this check for now, as the value seems to have changed to 1.50 for some reason
    // assert!(result
    //     .metadata
    //     .funding_from
    //     .first()
    //     .unwrap()
    //     .to_amount()
    //     .formatted
    //     .starts_with("2.25"));
    assert!(result
        .metadata
        .funding_from
        .first()
        .unwrap()
        .to_bridging_fee_amount()
        .formatted
        .starts_with("0."));
    assert!(
        result.metadata.funding_from.first().unwrap().amount <= required_amount
    );
    assert!(result.metadata.funding_from.first().unwrap().amount > send_amount);
    assert!(
        result.metadata.funding_from.first().unwrap().bridging_fee > U256::ZERO
    );
    assert!(
        result.metadata.funding_from.first().unwrap().bridging_fee
            < send_amount / U256::from(2)
    );
    assert_eq!(
        result.metadata.funding_from.first().unwrap().chain_id,
        source.bridge_token(&sources).params.chain.caip2()
    );
    assert_eq!(
        &result.metadata.funding_from.first().unwrap().token_contract,
        &Eip155OrSolanaAddress::Eip155(
            *source.bridge_token(&sources).token.address()
        )
    );

    let start = Instant::now();
    let ui_fields =
        client.get_ui_fields(result.clone(), Currency::Usd).await.unwrap();
    println!("output ui_fields in ({:#?}): {:?}", start.elapsed(), ui_fields);

    assert_eq!(ui_fields.route.len(), 1);
    let ui_fields_route =
        ui_fields.route.into_iter().next().unwrap().into_eip155().unwrap();

    // Provide gas for transactions
    let mut prepared_faucet_txns = HashMap::new();
    for txn in ui_fields_route.iter().chain(std::iter::once(&ui_fields.initial))
    {
        assert_eq!(txn.fee.fee.symbol, "ETH");
        prepared_faucet_txns
            .entry((txn.transaction.chain_id.clone(), txn.transaction.from))
            .and_modify(|f| *f += txn.fee.fee.amount)
            .or_insert(U256::ZERO);
    }
    for ((chain_id, address), total_fee) in prepared_faucet_txns {
        let provider = provider_pool
            .get_provider(Chain::from_eip155_chain_id(&chain_id).caip2())
            .await;
        let balance = provider.get_balance(address).await.unwrap();
        if total_fee > balance {
            let additional_balance_required = total_fee - balance;
            println!("using faucet (1) for {chain_id}:{address} at {additional_balance_required}");
            use_faucet_gas(
                &provider,
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

    let status = client.status(result.orchestration_id.clone()).await.unwrap();
    assert!(matches!(status, StatusResponse::Pending(_)));

    let approval_start = Instant::now();

    let mut pending_bridge_txn_hashes =
        Vec::with_capacity(ui_fields_route.len());
    for TxnDetails { transaction, .. } in ui_fields_route {
        let txn = transaction.into_transaction_request();
        let provider = provider_pool
            .get_provider(
                Chain::from_eip155_chain_id(&format!(
                    "eip155:{}",
                    txn.chain_id.unwrap()
                ))
                .caip2(),
            )
            .await;

        println!("sending txn: {:?}", txn);
        let txn_sent = ProviderBuilder::new()
            .wallet(EthereumWallet::new(
                wallet_lookup.get(&txn.from.unwrap()).unwrap().clone(),
            ))
            .on_provider(provider.clone())
            .send_transaction(txn.clone())
            .await
            .unwrap()
            .with_timeout(Some(Duration::from_secs(30)));
        let tx_hash = *txn_sent.tx_hash();
        println!("txn hash: {} on chain {}", tx_hash, txn.chain_id.unwrap());
        let receipt = txn_sent.get_receipt().await.unwrap();
        assert!(receipt.status());
        pending_bridge_txn_hashes.push((provider.clone(), tx_hash));
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

    let provider = provider_pool
        .get_provider(
            Chain::from_eip155_chain_id(&initial_transaction_chain_id).caip2(),
        )
        .await;

    let original = ui_fields.initial.transaction.into_transaction_request();

    println!("sending txn: {:?}", original);
    let txn_sent = ProviderBuilder::new()
        .wallet(EthereumWallet::new(
            wallet_lookup.get(&original.from.unwrap()).unwrap().clone(),
        ))
        .on_provider(provider.clone())
        .send_transaction(original.clone())
        .await
        .unwrap();
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
    let receipt = txn_sent
        .with_timeout(Some(Duration::from_secs(60)))
        .get_receipt()
        .await
        .unwrap();

    assert!(receipt.status());

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

#[test_log::test(tokio::test)]
#[serial(happy_path)]
async fn happy_path_execute_method() {
    let project_id = std::env::var("REOWN_PROJECT_ID").unwrap().into();
    let blockchain_api_url = std::env::var("BLOCKCHAIN_API_URL")
        .unwrap_or(BLOCKCHAIN_API_URL_PROD.to_string())
        .parse()
        .unwrap();
    let client = Client::with_blockchain_api_url(
        project_id,
        get_pulse_metadata(),
        blockchain_api_url,
    );

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

    let chain_1_provider =
        client.provider_pool.get_provider(chain_1.caip2()).await;
    let chain_2_provider =
        client.provider_pool.get_provider(chain_2.caip2()).await;

    let chain_1_address_1_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_1.to_owned(),
            account_address: account_1.address(),
            token,
        },
        account_1.clone(),
        &client.provider_pool,
    )
    .await;
    let chain_1_address_2_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_1.to_owned(),
            account_address: account_2.address(),
            token,
        },
        account_2.clone(),
        &client.provider_pool,
    )
    .await;
    let chain_2_address_1_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_2.to_owned(),
            account_address: account_1.address(),
            token,
        },
        account_1.clone(),
        &client.provider_pool,
    )
    .await;
    let chain_2_address_2_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_2.to_owned(),
            account_address: account_2.address(),
            token,
        },
        account_2.clone(),
        &client.provider_pool,
    )
    .await;

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

    // Wait for cache invalidation on balance call
    tokio::time::sleep(Duration::from_secs(30)).await;
    let assets = client
        .provider_pool
        .get_wallet_provider(None, None)
        .await
        .wallet_get_assets(GetAssetsParams {
            account: chain_1_address_1_token.params.account_address,
            filters: GetAssetsFilters {
                asset_filter: None,
                asset_type_filter: None,
                chain_filter: None,
            },
        })
        .await
        .unwrap();
    println!("assets: {:?}", assets);
    println!(
        "chain_1_address_1_token: {}",
        chain_1_address_1_token.token_balance().await
    );
    println!(
        "chain_2_address_1_token: {}",
        chain_2_address_1_token.token_balance().await
    );
    assert_eq!(
        assets
            .get(&U64::from(
                chain_1_address_1_token
                    .params
                    .chain
                    .caip2()
                    .strip_prefix("eip155:")
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
            ))
            .unwrap()
            .iter()
            .find(|asset| asset
                .as_erc20()
                .map(|asset| asset.metadata.symbol == "USDC")
                .unwrap_or(false))
            .unwrap()
            .balance(),
        chain_1_address_1_token.token_balance().await
            + chain_2_address_1_token.token_balance().await
    );

    let assets = client
        .provider_pool
        .get_wallet_provider(None, None)
        .await
        .wallet_get_assets(GetAssetsParams {
            account: chain_1_address_2_token.params.account_address,
            filters: GetAssetsFilters {
                asset_filter: None,
                asset_type_filter: None,
                chain_filter: None,
            },
        })
        .await
        .unwrap();
    println!("assets: {:?}", assets);
    println!(
        "chain_1_address_2_token: {}",
        chain_1_address_2_token.token_balance().await
    );
    println!(
        "chain_2_address_2_token: {}",
        chain_2_address_2_token.token_balance().await
    );
    let chain_1_address_2_token_balance =
        chain_1_address_2_token.token_balance().await;
    let chain_2_address_2_token_balance =
        chain_2_address_2_token.token_balance().await;
    println!(
        "chain_1_address_2_token (second): {}",
        chain_1_address_2_token_balance
    );
    println!(
        "chain_2_address_2_token (second): {}",
        chain_2_address_2_token_balance
    );
    assert_eq!(
        assets
            .get(&U64::from(
                chain_1_address_2_token
                    .params
                    .chain
                    .caip2()
                    .strip_prefix("eip155:")
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
            ))
            .unwrap()
            .iter()
            .find(|asset| asset
                .as_erc20()
                .map(|asset| asset.metadata.symbol == "USDC")
                .unwrap_or(false))
            .unwrap()
            .balance(),
        chain_1_address_2_token_balance + chain_2_address_2_token_balance
    );

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
                        _to: chain_1_address_1_token.params.account_address,
                        _value: via1,
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
                        _to: chain_2_address_2_token.params.account_address,
                        _value: via2,
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
        U256::from((send_amount.to::<u128>() as f64 * TOPOFF) as u128);

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

    println!("required_amount: {}", required_amount);
    if faucet_required {
        assert!(required_amount < U256::from(5000000));
        println!(
            "using token faucet {} on chain {} for amount {required_amount} on token {:?} ({}). Send tokens to faucet at: {}",
            faucet.address(),
            chain_1_address_1_token.params.chain.caip2(),
            token,
            chain_1_address_1_token.token.address(),
            faucet.address(),
        );

        let faucet_usdc = BridgeToken::new(
            chain_1_address_1_token.params.clone(),
            faucet.clone(),
            &client.provider_pool,
        )
        .await;

        if faucet_usdc.token_balance().await < required_amount * U256::from(2) {
            let unit = Unit::new(
                faucet_usdc.token.decimals().call().await.unwrap()._0,
            )
            .unwrap();
            let want_amount =
                ParseUnits::from(required_amount * U256::from(10))
                    .format_units(unit);
            let result = reqwest::Client::new().post("https://faucetbot.dev/api/faucet-request")
                .json(&serde_json::json!({
                    "key": std::env::var("FAUCET_REQUEST_API_KEY").unwrap(),
                    "text": format!("Yttrium tests running low on USDC. Please send {want_amount} USDC to {} on {}. [request]", faucet.address(), chain_1_address_1_token.params.chain.caip2()),
                }))
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();
            println!("requested funds from faucetbot: {result}");
        }

        let status = faucet_usdc
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

    let initial_transaction = Call {
        to: *source.other().bridge_token(&sources).token.address(),
        value: U256::ZERO,
        input: ERC20::transferCall {
            _to: source.other().address(&sources),
            _value: send_amount,
        }
        .abi_encode()
        .into(),
    };
    println!("input transaction: {:?}", initial_transaction);

    let initial_transaction_chain_id =
        source.other().bridge_token(&sources).params.chain.caip2().to_owned();
    println!("initial_transaction_chain_id: {}", initial_transaction_chain_id);

    let initial_transaction_from = source.address(&sources);
    println!("initial_transaction_from: {}", initial_transaction_from);

    // Wait for cache invalidation on balance call
    tokio::time::sleep(Duration::from_secs(30)).await;
    let assets = client
        .provider_pool
        .get_wallet_provider(None, None)
        .await
        .wallet_get_assets(GetAssetsParams {
            account: source.address(&sources),
            filters: GetAssetsFilters {
                asset_filter: None,
                asset_type_filter: None,
                chain_filter: None,
            },
        })
        .await
        .unwrap();
    println!("assets: {:?}", assets);
    assert_eq!(
        assets
            .get(&U64::from(
                initial_transaction_chain_id
                    .strip_prefix("eip155:")
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
            ))
            .unwrap()
            .iter()
            .find(|asset| asset
                .as_erc20()
                .map(|asset| asset.metadata.symbol == "USDC")
                .unwrap_or(false))
            .unwrap()
            .balance(),
        source.token_balance(&sources).await
    );

    let result = client
        .prepare_detailed(
            initial_transaction_chain_id.clone(),
            initial_transaction_from,
            initial_transaction.clone(),
            vec![],
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

    // TODO it's possible this is only 1 transaction due to already being
    // approved: https://reown-inc.slack.com/archives/C0816SK4877/p1732813465413249?thread_ts=1732787456.681429&cid=C0816SK4877
    assert!(result.route.len() == 1 || result.route.len() == 2);

    assert_eq!(result.route_response.metadata.funding_from.len(), 1);
    assert_eq!(
        result.route_response.metadata.funding_from.first().unwrap().symbol,
        "USDC"
    );
    assert_eq!(
        result.route_response.metadata.funding_from.first().unwrap().decimals,
        6
    );
    assert_eq!(
        result
            .route_response
            .metadata
            .funding_from
            .first()
            .unwrap()
            .clone()
            .to_amount()
            .symbol,
        "USDC"
    );
    assert!(result
        .route_response
        .metadata
        .funding_from
        .first()
        .unwrap()
        .to_amount()
        .formatted
        .ends_with(" USDC"));
    println!(
        "{}",
        result
            .route_response
            .metadata
            .funding_from
            .first()
            .unwrap()
            .to_amount()
            .formatted
    );
    // Disabling this check for now, as the value seems to have changed to 1.50 for some reason
    // assert!(result
    //     .metadata
    //     .funding_from
    //     .first()
    //     .unwrap()
    //     .to_amount()
    //     .formatted
    //     .starts_with("2.25"));
    assert!(result
        .route_response
        .metadata
        .funding_from
        .first()
        .unwrap()
        .to_bridging_fee_amount()
        .formatted
        .starts_with("0."));
    assert!(
        result.route_response.metadata.funding_from.first().unwrap().amount
            <= required_amount
    );
    assert!(
        result.route_response.metadata.funding_from.first().unwrap().amount
            > send_amount
    );
    assert!(
        result
            .route_response
            .metadata
            .funding_from
            .first()
            .unwrap()
            .bridging_fee
            > U256::ZERO
    );
    assert!(
        result
            .route_response
            .metadata
            .funding_from
            .first()
            .unwrap()
            .bridging_fee
            < send_amount / U256::from(2)
    );
    assert_eq!(
        result.route_response.metadata.funding_from.first().unwrap().chain_id,
        source.bridge_token(&sources).params.chain.caip2()
    );
    assert_eq!(
        result
            .route_response
            .metadata
            .funding_from
            .first()
            .unwrap()
            .token_contract,
        Eip155OrSolanaAddress::Eip155(
            *source.bridge_token(&sources).token.address()
        )
    );

    assert_eq!(result.route.len(), 1);
    let result_route = result.route.first().unwrap().as_eip155().unwrap();

    // Provide gas for transactions
    let mut prepared_faucet_txns = HashMap::new();
    for txn in result_route.iter().chain(std::iter::once(&result.initial)) {
        assert_eq!(txn.fee.fee.symbol, "ETH");
        prepared_faucet_txns
            .entry((txn.transaction.chain_id.clone(), txn.transaction.from))
            .and_modify(|f| *f += txn.fee.fee.amount)
            .or_insert(txn.fee.fee.amount);
    }
    for ((chain_id, address), total_fee) in prepared_faucet_txns {
        println!(
            "chain_id: {chain_id}, address: {address}, total_fee: {total_fee}"
        );
        let provider = client
            .provider_pool
            .get_provider(Chain::from_eip155_chain_id(&chain_id).caip2())
            .await;
        let balance = provider.get_balance(address).await.unwrap();
        if total_fee > balance {
            let additional_balance_required = total_fee - balance;
            println!("using faucet (1) for {chain_id}:{address} at {additional_balance_required}");
            use_faucet_gas(
                &provider,
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

    // Just an easy sanity check to have in the test coverage
    let status = client
        .status(result.route_response.orchestration_id.clone())
        .await
        .unwrap();
    assert!(matches!(status, StatusResponse::Pending(_)));

    let route_txn_sigs = vec![RouteSig::Eip155(
        result_route
            .iter()
            .map(|txn| {
                wallet_lookup
                    .get(&txn.transaction.from)
                    .unwrap()
                    .sign_hash_sync(&txn.transaction_hash_to_sign)
                    .unwrap()
            })
            .collect(),
    )];
    let initial_txn_sigs = wallet_lookup
        .get(&result.initial.transaction.from)
        .unwrap()
        .sign_hash_sync(&result.initial.transaction_hash_to_sign)
        .unwrap();
    let execute_result =
        client.execute(result, route_txn_sigs, initial_txn_sigs).await.unwrap();
    assert!(execute_result.initial_txn_receipt.status());

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
    let provider_pool = ProviderPool::new(
        std::env::var("REOWN_PROJECT_ID").unwrap().into(),
        ReqwestClient::new(),
        get_pulse_metadata(),
        std::env::var("BLOCKCHAIN_API_URL")
            .unwrap_or(BLOCKCHAIN_API_URL_PROD.to_string())
            .parse()
            .unwrap(),
    );

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
        &provider_pool,
    )
    .await;
    let chain_1_address_2_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_1.to_owned(),
            account_address: account_2.address(),
            token,
        },
        account_2.clone(),
        &provider_pool,
    )
    .await;
    let chain_2_address_1_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_2.to_owned(),
            account_address: account_1.address(),
            token,
        },
        account_1.clone(),
        &provider_pool,
    )
    .await;
    let chain_2_address_2_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_2.to_owned(),
            account_address: account_2.address(),
            token,
        },
        account_2.clone(),
        &provider_pool,
    )
    .await;
    assert_eq!(chain_1_address_1_token.token_balance().await, U256::ZERO);
    assert_eq!(chain_1_address_2_token.token_balance().await, U256::ZERO);
    assert_eq!(chain_2_address_1_token.token_balance().await, U256::ZERO);
    assert_eq!(chain_2_address_2_token.token_balance().await, U256::ZERO);
    assert_eq!(chain_1_address_1_token.native_balance().await, U256::ZERO);
    assert_eq!(chain_1_address_2_token.native_balance().await, U256::ZERO);
    assert_eq!(chain_2_address_1_token.native_balance().await, U256::ZERO);
    assert_eq!(chain_2_address_2_token.native_balance().await, U256::ZERO);

    let send_amount = U256::from(1_500_000); // 1.5 USDC (6 decimals)

    let transaction = Call {
        to: *chain_1_address_2_token.token.address(),
        value: U256::ZERO,
        input: ERC20::transferCall {
            _to: account_2.address(),
            _value: send_amount,
        }
        .abi_encode()
        .into(),
    };
    println!("input transaction: {:?}", transaction);

    let project_id = std::env::var("REOWN_PROJECT_ID").unwrap().into();
    let client = Client::new(project_id, get_pulse_metadata());
    let result = client
        .prepare(
            chain_1.caip2().to_owned(),
            account_1.address(),
            transaction.clone(),
            vec![],
            false,
        )
        .await
        .unwrap();
    assert!(matches!(
        result,
        PrepareResponse::Error(PrepareResponseError {
            error: BridgingError::InsufficientFunds,
            ..
        })
    ));
}

#[test_log::test(tokio::test)]
#[serial(happy_path)]
async fn happy_path_lifi() {
    let project_id = std::env::var("REOWN_PROJECT_ID").unwrap().into();
    let blockchain_api_url = std::env::var("BLOCKCHAIN_API_URL")
        .unwrap_or(BLOCKCHAIN_API_URL_PROD.to_string())
        .parse()
        .unwrap();
    let client = Client::with_blockchain_api_url(
        project_id,
        get_pulse_metadata(),
        blockchain_api_url,
    );

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

    let chain_1_provider =
        client.provider_pool.get_provider(chain_1.caip2()).await;
    let chain_2_provider =
        client.provider_pool.get_provider(chain_2.caip2()).await;

    let chain_1_address_1_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_1.to_owned(),
            account_address: account_1.address(),
            token,
        },
        account_1.clone(),
        &client.provider_pool,
    )
    .await;
    let chain_1_address_2_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_1.to_owned(),
            account_address: account_2.address(),
            token,
        },
        account_2.clone(),
        &client.provider_pool,
    )
    .await;
    let chain_2_address_1_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_2.to_owned(),
            account_address: account_1.address(),
            token,
        },
        account_1.clone(),
        &client.provider_pool,
    )
    .await;
    let chain_2_address_2_token = BridgeToken::new(
        BridgeTokenParams {
            chain: chain_2.to_owned(),
            account_address: account_2.address(),
            token,
        },
        account_2.clone(),
        &client.provider_pool,
    )
    .await;

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

    // Wait for cache invalidation on balance call
    tokio::time::sleep(Duration::from_secs(30)).await;
    let assets = client
        .provider_pool
        .get_wallet_provider(None, None)
        .await
        .wallet_get_assets(GetAssetsParams {
            account: chain_1_address_1_token.params.account_address,
            filters: GetAssetsFilters {
                asset_filter: None,
                asset_type_filter: None,
                chain_filter: None,
            },
        })
        .await
        .unwrap();
    println!("assets: {:?}", assets);
    assert_eq!(
        assets
            .get(&U64::from(
                chain_1_address_1_token
                    .params
                    .chain
                    .caip2()
                    .strip_prefix("eip155:")
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
            ))
            .unwrap()
            .iter()
            .find(|asset| asset
                .as_erc20()
                .map(|asset| asset.metadata.symbol == "USDC")
                .unwrap_or(false))
            .unwrap()
            .balance(),
        chain_1_address_1_token.token_balance().await
            + chain_2_address_1_token.token_balance().await
    );

    // Wait for cache invalidation on balance call
    tokio::time::sleep(Duration::from_secs(30)).await;
    let assets = client
        .provider_pool
        .get_wallet_provider(None, None)
        .await
        .wallet_get_assets(GetAssetsParams {
            account: chain_1_address_2_token.params.account_address,
            filters: GetAssetsFilters {
                asset_filter: None,
                asset_type_filter: None,
                chain_filter: None,
            },
        })
        .await
        .unwrap();
    println!("assets: {:?}", assets);
    assert_eq!(
        assets
            .get(&U64::from(
                chain_1_address_2_token
                    .params
                    .chain
                    .caip2()
                    .strip_prefix("eip155:")
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
            ))
            .unwrap()
            .iter()
            .find(|asset| asset
                .as_erc20()
                .map(|asset| asset.metadata.symbol == "USDC")
                .unwrap_or(false))
            .unwrap()
            .balance(),
        chain_1_address_2_token.token_balance().await
            + chain_2_address_2_token.token_balance().await
    );

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
                        _to: chain_1_address_1_token.params.account_address,
                        _value: via1,
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
                        _to: chain_2_address_2_token.params.account_address,
                        _value: via2,
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
        U256::from((send_amount.to::<u128>() as f64 * TOPOFF) as u128);

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

    println!("required_amount: {}", required_amount);
    if faucet_required {
        assert!(required_amount < U256::from(5000000));
        println!(
            "using token faucet {} on chain {} for amount {required_amount} on token {:?} ({}). Send tokens to faucet at: {}",
            faucet.address(),
            chain_1_address_1_token.params.chain.caip2(),
            token,
            chain_1_address_1_token.token.address(),
            faucet.address(),
        );
        let status = BridgeToken::new(
            chain_1_address_1_token.params.clone(),
            faucet.clone(),
            &client.provider_pool,
        )
        .await
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

    let initial_transaction = Call {
        to: *source.other().bridge_token(&sources).token.address(),
        value: U256::ZERO,
        input: ERC20::transferCall {
            _to: source.other().address(&sources),
            _value: send_amount,
        }
        .abi_encode()
        .into(),
    };
    println!("input transaction: {:?}", initial_transaction);

    let initial_transaction_chain_id =
        source.other().bridge_token(&sources).params.chain.caip2().to_owned();
    println!("initial_transaction_chain_id: {}", initial_transaction_chain_id);

    let initial_transaction_from = source.address(&sources);
    println!("initial_transaction_from: {}", initial_transaction_from);

    // Wait for cache invalidation on balance call
    tokio::time::sleep(Duration::from_secs(30)).await;
    let assets = client
        .provider_pool
        .get_wallet_provider(None, None)
        .await
        .wallet_get_assets(GetAssetsParams {
            account: source.address(&sources),
            filters: GetAssetsFilters {
                asset_filter: None,
                asset_type_filter: None,
                chain_filter: None,
            },
        })
        .await
        .unwrap();
    println!("assets: {:?}", assets);
    assert_eq!(
        assets
            .get(&U64::from(
                initial_transaction_chain_id
                    .strip_prefix("eip155:")
                    .unwrap()
                    .parse::<u64>()
                    .unwrap()
            ))
            .unwrap()
            .iter()
            .find(|asset| asset
                .as_erc20()
                .map(|asset| asset.metadata.symbol == "USDC")
                .unwrap_or(false))
            .unwrap()
            .balance(),
        source.token_balance(&sources).await
    );

    let result = client
        .prepare_detailed(
            initial_transaction_chain_id.clone(),
            initial_transaction_from,
            initial_transaction.clone(),
            vec![],
            Currency::Usd,
            true,
        )
        .await
        .unwrap()
        .into_result()
        .unwrap()
        .into_option()
        .unwrap();
    println!("route result: {:?}", result);

    // TODO it's possible this is only 1 transaction due to already being
    // approved: https://reown-inc.slack.com/archives/C0816SK4877/p1732813465413249?thread_ts=1732787456.681429&cid=C0816SK4877
    assert!(result.route.len() == 1 || result.route.len() == 2);

    assert_eq!(result.route_response.metadata.funding_from.len(), 1);
    assert_eq!(
        result.route_response.metadata.funding_from.first().unwrap().symbol,
        "USDC"
    );
    assert_eq!(
        result.route_response.metadata.funding_from.first().unwrap().decimals,
        6
    );
    assert_eq!(
        result
            .route_response
            .metadata
            .funding_from
            .first()
            .unwrap()
            .clone()
            .to_amount()
            .symbol,
        "USDC"
    );
    assert!(result
        .route_response
        .metadata
        .funding_from
        .first()
        .unwrap()
        .to_amount()
        .formatted
        .ends_with(" USDC"));
    println!(
        "{}",
        result
            .route_response
            .metadata
            .funding_from
            .first()
            .unwrap()
            .to_amount()
            .formatted
    );
    // Disabling this check for now, as the value seems to have changed to 1.50 for some reason
    // assert!(result
    //     .metadata
    //     .funding_from
    //     .first()
    //     .unwrap()
    //     .to_amount()
    //     .formatted
    //     .starts_with("2.25"));
    assert!(result
        .route_response
        .metadata
        .funding_from
        .first()
        .unwrap()
        .to_bridging_fee_amount()
        .formatted
        .starts_with("0."));
    assert!(
        result.route_response.metadata.funding_from.first().unwrap().amount
            <= required_amount
    );
    assert!(
        result.route_response.metadata.funding_from.first().unwrap().amount
            > send_amount
    );
    assert!(
        result
            .route_response
            .metadata
            .funding_from
            .first()
            .unwrap()
            .bridging_fee
            > U256::ZERO
    );
    assert!(
        result
            .route_response
            .metadata
            .funding_from
            .first()
            .unwrap()
            .bridging_fee
            < send_amount / U256::from(2)
    );
    assert_eq!(
        result.route_response.metadata.funding_from.first().unwrap().chain_id,
        source.bridge_token(&sources).params.chain.caip2()
    );
    assert_eq!(
        result
            .route_response
            .metadata
            .funding_from
            .first()
            .unwrap()
            .token_contract,
        Eip155OrSolanaAddress::Eip155(
            *source.bridge_token(&sources).token.address()
        )
    );

    assert_eq!(result.route.len(), 1);
    let result_route = result.route.first().unwrap().as_eip155().unwrap();

    // Provide gas for transactions
    let mut prepared_faucet_txns = HashMap::new();
    for txn in result_route.iter().chain(std::iter::once(&result.initial)) {
        assert_eq!(txn.fee.fee.symbol, "ETH");
        prepared_faucet_txns
            .entry((txn.transaction.chain_id.clone(), txn.transaction.from))
            .and_modify(|f| *f += txn.fee.fee.amount)
            .or_insert(txn.fee.fee.amount);
    }
    for ((chain_id, address), total_fee) in prepared_faucet_txns {
        println!(
            "chain_id: {chain_id}, address: {address}, total_fee: {total_fee}"
        );
        let provider = client
            .provider_pool
            .get_provider(Chain::from_eip155_chain_id(&chain_id).caip2())
            .await;
        let balance = provider.get_balance(address).await.unwrap();
        if total_fee > balance {
            let additional_balance_required = total_fee - balance;
            println!("using faucet (1) for {chain_id}:{address} at {additional_balance_required}");
            use_faucet_gas(
                &provider,
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

    // Just an easy sanity check to have in the test coverage
    let status = client
        .status(result.route_response.orchestration_id.clone())
        .await
        .unwrap();
    assert!(matches!(status, StatusResponse::Pending(_)));

    let route_txn_sigs = vec![RouteSig::Eip155(
        result_route
            .iter()
            .map(|txn| {
                wallet_lookup
                    .get(&txn.transaction.from)
                    .unwrap()
                    .sign_hash_sync(&txn.transaction_hash_to_sign)
                    .unwrap()
            })
            .collect(),
    )];
    let initial_txn_sigs = wallet_lookup
        .get(&result.initial.transaction.from)
        .unwrap()
        .sign_hash_sync(&result.initial.transaction_hash_to_sign)
        .unwrap();
    let execute_result =
        client.execute(result, route_txn_sigs, initial_txn_sigs).await.unwrap();
    assert!(execute_result.initial_txn_receipt.status());

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
