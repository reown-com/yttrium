use {
    super::{
        api::{
            fungible_price::{
                PriceRequestBody, PriceResponseBody,
                FUNGIBLE_PRICE_ENDPOINT_PATH, NATIVE_TOKEN_ADDRESS,
            },
            route::{
                RouteQueryParams, RouteRequest, RouteResponse,
                RouteResponseAvailable, ROUTE_ENDPOINT_PATH,
            },
            status::{
                StatusQueryParams, StatusResponse, StatusResponseCompleted,
                STATUS_ENDPOINT_PATH,
            },
            InitialTransaction, Transaction,
        },
        currency::Currency,
        error::{RouteError, WaitForSuccessError},
        route_ui_fields::RouteUiFields,
    },
    crate::{
        chain_abstraction::{
            error::RouteUiFieldsError, l1_data_fee::get_l1_data_fee,
            route_ui_fields,
        },
        erc20::ERC20,
    },
    alloy::{
        network::{Ethereum, TransactionBuilder},
        primitives::{Address, U256, U64},
        rpc::{client::RpcClient, types::TransactionRequest},
        transports::http::Http,
    },
    alloy_provider::{utils::Eip1559Estimation, Provider, ReqwestProvider},
    relay_rpc::domain::ProjectId,
    reqwest::{Client as ReqwestClient, Url},
    std::{
        collections::{HashMap, HashSet},
        sync::Arc,
        time::{Duration, Instant},
    },
    tokio::sync::RwLock,
};

pub const PROXY_ENDPOINT_PATH: &str = "/v1";

#[derive(Clone)]
pub struct Client {
    client: ReqwestClient,
    providers: Arc<RwLock<HashMap<String, ReqwestProvider>>>,
    base_url: Url,
    pub project_id: ProjectId,
}

impl Client {
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            client: ReqwestClient::new(),
            providers: Arc::new(RwLock::new(HashMap::new())),
            base_url: "https://rpc.walletconnect.com".parse().unwrap(),
            project_id,
        }
    }

    pub async fn route(
        &self,
        transaction: InitialTransaction,
    ) -> Result<RouteResponse, RouteError> {
        let response = self
            .client
            .post(self.base_url.join(ROUTE_ENDPOINT_PATH).unwrap())
            .json(&RouteRequest { transaction })
            .query(&RouteQueryParams { project_id: self.project_id.clone() })
            .send()
            .await
            .map_err(RouteError::Request)?;
        let status = response.status();
        if status.is_success() {
            let text =
                response.text().await.map_err(RouteError::DecodingText)?;
            serde_json::from_str(&text)
                .map_err(|e| RouteError::DecodingJson(e, text))
        } else {
            Err(RouteError::RequestFailed(response.text().await))
        }
    }

    pub async fn get_route_ui_fields(
        &self,
        route_response: RouteResponseAvailable,
        local_currency: Currency,
        // TODO use this to e.g. modify priority fee
        // _speed: String,
    ) -> Result<RouteUiFields, RouteUiFieldsError> {
        if local_currency != Currency::Usd {
            unimplemented!("Only USD currency is supported for now");
        }

        let chains = route_response
            .transactions
            .iter()
            .chain(std::iter::once(&route_response.initial_transaction))
            .map(|t| t.chain_id.clone())
            .collect::<HashSet<_>>();
        println!("chains: {chains:?}");

        // TODO run fungible lookup, eip1559_fees, and l1 data fee, in parallel

        let addresses =
            chains
                .iter()
                .map(|t| format!("{}:{}", t, NATIVE_TOKEN_ADDRESS))
                .chain(
                    route_response.metadata.funding_from.iter().map(|f| {
                        format!("{}:{}", f.chain_id, f.token_contract)
                    }),
                )
                .collect::<HashSet<_>>();
        println!("addresses: {addresses:?}");

        let fungibles_future = futures::future::try_join_all(
            addresses.into_iter().map(|address| async move {
                // TODO: batch these requests when Blockchain API supports it: https://reown-inc.slack.com/archives/C0816SK4877/p1733168173213809
                let response = self
                    .client
                    .post(
                        self.base_url
                            .join(FUNGIBLE_PRICE_ENDPOINT_PATH)
                            .unwrap(),
                    )
                    .json(&PriceRequestBody {
                        project_id: self.project_id.clone(),
                        currency: local_currency,
                        addresses: HashSet::from([address]),
                    })
                    .send()
                    .await
                    .map_err(RouteUiFieldsError::Request)?;
                let prices = if response.status().is_success() {
                    response
                        .json::<PriceResponseBody>()
                        .await
                        .map_err(RouteUiFieldsError::Json)
                } else {
                    Err(RouteUiFieldsError::RequestFailed(
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
                    .get_provider(chain_id.clone())
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
        ) -> Result<U256, RouteUiFieldsError> {
            Ok(get_l1_data_fee(
                TransactionRequest::default()
                    .with_from(txn.from)
                    .with_to(txn.to)
                    .with_value(txn.value)
                    .with_gas_limit(txn.gas_limit.to())
                    .with_input(txn.data.clone())
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
                providers.get_provider(txn.chain_id.clone()).await,
            )
            .await)
        }

        let route_l1_data_fee_futures = futures::future::try_join_all(
            route_response
                .transactions
                .iter()
                .map(|txn| l1_data_fee(txn.clone(), self)),
        );
        let initial_l1_data_fee_future =
            l1_data_fee(route_response.initial_transaction.clone(), self);

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
            Vec::with_capacity(route_response.transactions.len());
        for (txn, l1_data_fee) in route_response
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
            route_response.initial_transaction.clone(),
            &eip1559_fees,
            initial_l1_data_fee,
        );

        Ok(route_ui_fields::get_route_ui_fields(
            route_response,
            estimated_transactions,
            estimated_initial_transaction,
            fungibles,
        ))
    }

    pub async fn status(
        &self,
        orchestration_id: String,
    ) -> Result<StatusResponse, RouteError> {
        let response = self
            .client
            .get(self.base_url.join(STATUS_ENDPOINT_PATH).unwrap())
            .query(&StatusQueryParams {
                project_id: self.project_id.clone(),
                orchestration_id,
            })
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(RouteError::Request)?
            .error_for_status()
            .map_err(RouteError::Request)?;
        let status = response.status();
        if status.is_success() {
            let text =
                response.text().await.map_err(RouteError::DecodingText)?;
            serde_json::from_str(&text)
                .map_err(|e| RouteError::DecodingJson(e, text))
        } else {
            Err(RouteError::RequestFailed(response.text().await))
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
                    (WaitForSuccessError::RouteError(e), Duration::from_secs(1))
                    // TODO exponential back-off: 0ms, 500ms, 1s
                }
            };
            if start.elapsed() > timeout {
                return Err(error);
            }
            tokio::time::sleep(check_in).await;
        }
    }

    pub async fn get_provider(&self, chain_id: String) -> ReqwestProvider {
        let providers = self.providers.read().await;
        if let Some(provider) = providers.get(&chain_id) {
            provider.clone()
        } else {
            std::mem::drop(providers);

            // TODO use universal version: https://linear.app/reown/issue/RES-142/universal-provider-router
            let mut url = self.base_url.join(PROXY_ENDPOINT_PATH).unwrap();
            url.query_pairs_mut()
                .append_pair("chainId", &chain_id)
                .append_pair("projectId", self.project_id.as_ref());
            let provider = ReqwestProvider::<Ethereum>::new(RpcClient::new(
                Http::with_client(self.client.clone(), url),
                false,
            ));
            self.providers.write().await.insert(chain_id, provider.clone());
            provider
        }
    }

    pub async fn erc20_token_balance(
        &self,
        chain_id: String,
        token: Address,
        owner: Address,
    ) -> Result<U256, alloy::contract::Error> {
        let provider = self.get_provider(chain_id).await;
        let erc20 = ERC20::new(token, provider);
        let balance = erc20.balanceOf(owner).call().await?;
        Ok(balance.balance)
    }
}
