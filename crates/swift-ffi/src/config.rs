use super::ffi;

impl From<ffi::FFIEndpoint> for yttrium::config::Endpoint {
    fn from(val: ffi::FFIEndpoint) -> Self {
        yttrium::config::Endpoint {
            api_key: val.api_key,
            base_url: val.base_url,
        }
    }
}

impl From<ffi::FFIEndpoints> for yttrium::config::Endpoints {
    fn from(val: ffi::FFIEndpoints) -> Self {
        yttrium::config::Endpoints {
            rpc: val.rpc.into(),
            bundler: val.bundler.into(),
            paymaster: val.paymaster.into(),
        }
    }
}

impl From<ffi::FFIConfig> for yttrium::config::Config {
    fn from(val: ffi::FFIConfig) -> Self {
        yttrium::config::Config { endpoints: val.endpoints.into() }
    }
}
