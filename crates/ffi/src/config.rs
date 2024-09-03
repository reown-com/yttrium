use super::ffi;

impl Into<yttrium::config::Endpoint> for ffi::FFIEndpoint {
    fn into(self) -> yttrium::config::Endpoint {
        yttrium::config::Endpoint {
            api_key: self.api_key,
            base_url: self.base_url,
        }
    }
}

impl Into<yttrium::config::Endpoints> for ffi::FFIEndpoints {
    fn into(self) -> yttrium::config::Endpoints {
        yttrium::config::Endpoints {
            rpc: self.rpc.into(),
            bundler: self.bundler.into(),
            paymaster: self.paymaster.into(),
        }
    }
}

impl Into<yttrium::config::Config> for ffi::FFIConfig {
    fn into(self) -> yttrium::config::Config {
        yttrium::config::Config { endpoints: self.endpoints.into() }
    }
}
