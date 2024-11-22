use super::dart_ffi;

impl From<dart_ffi::FFIEndpoint> for yttrium::config::Endpoint {
    fn from(val: dart_ffi::FFIEndpoint) -> Self {
        yttrium::config::Endpoint {
            api_key: val.api_key,
            base_url: val.base_url,
        }
    }
}

impl From<dart_ffi::FFIEndpoints> for yttrium::config::Endpoints {
    fn from(val: dart_ffi::FFIEndpoints) -> Self {
        yttrium::config::Endpoints {
            rpc: val.rpc.into(),
            bundler: val.bundler.into(),
            paymaster: val.paymaster.into(),
        }
    }
}

impl From<dart_ffi::FFIConfig> for yttrium::config::Config {
    fn from(val: dart_ffi::FFIConfig) -> Self {
        yttrium::config::Config { endpoints: val.endpoints.into() }
    }
}
