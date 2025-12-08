use {
    crate::pay::WalletConnectPay as CoreWalletConnectPay,
    relay_rpc::domain::ProjectId,
};

#[derive(uniffi::Object)]
pub struct WalletConnectPay {
    _inner: CoreWalletConnectPay,
}

#[uniffi::export]
impl WalletConnectPay {
    #[uniffi::constructor]
    pub fn new(project_id: ProjectId) -> Self {
        Self { _inner: CoreWalletConnectPay::new(project_id) }
    }
}
