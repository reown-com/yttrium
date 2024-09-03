#[cfg(target_os = "ios")]
use crate::ffi::FFIError;
#[cfg(target_os = "ios")]
use oslog;
#[cfg(target_os = "ios")]
use oslog::OsLogger;

#[cfg(target_os = "ios")]
pub fn init_os_logger() -> Result<(), FFIError> {
    #[cfg(debug_assertions)]
    let level = log::LevelFilter::Debug;
    #[cfg(not(debug_assertions))]
    let level = log::LevelFilter::Info;

    OsLogger::new("com.walletconnect.YttriumCore")
        .level_filter(level)
        .init()
        .map_err(|e| FFIError::Unknown(e.to_string()))
}
