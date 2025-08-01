use {
    relay_rpc::{
        auth::RELAY_WEBSOCKET_ADDRESS, domain::ProjectId, user_agent::UserAgent,
    },
    serde::Serialize,
    url::Url,
};

// copied from https://github.com/reown-com/reown-rust/blob/b2ebad075297abd717e4951f566938dc4377b17c/relay_client/src/lib.rs#L36

/// Relay connection options.
#[derive(Debug, Clone)]
pub struct ConnectionOptions {
    /// The Relay websocket address. The default address is
    /// `wss://relay.walletconnect.com`.
    pub address: String,

    /// The project-specific secret key. Can be generated in the Cloud Dashboard
    /// at the following URL: <https://cloud.walletconnect.com/app>
    pub project_id: ProjectId,

    /// The authorization method and auth token to use.
    pub auth: String,

    /// Optional origin of the request. Subject to allow-list validation.
    pub origin: Option<String>,

    /// Optional package name. Used instead of `origin` for allow-list
    /// validation.
    pub package_name: Option<String>,

    /// Optional bundle ID. Used instead of `origin` for allow-list validation.
    pub bundle_id: Option<String>,

    /// Optional user agent parameters.
    pub user_agent: Option<UserAgent>,
}

impl ConnectionOptions {
    pub fn new(project_id: impl Into<ProjectId>, auth: String) -> Self {
        Self {
            address: RELAY_WEBSOCKET_ADDRESS.into(),
            project_id: project_id.into(),
            auth,
            origin: None,
            user_agent: None,
            package_name: None,
            bundle_id: None,
        }
    }

    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        self.address = address.into();
        self
    }

    #[allow(unused)]
    pub fn with_package_name(
        mut self,
        package_name: impl Into<String>,
    ) -> Self {
        self.package_name = Some(package_name.into());
        self
    }

    #[allow(unused)]
    pub fn with_bundle_id(mut self, bundle_id: impl Into<String>) -> Self {
        self.bundle_id = Some(bundle_id.into());
        self
    }

    #[allow(unused)]
    pub fn with_origin(mut self, origin: impl Into<Option<String>>) -> Self {
        self.origin = origin.into();
        self
    }

    #[allow(unused)]
    pub fn with_user_agent(
        mut self,
        user_agent: impl Into<Option<UserAgent>>,
    ) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    pub fn as_url(&self) -> Result<Url, RequestBuildError> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct QueryParams<'a> {
            project_id: &'a ProjectId,
            auth: Option<&'a String>,
            ua: Option<&'a UserAgent>,
            package_name: Option<&'a str>,
            bundle_id: Option<&'a str>,
        }

        let query = QueryParams {
            project_id: &self.project_id,
            auth: Some(&self.auth),
            ua: self.user_agent.as_ref(),
            package_name: self.package_name.as_deref(),
            bundle_id: self.bundle_id.as_deref(),
        };

        let mut url =
            Url::parse(&self.address).map_err(RequestBuildError::Url)?;
        url.query_pairs_mut()
            .append_pair("projectId", query.project_id.as_ref());
        if let Some(auth) = query.auth {
            url.query_pairs_mut().append_pair("auth", &auth.to_string());
        }
        if let Some(ua) = query.ua {
            url.query_pairs_mut()
                .append_pair("ua", &serde_json::to_string(ua).unwrap());
        }
        if let Some(package_name) = query.package_name {
            url.query_pairs_mut().append_pair("packageName", package_name);
        }
        if let Some(bundle_id) = query.bundle_id {
            url.query_pairs_mut().append_pair("bundleId", bundle_id);
        }

        Ok(url)
    }
}

/// Errors generated while parsing
/// [`ConnectionOptions`][crate::ConnectionOptions] and creating an HTTP request
/// for the websocket connection.
#[derive(Debug, thiserror::Error)]
pub enum RequestBuildError {
    #[error("Failed to parse connection URL: {0}")]
    Url(#[from] url::ParseError),
}
