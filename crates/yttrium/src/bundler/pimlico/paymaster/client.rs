use super::models::{
    SponsorshipResponseV07, SponsorshipResultV07,
    UserOperationPreSponsorshipV07,
};
use crate::bundler::config::BundlerConfig;
use crate::entry_point::EntryPointAddress;
use crate::jsonrpc::{JSONRPCResponse, Request, Response};
use serde_json;
use tracing::debug;

pub struct PaymasterClient {
    client: reqwest::Client,
    config: BundlerConfig,
}

impl PaymasterClient {
    pub fn new(config: BundlerConfig) -> Self {
        Self { client: reqwest::Client::new(), config }
    }

    pub async fn sponsor_user_operation_v07(
        &self,
        user_operation: &UserOperationPreSponsorshipV07,
        entry_point: &EntryPointAddress,
        sponsorship_policy_id: Option<String>,
    ) -> eyre::Result<SponsorshipResultV07> {
        let params = {
            let mut params = vec![
                serde_json::to_value(user_operation)?,
                entry_point.to_address().to_string().into(),
            ];
            if let Some(sponsorship_policy_id) = sponsorship_policy_id {
                params.push(sponsorship_policy_id.into());
            }
            params
        };

        let req_body = Request {
            jsonrpc: "2.0".into(),
            id: 1,
            method: "pm_sponsorUserOperation".into(),
            params,
        };
        debug!("req_body: {:?}", serde_json::to_string(&req_body)?);

        let post =
            self.client.post(self.config.url()).json(&req_body).send().await?;
        debug!("pm_sponsorUserOperation post: {:?}", post);
        let res = post.text().await?;
        debug!("pm_sponsorUserOperation res: {:?}", res);
        let v = serde_json::from_str::<JSONRPCResponse<SponsorshipResponseV07>>(
            &res,
        )?;

        debug!("pm_sponsorUserOperation json: {:?}", v);

        let response: Response<SponsorshipResponseV07> = v.into();

        let response_estimate = response?;
        let response_estimate = response_estimate
            .ok_or(eyre::eyre!("response_estimate is None"))?;

        let result = SponsorshipResultV07 {
            call_gas_limit: response_estimate.call_gas_limit,
            verification_gas_limit: response_estimate.verification_gas_limit,
            pre_verification_gas: response_estimate.pre_verification_gas,
            paymaster: response_estimate.paymaster,
            paymaster_verification_gas_limit: response_estimate
                .paymaster_verification_gas_limit,
            paymaster_post_op_gas_limit: response_estimate
                .paymaster_post_op_gas_limit,
            paymaster_data: response_estimate.paymaster_data,
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Address;
    use eyre::ensure;

    pub async fn setup_sponsor_user_operation_v07_paymaster_mock(
    ) -> eyre::Result<PaymasterClient> {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        let url = mock_server.uri().to_string();

        let expected_request_body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "pm_sponsorUserOperation",
        });

        let sponsorship_payload = SponsorshipResponseV07::mock();

        let response_body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "result": sponsorship_payload,
        });

        let response = ResponseTemplate::new(200).set_body_json(response_body);

        use wiremock::matchers::body_partial_json;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_partial_json(&expected_request_body))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let bundler_client = PaymasterClient::new(BundlerConfig::new(url));

        Ok(bundler_client)
    }

    #[tokio::test]
    async fn test_sponsor_user_operation_v07() -> eyre::Result<()> {
        let paymaster_client =
            setup_sponsor_user_operation_v07_paymaster_mock().await?;

        let entry_point =
            "0x0000000071727De22E5E9d8BAf0edAc6f37da032".parse::<Address>()?;
        let entry_point_address =
            crate::entry_point::EntryPointAddress::new(entry_point);

        let user_operation = crate::user_operation::UserOperationV07::mock();
        let user_operation_pre =
            UserOperationPreSponsorshipV07::from(user_operation);

        let sponsorship_result = paymaster_client
            .sponsor_user_operation_v07(
                &user_operation_pre,
                &entry_point_address,
                None,
            )
            .await?;

        ensure!(sponsorship_result.call_gas_limit.to_string() == "100000");

        Ok(())
    }
}
