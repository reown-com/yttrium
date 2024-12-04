use {
    super::{
        amount::Amount,
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
            Transaction,
        },
        currency::Currency,
        error::{RouteError, WaitForSuccessError},
    },
    crate::{
        chain_abstraction::{
            amount::from_float, api::fungible_price::FungiblePriceItem,
            error::RouteUiFieldsError, l1_data_fee::get_l1_data_fee,
            local_fee_acc::LocalAmountAcc,
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
    tracing::warn,
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
        transaction: Transaction,
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
        initial_transaction: Transaction,
        currency: Currency,
        // TODO use this to e.g. modify priority fee
        _speed: String,
    ) -> Result<RouteUiFields, RouteUiFieldsError> {
        let chains = route_response
            .transactions
            .iter()
            .chain(std::iter::once(&initial_transaction))
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
                        currency,
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
                    .with_gas_limit(txn.gas.to())
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
            l1_data_fee(initial_transaction.clone(), self);

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
                .checked_mul(U256::from(txn.gas))
                .expect("fee overflow")
                .checked_add(l1_data_fee)
                .expect("fee overflow in adding");
            (txn, eip1559_estimation, fee)
        }

        let mut estimated_transactions =
            Vec::with_capacity(route_response.transactions.len());
        for (txn, l1_data_fee) in route_response
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
            initial_transaction,
            &eip1559_fees,
            initial_l1_data_fee,
        );

        let mut total_local_fee = LocalAmountAcc::new();

        fn compute_amounts(
            fee: U256,
            total_local_fee: &mut LocalAmountAcc,
            fungible: &FungiblePriceItem,
        ) -> TransactionFee {
            // `fungible.price` is a float; with obviously floating-point so should have great precision
            // Set this value to a value that is high enough to capture the desired price movement
            // Setting it too high may overflow the 77 decimal places (Unit::MAX) of the U256
            // Some tokens such as ETH only need 2 decimal places because their value is very high (>1000) and price moves are large
            // Some tokens may be worth e.g. 0.000001 USD per token, so we need to capture more decimal places to even see price movement
            const FUNGIBLE_PRICE_PRECISION: u8 = 8;

            let (fungible_price, fungible_price_decimals) =
                from_float(fungible.price, FUNGIBLE_PRICE_PRECISION);

            total_local_fee.add(
                fee,
                fungible.decimals,
                fungible_price,
                fungible_price_decimals,
            );

            let mut local_fee = LocalAmountAcc::new();
            local_fee.add(
                fee,
                fungible.decimals,
                fungible_price,
                fungible_price_decimals,
            );
            let (local_fee, local_fee_unit) = local_fee.compute();

            TransactionFee {
                fee: Amount::new(
                    fungible.symbol.clone(),
                    fee,
                    fungible.decimals,
                ),
                local_fee: Amount::new(
                    "USD".to_owned(),
                    local_fee,
                    local_fee_unit,
                ),
            }
        }

        let mut route = Vec::with_capacity(estimated_transactions.len());
        for item in estimated_transactions {
            let fee = compute_amounts(
                item.2,
                &mut total_local_fee,
                fungibles
                    .iter()
                    .find(|f| {
                        f.address
                            == format!(
                                "{}:{}",
                                item.0.chain_id,
                                NATIVE_TOKEN_ADDRESS.to_checksum(None)
                            )
                    })
                    .unwrap(),
            );
            route.push((item.0, item.1, fee));
        }

        let initial_fee = compute_amounts(
            estimated_initial_transaction.2,
            &mut total_local_fee,
            fungibles
                .iter()
                .find(|f| {
                    f.address
                        == format!(
                            "{}:{}",
                            estimated_initial_transaction.0.chain_id,
                            NATIVE_TOKEN_ADDRESS.to_checksum(None)
                        )
                })
                .unwrap(),
        );
        let initial = (
            estimated_initial_transaction.0,
            estimated_initial_transaction.1,
            initial_fee,
        );

        let mut bridge =
            Vec::with_capacity(route_response.metadata.funding_from.len());
        for item in route_response.metadata.funding_from {
            let fungible = fungibles
                .iter()
                .find(|f| {
                    f.address
                        == format!("{}:{}", item.chain_id, item.token_contract)
                })
                .unwrap();
            if item.symbol != fungible.symbol {
                warn!(
                    "Fungible symbol mismatch: item:{} != fungible:{}",
                    item.symbol, fungible.symbol
                );
            }
            bridge.push(compute_amounts(
                item.bridging_fee,
                &mut total_local_fee,
                fungible,
            ))
        }

        let (local_total_fee, local_total_fee_unit) = total_local_fee.compute();
        Ok(RouteUiFields {
            route,
            bridge,
            initial,
            local_total: Amount::new(
                "USD".to_owned(),
                local_total_fee,
                local_total_fee_unit,
            ),
        })
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
        &'static self,
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

#[derive(Debug)]
pub struct RouteUiFields {
    pub route: Vec<TxnDetails>,
    pub bridge: Vec<TransactionFee>,
    pub initial: TxnDetails,
    pub local_total: Amount,
}

pub type TxnDetails = (Transaction, Eip1559Estimation, TransactionFee);

#[derive(Debug)]
pub struct TransactionFee {
    pub fee: Amount,
    pub local_fee: Amount,
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     #[ignore = "manual"]
//     async fn manual_test_get_route_ui_fields() {
//         let client =
// Client::new(std::env::var("REOWN_PROJECT_ID").unwrap().into());

//     }
// }
