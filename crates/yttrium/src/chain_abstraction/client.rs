use {
    super::{
        api::{
            fungible_price::{
                PriceRequestBody, PriceResponseBody,
                FUNGIBLE_PRICE_ENDPOINT_PATH, NATIVE_TOKEN_ADDRESS,
            },
            prepare::{
                CallOrCalls, PrepareRequest, PrepareRequestTransaction,
                PrepareResponse, PrepareResponseAvailable,
                PrepareResponseSuccess, RouteQueryParams, ROUTE_ENDPOINT_PATH,
            },
            status::{
                StatusQueryParams, StatusResponse, StatusResponseCompleted,
                STATUS_ENDPOINT_PATH,
            },
            Transaction,
        },
        currency::Currency,
        error::{
            ExecuteError, ExecuteErrorReason, PrepareDetailedError,
            PrepareDetailedResponse, PrepareDetailedResponseSuccess,
            PrepareError, StatusError, WaitForSuccessError,
        },
        pulse::{PulseMetadata, PULSE_SDK_TYPE},
        send_transaction::{send_transaction, TransactionAnalytics},
        solana,
        ui_fields::{RouteSig, UiFields},
    },
    crate::{
        blockchain_api::BLOCKCHAIN_API_URL_PROD,
        call::Call,
        chain_abstraction::{
            api::{fungible_price::PriceQueryParams, prepare::Transactions},
            error::UiFieldsError,
            l1_data_fee::get_l1_data_fee,
            pulse::pulse,
            ui_fields::{self, Route},
        },
        erc20::ERC20,
        provider_pool::ProviderPool,
        serde::{duration_millis, option_duration_millis, systemtime_millis},
        time::{sleep, Duration, Instant, SystemTime},
    },
    alloy::{
        network::TransactionBuilder,
        primitives::{Address, PrimitiveSignature, B256, U256, U64},
        rpc::types::{TransactionReceipt, TransactionRequest},
    },
    alloy_provider::{utils::Eip1559Estimation, Provider},
    relay_rpc::domain::ProjectId,
    reqwest::Client as ReqwestClient,
    serde::{Deserialize, Serialize},
    solana_sdk::transaction::VersionedTransaction,
    std::collections::{HashMap, HashSet},
    url::Url,
};

#[derive(Clone)]
pub struct Client {
    pub provider_pool: ProviderPool,
    http_client: ReqwestClient,
    project_id: ProjectId,
    pulse_metadata: PulseMetadata,
}

impl Client {
    pub fn new(project_id: ProjectId, pulse_metadata: PulseMetadata) -> Self {
        Self::with_blockchain_api_url(
            project_id,
            pulse_metadata,
            BLOCKCHAIN_API_URL_PROD.parse().unwrap(),
        )
    }

    pub fn with_blockchain_api_url(
        project_id: ProjectId,
        pulse_metadata: PulseMetadata,
        blockchain_api_base_url: Url,
    ) -> Self {
        let client = ReqwestClient::builder().build();
        let client = match client {
            Ok(client) => client,
            Err(e) => {
                panic!("Failed to create reqwest client: {} ... {:?}", e, e)
            }
        };
        Self {
            provider_pool: ProviderPool::new(
                project_id.clone(),
                client.clone(),
                pulse_metadata.clone(),
                blockchain_api_base_url,
            ),
            http_client: client,
            project_id,
            pulse_metadata,
        }
    }

    /// accounts - List of other CAIP-10 accounts that the wallet has signing ability for
    pub async fn prepare(
        &self,
        chain_id: String,
        from: Address,
        call: Call,
        accounts: Vec<String>,
    ) -> Result<PrepareResponse, PrepareError> {
        let response = self
            .provider_pool
            .client
            .post(
                self.provider_pool
                    .blockchain_api_base_url
                    .join(ROUTE_ENDPOINT_PATH)
                    .unwrap(),
            )
            .json(&PrepareRequest {
                transaction: PrepareRequestTransaction {
                    chain_id,
                    from,
                    calls: CallOrCalls::Call { call },
                },
                accounts,
            })
            .query(&RouteQueryParams {
                project_id: self.provider_pool.project_id.clone(),
                sdk_type: Some(PULSE_SDK_TYPE.to_string()),
                sdk_version: Some(self.pulse_metadata.sdk_version.clone()),
                session_id: Some(self.provider_pool.session_id.to_string()),
            })
            .send()
            .await
            .map_err(PrepareError::Request)?;
        let status = response.status();
        if status.is_success() {
            let text =
                response.text().await.map_err(PrepareError::DecodingText)?;
            serde_json::from_str(&text)
                // .map(|mut response| {
                //     if let PrepareResponse::Success(
                //         PrepareResponseSuccess::Available(ref mut response),
                //     ) = response
                //     {
                //         // response.transactions.iter_mut().for_each(|t| {
                //         //     t.gas_limit *= U64::from(3);
                //         // });
                //         // response.initial_transaction.gas_limit *= U64::from(3);
                //         response.metadata.funding_from.iter_mut().for_each(
                //             |f| {
                //                 f.bridging_fee = U256::from(1000);
                //             },
                //         );
                //     }
                //     response
                // })
                .map_err(|e| PrepareError::DecodingJson(e, text))
        } else {
            match response.text().await {
                Ok(text) => Err(PrepareError::RequestFailed(text)),
                Err(e) => Err(PrepareError::RequestFailedText(e)),
            }
        }
    }

    pub async fn get_ui_fields(
        &self,
        prepare_response: PrepareResponseAvailable,
        local_currency: Currency,
        // TODO use this to e.g. modify priority fee
        // _speed: String,
    ) -> Result<UiFields, UiFieldsError> {
        if local_currency != Currency::Usd {
            unimplemented!("Only USD currency is supported for now");
        }

        let chains = prepare_response
            .transactions
            .iter()
            .flat_map(|t| match t {
                Transactions::Eip155(txns) => {
                    txns.iter().map(|t| t.chain_id.clone()).collect::<Vec<_>>()
                }
                Transactions::Solana(txns) => {
                    txns.iter().map(|t| t.chain_id.clone()).collect::<Vec<_>>()
                }
            })
            .chain(std::iter::once(
                prepare_response.initial_transaction.chain_id.clone(),
            ))
            .collect::<HashSet<_>>();
        println!("chains: {chains:?}");

        // TODO run fungible lookup, eip1559_fees, and l1 data fee, in parallel

        let addresses =
            chains
                .iter()
                .map(|t| format!("{}:{}", t, NATIVE_TOKEN_ADDRESS))
                .chain(
                    prepare_response.metadata.funding_from.iter().map(|f| {
                        format!("{}:{}", f.chain_id, f.token_contract)
                    }),
                )
                .collect::<HashSet<_>>();
        println!("addresses: {addresses:?}");

        let fungibles_future = futures::future::try_join_all(
            addresses.into_iter().map(|address| async move {
                // TODO: batch these requests when Blockchain API supports it: https://reown-inc.slack.com/archives/C0816SK4877/p1733168173213809
                let response = self
                    .provider_pool
                    .client
                    .post(
                        self.provider_pool
                            .blockchain_api_base_url
                            .join(FUNGIBLE_PRICE_ENDPOINT_PATH)
                            .unwrap(),
                    )
                    .query(&PriceQueryParams {
                        sdk_type: PULSE_SDK_TYPE.to_string(),
                        sdk_version: self.pulse_metadata.sdk_version.clone(),
                    })
                    .json(&PriceRequestBody {
                        project_id: self.provider_pool.project_id.clone(),
                        currency: local_currency,
                        addresses: HashSet::from([address]),
                    })
                    .send()
                    .await
                    .map_err(UiFieldsError::FungiblesRequest)?;
                let prices = if response.status().is_success() {
                    response
                        .json::<PriceResponseBody>()
                        .await
                        .map_err(UiFieldsError::FungiblesJson)
                } else {
                    Err(UiFieldsError::FungiblesRequestFailed(
                        response.status(),
                        response.text().await,
                    ))
                }?;
                Ok(prices.fungibles)
            }),
        );

        let estimate_future = futures::future::try_join_all(chains.iter().map(
            |chain_id| async move {
                self.provider_pool
                    .get_provider(chain_id)
                    .await
                    .estimate_eip1559_fees(None)
                    .await
                    .map_err(UiFieldsError::Eip1559Estimation)
                    .map(|estimate| (chain_id, estimate))
            },
        ));

        async fn l1_data_fee(
            txn: Transaction,
            providers: &Client,
        ) -> Result<U256, UiFieldsError> {
            get_l1_data_fee(
                TransactionRequest::default()
                    .with_from(txn.from)
                    .with_to(txn.to)
                    .with_value(txn.value)
                    .with_gas_limit(txn.gas_limit.to())
                    .with_input(txn.input.clone())
                    .with_nonce(txn.nonce.to())
                    .with_chain_id(
                        txn.chain_id
                            .strip_prefix("eip155:")
                            .unwrap()
                            .parse::<U64>()
                            .unwrap()
                            .to(),
                    )
                    .with_max_fee_per_gas(100000)
                    .with_max_priority_fee_per_gas(1),
                &providers.provider_pool.get_provider(&txn.chain_id).await,
            )
            .await
            .map_err(UiFieldsError::L1DataFee)
        }

        let route_l1_data_fee_futures = futures::future::try_join_all(
            prepare_response
                .transactions
                .iter()
                .flat_map(|t| match t {
                    Transactions::Eip155(txns) => txns.clone(),
                    Transactions::Solana(_txns) => Vec::new(),
                })
                .map(|txn| l1_data_fee(txn, self)),
        );
        let initial_l1_data_fee_future =
            l1_data_fee(prepare_response.initial_transaction.clone(), self);

        let (fungibles, eip1559_fees, route_l1_data_fees, initial_l1_data_fee) =
            tokio::try_join!(
                fungibles_future,
                estimate_future,
                route_l1_data_fee_futures,
                initial_l1_data_fee_future
            )?;
        let fungibles = fungibles.into_iter().flatten().collect::<Vec<_>>();
        let eip1559_fees = eip1559_fees.into_iter().collect::<HashMap<_, _>>();

        fn estimate_gas_fees(
            txn: Transaction,
            eip1559_fees: &HashMap<&String, Eip1559Estimation>,
            l1_data_fee: U256,
        ) -> (Transaction, Eip1559Estimation, U256) {
            let eip1559_estimation = *eip1559_fees.get(&txn.chain_id).unwrap();
            println!("l1_data_fee: {l1_data_fee}");
            let fee = U256::from(eip1559_estimation.max_fee_per_gas)
                .checked_mul(U256::from(txn.gas_limit))
                .expect("fee overflow")
                .checked_add(l1_data_fee)
                .expect("fee overflow in adding");
            (txn, eip1559_estimation, fee)
        }

        let mut estimated_transactions =
            Vec::with_capacity(prepare_response.transactions.len());
        for (txn, l1_data_fee) in prepare_response
            .transactions
            .iter()
            .flat_map(|t| match t {
                Transactions::Eip155(txns) => txns.clone(),
                Transactions::Solana(_txns) => Vec::new(),
            })
            .zip(route_l1_data_fees.into_iter())
        {
            estimated_transactions.push(estimate_gas_fees(
                txn,
                &eip1559_fees,
                l1_data_fee,
            ));
        }
        let estimated_initial_transaction = estimate_gas_fees(
            prepare_response.initial_transaction.clone(),
            &eip1559_fees,
            initial_l1_data_fee,
        );

        Ok(ui_fields::ui_fields(
            prepare_response,
            estimated_transactions,
            estimated_initial_transaction,
            fungibles,
        ))
    }

    // TODO test
    pub async fn prepare_detailed(
        &self,
        chain_id: String,
        from: Address,
        call: Call,
        accounts: Vec<String>,
        local_currency: Currency,
        // TODO use this to e.g. modify priority fee
        // _speed: String,
    ) -> Result<PrepareDetailedResponse, PrepareDetailedError> {
        let response = self
            .prepare(chain_id, from, call, accounts)
            .await
            .map_err(PrepareDetailedError::Prepare)?;
        match response {
            PrepareResponse::Success(response) => {
                Ok(PrepareDetailedResponse::Success(match response {
                    PrepareResponseSuccess::Available(response) => {
                        let res = self
                            .get_ui_fields(response, local_currency)
                            .await
                            .map_err(PrepareDetailedError::UiFields)?;
                        PrepareDetailedResponseSuccess::Available(res)
                    }
                    PrepareResponseSuccess::NotRequired(e) => {
                        PrepareDetailedResponseSuccess::NotRequired(e)
                    }
                }))
            }
            PrepareResponse::Error(e) => Ok(PrepareDetailedResponse::Error(e)),
        }
    }

    pub async fn status(
        &self,
        orchestration_id: String,
    ) -> Result<StatusResponse, StatusError> {
        let response = {
            let req = self
                .provider_pool
                .client
                .get(
                    self.provider_pool
                        .blockchain_api_base_url
                        .join(STATUS_ENDPOINT_PATH)
                        .unwrap(),
                )
                .query(&StatusQueryParams {
                    project_id: self.provider_pool.project_id.clone(),
                    orchestration_id,
                    session_id: Some(self.provider_pool.session_id.to_string()),
                    sdk_type: Some(PULSE_SDK_TYPE.to_string()),
                    sdk_version: Some(self.pulse_metadata.sdk_version.clone()),
                });
            // https://github.com/seanmonstar/reqwest/pull/1760
            #[cfg(not(target_arch = "wasm32"))]
            let req = req.timeout(Duration::from_secs(5));
            req
        }
        .send()
        .await
        .map_err(StatusError::Request)?
        .error_for_status()
        .map_err(StatusError::Request)?;
        let status = response.status();
        if status.is_success() {
            let text =
                response.text().await.map_err(StatusError::DecodingText)?;
            serde_json::from_str(&text)
                .map_err(|e| StatusError::DecodingJson(e, text))
        } else {
            match response.text().await {
                Ok(text) => Err(StatusError::RequestFailed(text)),
                Err(e) => Err(StatusError::RequestFailedText(e)),
            }
        }
    }

    pub async fn wait_for_success(
        &self,
        orchestration_id: String,
        check_in: Duration,
    ) -> Result<StatusResponseCompleted, WaitForSuccessError> {
        self.wait_for_success_with_timeout(
            orchestration_id,
            check_in,
            Duration::from_secs(120),
        )
        .await
    }

    /// Waits for the orchestration to complete, polling the status endpoint at
    /// a rate set by the orchestration server
    /// - `orchestration_id` - The orchestration ID returned from the route
    ///   endpoint
    /// - `check_in` - The check_in value returned from the route endpoint
    /// - `timeout` - An approximate timeout to wait for the orchestration to
    ///   complete
    pub async fn wait_for_success_with_timeout(
        &self,
        orchestration_id: String,
        check_in: Duration,
        timeout: Duration,
    ) -> Result<StatusResponseCompleted, WaitForSuccessError> {
        let start = Instant::now();
        sleep(check_in).await;
        loop {
            let result = self.status(orchestration_id.clone()).await;
            let (error, check_in) = match result {
                Ok(status_response_success) => match status_response_success {
                    StatusResponse::Completed(completed) => {
                        return Ok(completed);
                    }
                    StatusResponse::Error(e) => {
                        return Err(WaitForSuccessError::StatusResponseError(
                            e,
                        ));
                    }
                    StatusResponse::Pending(e) => {
                        let check_in = Duration::from_millis(e.check_in);
                        (
                            WaitForSuccessError::StatusResponsePending(e),
                            check_in,
                        )
                    }
                },
                Err(e) => {
                    (WaitForSuccessError::Status(e), Duration::from_secs(1))
                    // TODO exponential back-off (server-side): 0ms, 500ms, 1s
                }
            };
            if start.elapsed() > timeout {
                return Err(error);
            }
            sleep(check_in).await;
        }
    }

    /// Panics if:
    /// - The length of `route_txn_sigs` does not match the length of `ui_fields.route`
    pub async fn execute(
        &self,
        ui_fields: UiFields,
        route_txn_sigs: Vec<RouteSig>,
        initial_txn_sig: PrimitiveSignature,
    ) -> Result<ExecuteDetails, ExecuteError> {
        assert_eq!(
            ui_fields.route.len(),
            route_txn_sigs.len(),
            "route_txn_sigs length must match route length"
        );

        for (route_index, (route, route_sig)) in
            ui_fields.route.iter().zip(route_txn_sigs.iter()).enumerate()
        {
            match (route, route_sig) {
                (Route::Eip155(route), RouteSig::Eip155(route_sig)) => {
                    for (index, (txn, sig)) in
                        route.iter().zip(route_sig.iter()).enumerate()
                    {
                        let address = sig
                            .recover_address_from_prehash(
                                &txn.transaction_hash_to_sign,
                            )
                            .unwrap();
                        let expected_address = txn.transaction.from;
                        assert_eq!(address, expected_address, "invalid route signature at index {route_index}:eip155:{index}. Expected recovered address to be {expected_address} but got {address} instead");
                    }
                }
                (Route::Solana(route), RouteSig::Solana(route_sig)) => {
                    for (index, (txn, sig)) in
                        route.iter().zip(route_sig.iter()).enumerate()
                    {
                        assert!(
                            sig.verify(txn.from.as_array(), txn.transaction_hash_to_sign.as_ref()),
                            "invalid route signature at index {route_index}:solana:{index}. Signature is invalid");
                    }
                }
                _ => {
                    panic!("mis-matched route signature type for route transaction type at index {route_index}");
                }
            }
        }

        {
            let address = initial_txn_sig
                .recover_address_from_prehash(
                    &ui_fields.initial.transaction_hash_to_sign,
                )
                .unwrap();
            let expected_address = ui_fields.initial.transaction.from;
            assert_eq!(address, expected_address, "invalid initial txn signature. Expected recovered address to be {expected_address} but got {address} instead");
        }

        let result = self
            .execute_inner(ui_fields, route_txn_sigs, initial_txn_sig)
            .await;
        let (result, analytics) = match result {
            Ok((details, analytics)) => (Ok(details), analytics),
            Err((e, analytics)) => {
                // Doing it globally here in-case I would forget to do it deeper in
                // TODO refactor somehow to avoid this
                let mut analytics = analytics;
                analytics.error = Some(e.to_string());
                (Err(e), analytics)
            }
        };

        pulse(
            self.http_client.clone(),
            analytics,
            self.project_id.clone(),
            &self.pulse_metadata,
        );

        result
    }

    async fn execute_inner(
        &self,
        ui_fields: UiFields,
        route_txn_sigs: Vec<RouteSig>,
        initial_txn_sig: PrimitiveSignature,
    ) -> Result<
        (ExecuteDetails, ExecuteAnalytics),
        (ExecuteError, ExecuteAnalytics),
    > {
        let start = Instant::now();
        let start_time = SystemTime::now();

        let orchestration_id = ui_fields.route_response.orchestration_id;

        let route_start = start;
        let mut route = Vec::with_capacity(ui_fields.route.len());
        // TODO run in parallel
        for (route_index, (txn, sig)) in ui_fields
            .route
            .into_iter()
            .zip(route_txn_sigs.into_iter())
            .enumerate()
        {
            match (txn, sig) {
                (Route::Eip155(txn), RouteSig::Eip155(sig)) => {
                    for (txn, sig) in txn.into_iter().zip(sig.into_iter()) {
                        let result = send_transaction(
                            txn.transaction,
                            sig,
                            &self.provider_pool,
                        )
                        .await;
                        match result {
                            Ok((_receipt, analytics)) => {
                                route.push(analytics); // TODO refactor to avoid non-dry `route.push(analytics)` as it risks us forgettting it
                            }
                            Err((e, analytics)) => {
                                route.push(analytics);
                                let route_latency = route_start.elapsed();
                                let latency = start.elapsed();
                                return Err((
                                    ExecuteError::WithOrchestrationId {
                                        orchestration_id: orchestration_id
                                            .clone(),
                                        reason: ExecuteErrorReason::Route(e),
                                    },
                                    ExecuteAnalytics {
                                        orchestration_id: orchestration_id
                                            .clone(),
                                        error: None,
                                        start: start_time,
                                        route_latency,
                                        route,
                                        status_latency: None,
                                        initial_txn: None,
                                        latency,
                                        end: SystemTime::now(),
                                    },
                                ));
                            }
                        }
                    }
                }
                (Route::Solana(txn), RouteSig::Solana(sig)) => {
                    let sol_rpc = "https://api.mainnet-beta.solana.com";
                    let solana_rpc_client =
                        solana::SolanaRpcClient::new_with_commitment(
                            sol_rpc.to_string(),
                            solana::SolanaCommitmentConfig::confirmed(), // TODO what commitment level should we use?
                        );

                    for (txn, sig) in txn.into_iter().zip(sig.into_iter()) {
                        let transaction = VersionedTransaction {
                            signatures: vec![sig],
                            message: txn.transaction.message,
                        };

                        match solana_rpc_client
                            .send_and_confirm_transaction(&transaction)
                            .await
                        {
                            Ok(signature) => println!(
                                "Transfer successful! Signature: {}",
                                signature
                            ),
                            Err(e) => {
                                panic!("Error sending transaction: {}", e)
                            }
                        }
                    }
                }
                _ => {
                    panic!("mis-matched route transaction type for route signature type at index {route_index}");
                }
            }
        }
        let route_latency = route_start.elapsed();

        let status_start = Instant::now();
        let _success = self
            .wait_for_success(
                orchestration_id.clone(),
                Duration::from_millis(
                    ui_fields.route_response.metadata.check_in,
                ),
            )
            .await
            .map_err(|e| {
                let status_latency = status_start.elapsed();
                let latency = start.elapsed();
                (
                    ExecuteError::WithOrchestrationId {
                        orchestration_id: orchestration_id.clone(),
                        reason: ExecuteErrorReason::Bridge(e),
                    },
                    ExecuteAnalytics {
                        orchestration_id: orchestration_id.clone(),
                        error: None,
                        start: start_time,
                        route_latency,
                        route: route.clone(),
                        status_latency: Some(status_latency), // TODO refactor to avoid potentially forgetting to set this to Some() (also in subsequent ones)
                        initial_txn: None,
                        latency,
                        end: SystemTime::now(),
                    },
                )
            })?;
        let status_latency = status_start.elapsed();

        let (initial_txn_receipt, initial_txn_analytics) = send_transaction(
            ui_fields.initial.transaction,
            initial_txn_sig,
            &self.provider_pool,
        )
        .await
        .map_err(|(e, analytics)| {
            let latency = start.elapsed();
            (
                ExecuteError::WithOrchestrationId {
                    orchestration_id: orchestration_id.clone(),
                    reason: ExecuteErrorReason::Initial(e),
                },
                ExecuteAnalytics {
                    orchestration_id: orchestration_id.clone(),
                    error: None,
                    start: start_time,
                    route_latency,
                    route: route.clone(),
                    status_latency: Some(status_latency),
                    initial_txn: Some(analytics),
                    latency,
                    end: SystemTime::now(),
                },
            )
        })?;

        let details = ExecuteDetails {
            initial_txn_hash: initial_txn_receipt.transaction_hash,
            initial_txn_receipt,
        };

        let latency = start.elapsed();

        let analytics = ExecuteAnalytics {
            orchestration_id,
            error: None,
            start: start_time,
            route_latency,
            route,
            status_latency: Some(status_latency),
            initial_txn: Some(initial_txn_analytics),
            latency,
            end: SystemTime::now(),
        };

        Ok((details, analytics))
    }

    pub async fn erc20_token_balance(
        &self,
        chain_id: &str,
        token: Address,
        owner: Address,
    ) -> Result<U256, alloy::contract::Error> {
        let provider = self.provider_pool.get_provider(chain_id).await;
        let erc20 = ERC20::new(token, provider);
        let balance = erc20.balanceOf(owner).call().await?;
        Ok(balance.balance)
    }
}

#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteDetails {
    pub initial_txn_receipt: TransactionReceipt,
    pub initial_txn_hash: B256,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteAnalytics {
    pub orchestration_id: String,
    pub error: Option<String>,
    #[serde(with = "systemtime_millis")]
    pub start: SystemTime,
    #[serde(with = "duration_millis")]
    pub route_latency: Duration,
    pub route: Vec<TransactionAnalytics>,
    #[serde(with = "option_duration_millis")]
    pub status_latency: Option<Duration>,
    pub initial_txn: Option<TransactionAnalytics>,
    #[serde(with = "duration_millis")]
    pub latency: Duration,
    #[serde(with = "systemtime_millis")]
    pub end: SystemTime,
}

// TODO test non-happy paths: txn failures
