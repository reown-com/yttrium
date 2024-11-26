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
use crate::chain_abstraction::error::RouteUiFieldsError;
use alloy::{
    network::Ethereum,
    primitives::{utils::Unit, U256},
};
use alloy_provider::{utils::Eip1559Estimation, Provider, ReqwestProvider};
use relay_rpc::domain::ProjectId;
use reqwest::{Client as ReqwestClient, Url};
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

pub struct Client {
    client: ReqwestClient,
    base_url: Url,
    pub project_id: ProjectId,
}

impl Client {
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            client: ReqwestClient::new(),
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
        if response.status().is_success() {
            response.json().await.map_err(RouteError::Request)
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
        let addresses = chains
            .iter()
            .map(|t| format!("{}:{}", t, NATIVE_TOKEN_ADDRESS))
            .collect();
        println!("addresses: {addresses:?}");
        let response = self
            .client
            .post(self.base_url.join(FUNGIBLE_PRICE_ENDPOINT_PATH).unwrap())
            .json(&PriceRequestBody {
                project_id: self.project_id.clone(),
                currency,
                addresses,
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

        let mut eip1559_fees = HashMap::new();
        for chain_id in &chains {
            // TODO use internal provider mapping
            let url = format!(
                "https://rpc.walletconnect.com/v1?chainId={chain_id}&projectId={}",
                self.project_id
            )
                .parse()
            .expect("Invalid RPC URL");
            let provider = ReqwestProvider::<Ethereum>::new_http(url);
            let estimate = provider.estimate_eip1559_fees(None).await.unwrap();
            eip1559_fees.insert(chain_id, estimate);
        }

        fn estimate_gas_fees(
            txn: Transaction,
            eip1559_fees: &HashMap<&String, Eip1559Estimation>,
        ) -> (Transaction, Eip1559Estimation, U256) {
            let eip1559_estimation = *eip1559_fees.get(&txn.chain_id).unwrap();
            let fee = U256::from(eip1559_estimation.max_fee_per_gas)
                .checked_mul(U256::from(txn.gas))
                .expect("fee overflow");
            // TODO L1 Data Fee
            // TODO potentially queued cost?
            (txn, eip1559_estimation, fee)
        }

        let mut estimated_transactions =
            Vec::with_capacity(route_response.transactions.len());
        for txn in route_response.transactions {
            estimated_transactions.push(estimate_gas_fees(txn, &eip1559_fees));
        }
        let estimated_initial_transaction =
            estimate_gas_fees(initial_transaction, &eip1559_fees);

        // Set desired granularity for local currency exchange rate
        // Individual prices may have less granularity
        // Must have 1 decimals value for all rates for math to work
        const DECIMALS_LOCAL_EXCHANGE_RATE: u8 = 6;
        let const_local_unit =
            Unit::new(Unit::ETHER.get() + DECIMALS_LOCAL_EXCHANGE_RATE)
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
            prices: &PriceResponseBody,
        ) -> (Transaction, Eip1559Estimation, TransactionFee) {
            let fungible = prices.fungibles.first().unwrap();
            // let fungible = prices
            //     .fungibles
            //     .iter()
            //     .find(|f| {
            //         f.address
            //             == format!("{}:{}", txn.chain_id,
            // NATIVE_TOKEN_ADDRESS)     })
            //     .unwrap();
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
                    fee: Amount::new(fungible.symbol.clone(), fee, Unit::ETHER),
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
                &prices,
            ));
        }
        let initial = compute_amounts(
            estimated_initial_transaction,
            &mut total_local_fee,
            const_local_unit,
            &prices,
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
        if response.status().is_success() {
            response.json().await.map_err(RouteError::Request)
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
