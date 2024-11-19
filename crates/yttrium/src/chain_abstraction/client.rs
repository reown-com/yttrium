use super::{
    api::{
        route::{
            RouteQueryParams, RouteRequest, RouteResponse, ROUTE_ENDPOINT_PATH,
        },
        status::{
            StatusQueryParams, StatusResponse, StatusResponseCompleted,
            STATUS_ENDPOINT_PATH,
        },
        Transaction,
    },
    error::{RouteError, WaitForSuccessError},
};
use relay_rpc::domain::ProjectId;
use reqwest::{Client as ReqwestClient, Url};
use std::time::{Duration, Instant};

pub struct Client {
    client: ReqwestClient,
    base_url: Url,
    project_id: ProjectId,
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

    pub async fn status(
        &self,
        orchestration_id: String,
    ) -> Result<StatusResponse, RouteError> {
        self.status_with_timeout(orchestration_id, Duration::from_secs(5)).await
    }

    pub async fn status_with_timeout(
        &self,
        orchestration_id: String,
        timeout: Duration,
    ) -> Result<StatusResponse, RouteError> {
        let response = self
            .client
            .get(self.base_url.join(STATUS_ENDPOINT_PATH).unwrap())
            .query(&StatusQueryParams {
                project_id: self.project_id.clone(),
                orchestration_id,
            })
            .timeout(timeout)
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

    pub async fn wait_for_success_with_timeout(
        &self,
        orchestration_id: String,
        check_in: Duration,
        timeout: Duration,
    ) -> Result<StatusResponseCompleted, WaitForSuccessError> {
        let start = Instant::now();
        tokio::time::sleep(check_in).await;
        loop {
            let remaining_time = timeout - start.elapsed();
            let request_timeout = remaining_time.max(Duration::from_secs(5));
            let result = self
                .status_with_timeout(orchestration_id.clone(), request_timeout)
                .await;
            let (error, check_in) = match result {
                Ok(status_response_success) => match status_response_success {
                    StatusResponse::Completed(completed) => {
                        return Ok(completed)
                    }
                    StatusResponse::Error(e) => {
                        return Err(WaitForSuccessError::StatusResponseError(e))
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
                }
            };
            if start.elapsed() > timeout {
                return Err(error);
            }
            tokio::time::sleep(check_in).await;
        }
    }
}
