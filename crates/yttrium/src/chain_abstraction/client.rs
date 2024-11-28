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
use crate::chain_abstraction::{
    error::RouteUiFieldsError, l1_data_fee::get_l1_data_fee,
};
use alloy::{
    network::{Ethereum, TransactionBuilder},
    primitives::{utils::Unit, U256, U64},
    rpc::{client::RpcClient, types::TransactionRequest},
    transports::http::Http,
};
use alloy_provider::{utils::Eip1559Estimation, Provider, ReqwestProvider};
use relay_rpc::domain::ProjectId;
use reqwest::{Client as ReqwestClient, Url};
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

pub const PROXY_ENDPOINT_PATH: &str = "/v1";

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

        let mut providers = HashMap::new();
        for chain_id in &chains {
            let mut url = self.base_url.join(PROXY_ENDPOINT_PATH).unwrap();
            url.query_pairs_mut()
                .append_pair("chainId", chain_id)
                .append_pair("projectId", self.project_id.as_ref());
            let provider = ReqwestProvider::<Ethereum>::new(RpcClient::new(
                Http::with_client(self.client.clone(), url),
                false,
            ));
            providers.insert(chain_id, provider);
        }

        // TODO run fungible lookup, eip1559_fees, and l1 data fee, in parallel

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
            let estimate = providers
                .get(chain_id)
                .unwrap()
                .estimate_eip1559_fees(None)
                .await
                .unwrap();
            eip1559_fees.insert(chain_id, estimate);
        }

        async fn estimate_gas_fees(
            txn: Transaction,
            eip1559_fees: &HashMap<&String, Eip1559Estimation>,
            providers: &HashMap<&String, ReqwestProvider>,
        ) -> (Transaction, Eip1559EstimationFields, U256) {
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
                providers.get(&txn.chain_id).unwrap().clone(),
            )
            .await;
            println!("l1_data_fee: {l1_data_fee}");
            let fee = U256::from(eip1559_estimation.max_fee_per_gas)
                .checked_mul(U256::from(txn.gas))
                .expect("fee overflow")
                .checked_add(l1_data_fee)
                .expect("fee overflow in adding");

            (txn, Eip1559EstimationFields::from(eip1559_estimation), fee)
        }

        let mut estimated_transactions =
            Vec::with_capacity(route_response.transactions.len());
        for txn in route_response.transactions {
            estimated_transactions
                .push(estimate_gas_fees(txn, &eip1559_fees, &providers).await);
        }
        let estimated_initial_transaction =
            estimate_gas_fees(initial_transaction, &eip1559_fees, &providers)
                .await;

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
                Eip1559EstimationFields,
                U256,
            ),
            total_local_fee: &mut U256,
            const_local_unit: Unit,
            prices: &PriceResponseBody,
        ) -> TxnDetails {
            let fungible = prices.fungibles.first().unwrap();
            // let fungible = prices
            //     .fungibles
            //     .iter()
            //     .find(|f| {
            //         f.address
            //             == format!("{}:{}", txn.chain_id,
            // NATIVE_TOKEN_ADDRESS)     })
            //     .unwrap();

            // Math currently doesn't support variable decimals for fungible
            // assets
            assert_eq!(fungible.decimals, FUNGIBLE_DECIMALS);
            let fungible_local_exchange_rate = U256::from(
                fungible.price
                    * (10_f64).powf(DECIMALS_LOCAL_EXCHANGE_RATE as f64),
            );

            let local_fee = fee * fungible_local_exchange_rate;
            *total_local_fee += local_fee;
            TxnDetails {
                transaction: txn,
                eip1559: eip1559_estimation,
                transaction_fee: TransactionFee {
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
            }
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
            route_details: route,
            // bridge: TransactionFee {
            //     fee: Amount::zero(),
            //     local_fee: Amount::zero(),
            // },
            initial_details: initial,
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
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct RouteUiFields {
    pub route_details: Vec<TxnDetails>,
    // pub bridge: TxnDetails,
    pub initial_details: TxnDetails,
    pub local_total: Amount,
}

#[derive(Debug)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct TxnDetails {
    pub transaction: Transaction,
    pub eip1559: Eip1559EstimationFields,
    pub transaction_fee: TransactionFee,
}

#[derive(Debug)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct Eip1559EstimationFields {
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
}

#[derive(Debug)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Record))]
pub struct TransactionFee {
    pub fee: Amount,
    pub local_fee: Amount,
}

impl From<Eip1559Estimation> for Eip1559EstimationFields {
    fn from(estimation: Eip1559Estimation) -> Self {
        Eip1559EstimationFields {
            max_fee_per_gas: estimation.max_fee_per_gas.to_string(),
            max_priority_fee_per_gas: estimation
                .max_priority_fee_per_gas
                .to_string(),
        }
    }
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
