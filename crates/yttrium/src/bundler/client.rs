use {
    super::{config::BundlerConfig, models::estimate_result::EstimateResult},
    crate::{
        entry_point::EntryPointAddress,
        erc4337::get_user_operation_receipt,
        jsonrpc::{JSONRPCResponse, Request, Response},
        time::{Duration, Instant, sleep},
        user_operation::UserOperationV07,
    },
    alloy::{primitives::Bytes, rpc::types::UserOperationReceipt},
    alloy_provider::ProviderBuilder,
    eyre::Ok,
    serde_json::{self},
    tracing::debug,
};

pub struct BundlerClient {
    client: reqwest::Client,
    config: BundlerConfig,
}

impl BundlerClient {
    pub fn new(config: BundlerConfig) -> Self {
        Self { client: reqwest::Client::new(), config }
    }

    pub async fn send_user_operation(
        &self,
        entry_point_address: EntryPointAddress,
        user_op: UserOperationV07,
    ) -> eyre::Result<Bytes> {
        let send_body = crate::jsonrpc::Request {
            jsonrpc: "2.0".into(), // TODO use Arc<str>
            id: 1,
            method: "eth_sendUserOperation".into(), // TODO use Arc<str>
            params: vec![
                serde_json::to_value(&user_op)?,
                entry_point_address.to_string().into(),
            ],
        };

        let response = self
            .client
            .post(self.config.url())
            .json(&send_body)
            .send()
            .await?
            .text()
            .await?;

        debug!("response: {}", response);

        let response: Response<Bytes> =
            serde_json::from_str::<JSONRPCResponse<Bytes>>(&response)?.into();

        response?.ok_or(eyre::eyre!("send_user_operation got None"))
    }

    pub async fn estimate_user_operation_gas(
        &self,
        entry_point_address: EntryPointAddress,
        user_op: UserOperationV07,
    ) -> eyre::Result<EstimateResult> {
        let req_body = Request {
            jsonrpc: "2.0".into(), // TODO use Arc<str>
            id: 1,
            method: "eth_estimateUserOperationGas".into(), // TODO use Arc<str>
            params: vec![
                serde_json::to_value(&user_op)?,
                entry_point_address.to_string().into(),
            ],
        };

        let response: Response<EstimateResult> = self
            .client
            .post(self.config.url())
            .json(&req_body)
            .send()
            .await?
            .json::<JSONRPCResponse<EstimateResult>>()
            .await?
            .into();

        response?.ok_or(eyre::eyre!("estimate_user_operation_gas got None"))
    }

    pub async fn supported_entry_points(
        &self,
        op: String,
    ) -> eyre::Result<String> {
        Ok(op)
    }

    pub async fn chain_id(&self, op: String) -> eyre::Result<String> {
        Ok(op)
    }

    pub async fn get_user_operation_by_hash(
        &self,
        op: String,
    ) -> eyre::Result<String> {
        Ok(op)
    }

    pub async fn get_user_operation_receipt(
        &self,
        hash: Bytes,
    ) -> eyre::Result<Option<UserOperationReceipt>> {
        let provider = ProviderBuilder::new().on_http(self.config.url());
        get_user_operation_receipt(&provider, hash).await.map_err(Into::into)
    }

    pub async fn wait_for_user_operation_receipt(
        &self,
        hash: Bytes,
    ) -> eyre::Result<UserOperationReceipt> {
        let polling_interval = Duration::from_millis(2000);
        let timeout = Some(Duration::from_secs(30));

        let start_time = Instant::now();

        loop {
            match self.get_user_operation_receipt(hash.clone()).await? {
                Some(receipt) => return Ok(receipt),
                None => {
                    if let Some(timeout_duration) = timeout {
                        if start_time.elapsed() > timeout_duration {
                            return Err(eyre::eyre!(
                                "Timeout waiting for user operation receipt",
                            ));
                        }
                    }
                    println!(
                        "No Receipt yet. Trying again in {:?}",
                        polling_interval.as_millis()
                    );
                    sleep(polling_interval).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            bundler::models::estimate_result::EstimateResult, entry_point,
        },
        alloy::primitives::{Address, Bytes, U256},
        eyre::ensure,
    };

    pub async fn setup_gas_estimation_bundler_mock()
    -> eyre::Result<BundlerClient> {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{method, path},
        };

        let mock_server = MockServer::start().await;

        let expected_request_body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "eth_estimateUserOperationGas",
        });

        let response_estimate = EstimateResult {
            pre_verification_gas: U256::from(100000),
            verification_gas_limit: U256::from(100000),
            call_gas_limit: U256::from(100000),
            paymaster_verification_gas_limit: None,
            paymaster_post_op_gas_limit: None,
        };

        let response_body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "result": response_estimate,
        });

        let response = ResponseTemplate::new(200).set_body_json(response_body);

        use wiremock::matchers::body_partial_json;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_partial_json(&expected_request_body))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let bundler_client =
            BundlerClient::new(BundlerConfig::new(mock_server.uri().parse()?));

        Ok(bundler_client)
    }

    #[tokio::test]
    async fn test_estimate_gas() -> eyre::Result<()> {
        let sender: Address =
            "0x5FbDB2315678afecb367f032d93F642f64180aa3".parse()?;

        let entry_point_address =
            entry_point::EntryPointConfig::V07_SEPOLIA.address();

        let bundler_client = setup_gas_estimation_bundler_mock().await?;

        let user_op = {
            let sender = sender.into();
            let nonce: U256 = U256::from(0);
            let factory: Address = Address::ZERO;
            let factory_data: Bytes = Bytes::new();
            let call_data: Bytes = Bytes::new();
            let call_gas_limit: U256 = U256::from(100000);
            let verification_gas_limit: U256 = U256::from(100000);
            let pre_verification_gas: U256 = U256::from(100000);
            let max_fee_per_gas: U256 = U256::from(100000);
            let max_priority_fee_per_gas: U256 = U256::from(100000);
            let paymaster: Option<Address> = None;
            let paymaster_data: Option<Bytes> = None;
            let signature: Bytes = Bytes::new();

            UserOperationV07 {
                sender,
                nonce,
                factory: factory.into(),
                factory_data: factory_data.into(),
                call_data,
                call_gas_limit,
                verification_gas_limit,
                paymaster_post_op_gas_limit: Some(U256::from(100000)),
                paymaster_verification_gas_limit: Some(U256::from(100000)),
                pre_verification_gas,
                max_fee_per_gas,
                max_priority_fee_per_gas,
                paymaster,
                paymaster_data,
                // authorization_list: None,
                signature,
            }
        };

        let estimate_result = bundler_client
            .estimate_user_operation_gas(entry_point_address, user_op)
            .await?;

        ensure!(estimate_result.call_gas_limit == U256::from(100000));

        Ok(())
    }
}
