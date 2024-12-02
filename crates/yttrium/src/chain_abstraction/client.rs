use super::{
    amount::Amount,
    api::{
        fungible_price::{
            PriceRequestBody, PriceResponseBody, FUNGIBLE_PRICE_ENDPOINT_PATH,
            NATIVE_TOKEN_ADDRESS,
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
};
use crate::{
    chain_abstraction::{
        api::fungible_price::FungiblePriceItem, error::RouteUiFieldsError,
        l1_data_fee::get_l1_data_fee,
    },
    erc20::ERC20,
};
use alloy::{
    network::{Ethereum, TransactionBuilder},
    primitives::{utils::Unit, Address, U256, U64},
    rpc::{client::RpcClient, types::TransactionRequest},
    transports::http::Http,
};
use alloy_provider::{utils::Eip1559Estimation, Provider, ReqwestProvider};
use relay_rpc::domain::ProjectId;
use reqwest::{Client as ReqwestClient, Url};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

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

        let addresses = chains
            .iter()
            .map(|t| format!("{}:{}", t, NATIVE_TOKEN_ADDRESS))
            .collect::<HashSet<_>>();
        let mut fungibles = Vec::with_capacity(addresses.len());
        println!("addresses: {addresses:?}");
        for address in addresses {
            // TODO: batch these requests when Blockchain API supports it: https://reown-inc.slack.com/archives/C0816SK4877/p1733168173213809
            let response = self
                .client
                .post(self.base_url.join(FUNGIBLE_PRICE_ENDPOINT_PATH).unwrap())
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
            fungibles.extend(prices.fungibles);
        }

        let mut eip1559_fees = HashMap::new();
        for chain_id in &chains {
            let estimate = self
                .get_provider(chain_id.clone())
                .await
                .estimate_eip1559_fees(None)
                .await
                .unwrap();
            eip1559_fees.insert(chain_id, estimate);
        }

        async fn estimate_gas_fees(
            txn: Transaction,
            eip1559_fees: &HashMap<&String, Eip1559Estimation>,
            providers: &Client,
        ) -> (Transaction, Eip1559Estimation, U256) {
            let eip1559_estimation = *eip1559_fees.get(&txn.chain_id).unwrap();
            let l1_data_fee = get_l1_data_fee(
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
                    .with_max_fee_per_gas(eip1559_estimation.max_fee_per_gas)
                    .max_priority_fee_per_gas(
                        eip1559_estimation.max_priority_fee_per_gas,
                    ),
                providers.get_provider(txn.chain_id.clone()).await,
            )
            .await;
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
        for txn in route_response.transactions {
            estimated_transactions
                .push(estimate_gas_fees(txn, &eip1559_fees, self).await);
        }
        let estimated_initial_transaction =
            estimate_gas_fees(initial_transaction, &eip1559_fees, self).await;

        // Set desired granularity for local currency exchange rate
        // Individual prices may have less granularity
        // Must have 1 decimals value for all rates for math to work
        const DECIMALS_LOCAL_EXCHANGE_RATE: u8 = 6;
        const FUNGIBLE_DECIMALS: Unit = Unit::ETHER;
        let const_local_unit =
            Unit::new(FUNGIBLE_DECIMALS.get() + DECIMALS_LOCAL_EXCHANGE_RATE)
                .unwrap();
        let mut total_local_fee = U256::ZERO;

        fn compute_amounts(
            (txn, eip1559_estimation, fee): (
                Transaction,
                Eip1559Estimation,
                U256,
            ),
            total_local_fee: &mut U256,
            const_local_unit: Unit,
            fungibles: &[FungiblePriceItem],
        ) -> (Transaction, Eip1559Estimation, TransactionFee) {
            let native_token_caip10 = format!(
                "{}:{}",
                txn.chain_id,
                NATIVE_TOKEN_ADDRESS.to_checksum(None)
            );
            println!("native_token_caip10: {native_token_caip10}");
            println!("fungibles: {fungibles:?}");
            let fungible = fungibles
                .iter()
                .find(|f| f.address == native_token_caip10)
                .unwrap();

            // Math currently doesn't support variable decimals for fungible
            // assets
            assert_eq!(fungible.decimals, FUNGIBLE_DECIMALS);
            let fungible_local_exchange_rate = U256::from(
                fungible.price
                    * (10_f64).powf(DECIMALS_LOCAL_EXCHANGE_RATE as f64),
            );

            let local_fee = fee * fungible_local_exchange_rate;
            *total_local_fee += local_fee;

            (
                txn,
                eip1559_estimation,
                TransactionFee {
                    fee: Amount::new(
                        fungible.symbol.clone(),
                        fee,
                        fungible.decimals,
                    ),
                    local_fee: Amount::new(
                        "USD".to_owned(),
                        local_fee,
                        const_local_unit,
                    ),
                },
            )
        }

        let mut route = Vec::with_capacity(estimated_transactions.len());
        for item in estimated_transactions {
            route.push(compute_amounts(
                item,
                &mut total_local_fee,
                const_local_unit,
                &fungibles,
            ));
        }
        let initial = compute_amounts(
            estimated_initial_transaction,
            &mut total_local_fee,
            const_local_unit,
            &fungibles,
        );

        Ok(RouteUiFields {
            route,
            // bridge: TransactionFee {
            //     fee: Amount::zero(),
            //     local_fee: Amount::zero(),
            // },
            initial,
            local_total: Amount::new(
                "USD".to_owned(),
                total_local_fee,
                const_local_unit,
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
    // pub bridge: TxnDetails,
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
