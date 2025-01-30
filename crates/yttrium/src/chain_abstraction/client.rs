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
            PrepareDetailedError, PrepareDetailedResponse,
            PrepareDetailedResponseSuccess, PrepareError, WaitForSuccessError,
        },
        ui_fields::UiFields,
    },
    crate::{
        call::Call,
        chain_abstraction::{
            error::UiFieldsError, l1_data_fee::get_l1_data_fee, ui_fields,
        },
        erc20::ERC20,
        provider_pool::ProviderPool,
    },
    alloy::{
        consensus::{SignableTransaction, TxEnvelope},
        network::TransactionBuilder,
        primitives::{Address, PrimitiveSignature, U256, U64},
        rpc::types::{TransactionReceipt, TransactionRequest},
    },
    alloy_provider::{utils::Eip1559Estimation, Provider},
    relay_rpc::domain::ProjectId,
    std::{
        collections::{HashMap, HashSet},
        time::{Duration, Instant},
    },
};

#[derive(Clone)]
pub struct Client {
    provider_pool: ProviderPool,
}

impl Client {
    pub fn new(project_id: ProjectId) -> Self {
        Self { provider_pool: ProviderPool::new(project_id) }
    }

    pub async fn prepare(
        &self,
        chain_id: String,
        from: Address,
        call: Call,
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
            })
            .query(&RouteQueryParams {
                project_id: self.provider_pool.project_id.clone(),
            })
            .send()
            .await
            .map_err(PrepareError::Request)?;
        let status = response.status();
        if status.is_success() {
            let text =
                response.text().await.map_err(PrepareError::DecodingText)?;
            serde_json::from_str(&text)
                .map_err(|e| PrepareError::DecodingJson(e, text))
        } else {
            Err(PrepareError::RequestFailed(response.text().await))
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
            .chain(std::iter::once(&prepare_response.initial_transaction))
            .map(|t| t.chain_id.clone())
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
                    .json(&PriceRequestBody {
                        project_id: self.provider_pool.project_id.clone(),
                        currency: local_currency,
                        addresses: HashSet::from([address]),
                    })
                    .send()
                    .await
                    .map_err(UiFieldsError::Request)?;
                let prices = if response.status().is_success() {
                    response
                        .json::<PriceResponseBody>()
                        .await
                        .map_err(UiFieldsError::Json)
                } else {
                    Err(UiFieldsError::RequestFailed(
                        response.status(),
                        response.text().await,
                    ))
                }?;
                Ok(prices.fungibles)
            }),
        );

        let estimate_future = futures::future::try_join_all(chains.iter().map(
            |chain_id| async move {
                let estimate = self
                    .provider_pool
                    .get_provider(chain_id)
                    .await
                    .estimate_eip1559_fees(None)
                    .await
                    .unwrap();
                Ok((chain_id, estimate))
            },
        ));

        async fn l1_data_fee(
            txn: Transaction,
            providers: &Client,
        ) -> Result<U256, UiFieldsError> {
            Ok(get_l1_data_fee(
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
                providers.provider_pool.get_provider(&txn.chain_id).await,
            )
            .await)
        }

        let route_l1_data_fee_futures = futures::future::try_join_all(
            prepare_response
                .transactions
                .iter()
                .map(|txn| l1_data_fee(txn.clone(), self)),
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
            .clone()
            .transactions
            .into_iter()
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
        local_currency: Currency,
        // TODO use this to e.g. modify priority fee
        // _speed: String,
    ) -> Result<PrepareDetailedResponse, PrepareDetailedError> {
        let response = self
            .prepare(chain_id, from, call)
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

    // TODO don't use "prepare" error type here. Maybe rename to generic request error?
    pub async fn status(
        &self,
        orchestration_id: String,
    ) -> Result<StatusResponse, PrepareError> {
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
                });
            // https://github.com/seanmonstar/reqwest/pull/1760
            #[cfg(not(target_arch = "wasm32"))]
            let req = req.timeout(Duration::from_secs(5));
            req
        }
        .send()
        .await
        .map_err(PrepareError::Request)?
        .error_for_status()
        .map_err(PrepareError::Request)?;
        let status = response.status();
        if status.is_success() {
            let text =
                response.text().await.map_err(PrepareError::DecodingText)?;
            serde_json::from_str(&text)
                .map_err(|e| PrepareError::DecodingJson(e, text))
        } else {
            Err(PrepareError::RequestFailed(response.text().await))
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
            Duration::from_secs(30),
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
        tokio::time::sleep(check_in).await;
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
                    (WaitForSuccessError::Prepare(e), Duration::from_secs(1))
                    // TODO exponential back-off: 0ms, 500ms, 1s
                }
            };
            if start.elapsed() > timeout {
                return Err(error);
            }
            tokio::time::sleep(check_in).await;
        }
    }

    pub async fn execute(
        &self,
        ui_fields: UiFields,
        route_txn_sigs: Vec<PrimitiveSignature>,
        initial_txn_sig: PrimitiveSignature,
    ) -> ExecuteDetails {
        assert_eq!(
            ui_fields.route.len(),
            route_txn_sigs.len(),
            "route_txn_sigs length must match route length"
        );
        for (txn, sig) in
            ui_fields.route.into_iter().zip(route_txn_sigs.into_iter())
        {
            let provider = self
                .provider_pool
                .get_provider(&txn.transaction.chain_id)
                .await;
            let signed = txn.transaction.into_eip1559().into_signed(sig);
            assert!(provider
                .send_tx_envelope(TxEnvelope::Eip1559(signed))
                .await
                .unwrap()
                .with_timeout(Some(Duration::from_secs(15)))
                .get_receipt()
                .await
                .unwrap()
                .status());
        }

        let _success = self
            .wait_for_success(
                ui_fields.route_response.orchestration_id,
                Duration::from_millis(
                    ui_fields.route_response.metadata.check_in,
                ),
            )
            .await
            .unwrap();

        let provider = self
            .provider_pool
            .get_provider(&ui_fields.initial.transaction.chain_id)
            .await;
        let signed = ui_fields
            .initial
            .transaction
            .into_eip1559()
            .into_signed(initial_txn_sig);
        let initial_txn_receipt = provider
            .send_tx_envelope(TxEnvelope::Eip1559(signed))
            .await
            .unwrap()
            .with_timeout(Some(Duration::from_secs(15)))
            .get_receipt()
            .await
            .unwrap();
        assert!(initial_txn_receipt.status());

        ExecuteDetails { initial_txn_receipt }
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
pub struct ExecuteDetails {
    pub initial_txn_receipt: TransactionReceipt,
}
