use super::{
    api::{
        route::{
            RouteQueryParams, RouteRequest, RouteResponse, ROUTE_ENDPOINT_PATH,
        },
        status::{StatusQueryParams, StatusResponse, STATUS_ENDPOINT_PATH},
        Transaction,
    },
    error::RouteError,
};
use relay_rpc::domain::ProjectId;
use reqwest::{Client as ReqwestClient, Url};

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
        let response = self
            .client
            .get(self.base_url.join(STATUS_ENDPOINT_PATH).unwrap())
            .query(&StatusQueryParams {
                project_id: self.project_id.clone(),
                orchestration_id,
            })
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
}
