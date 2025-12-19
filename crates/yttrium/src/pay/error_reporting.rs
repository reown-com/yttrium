use {
    reqwest::Client as HttpClient,
    serde::Serialize,
    std::{
        panic::PanicHookInfo,
        sync::OnceLock,
        time::{SystemTime, UNIX_EPOCH},
    },
    uuid::Uuid,
};

const PULSE_ENDPOINT: &str = "https://pulse.walletconnect.org/e";

static PANIC_CONFIG: OnceLock<PanicConfig> = OnceLock::new();

struct PanicConfig {
    bundle_id: String,
}

/// Install a global panic hook that reports panics to the pulse endpoint.
/// Should be called once during initialization (e.g., in WalletConnectPay::new).
/// Only the first call will have effect; subsequent calls are ignored.
pub fn install_panic_hook(bundle_id: String) {
    if PANIC_CONFIG.set(PanicConfig { bundle_id }).is_ok() {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            report_panic(panic_info);
            previous_hook(panic_info);
        }));
    }
}

fn report_panic(panic_info: &PanicHookInfo<'_>) {
    let Some(config) = PANIC_CONFIG.get() else {
        return;
    };

    let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic".to_string()
    };

    let location = panic_info
        .location()
        .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
        .unwrap_or_else(|| "unknown location".to_string());

    let backtrace = std::backtrace::Backtrace::capture().to_string();

    let trace = format!(
        "PANIC at {}\nMessage: {}\nBacktrace:\n{}",
        location, message, backtrace
    );

    let event = ErrorEvent {
        event_id: Uuid::new_v4().to_string(),
        bundle_id: config.bundle_id.clone(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        props: ErrorProps {
            event: "error",
            error_type: "Panic".to_string(),
            properties: ErrorProperties { topic: "pay_sdk".to_string(), trace },
        },
    };

    // Use a new thread with its own tokio runtime since we're in a panic hook
    #[cfg(not(test))]
    std::thread::spawn(move || {
        if let Ok(rt) =
            tokio::runtime::Builder::new_current_thread().enable_all().build()
        {
            rt.block_on(async {
                let client = HttpClient::new();
                let _ = client.post(PULSE_ENDPOINT).json(&event).send().await;
            });
        }
    });

    #[cfg(test)]
    {
        let _ = event; // Don't send in tests
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorEvent {
    event_id: String,
    bundle_id: String,
    timestamp: u64,
    props: ErrorProps,
}

#[derive(Debug, Serialize)]
struct ErrorProps {
    event: &'static str,
    #[serde(rename = "type")]
    error_type: String,
    properties: ErrorProperties,
}

#[derive(Debug, Serialize)]
struct ErrorProperties {
    topic: String,
    trace: String,
}

pub(crate) fn report_error(
    http_client: &HttpClient,
    bundle_id: &str,
    error_type: &str,
    topic: &str,
    trace: &str,
) {
    let event = ErrorEvent {
        event_id: Uuid::new_v4().to_string(),
        bundle_id: bundle_id.to_string(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        props: ErrorProps {
            event: "error",
            error_type: error_type.to_string(),
            properties: ErrorProperties {
                topic: topic.to_string(),
                trace: trace.to_string(),
            },
        },
    };

    let client = http_client.clone();
    let fut = async move {
        match client.post(PULSE_ENDPOINT).json(&event).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    tracing::debug!(
                        "Pay error reporting failed: {}",
                        response.status()
                    );
                }
            }
            Err(e) => {
                tracing::debug!("Pay error reporting failed: {}", e);
            }
        }
    };

    #[cfg(not(test))]
    crate::spawn::spawn(fut);

    #[cfg(test)]
    {
        let _ = fut; // Don't send in tests
    }
}

/// Helper to get error type name from an error enum
pub(crate) fn error_type_name<E: std::fmt::Debug>(error: &E) -> String {
    let debug = format!("{:?}", error);
    debug.split('(').next().unwrap_or("Unknown").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_type_name() {
        #[derive(Debug)]
        enum TestError {
            NotFound(String),
            InvalidRequest(String),
        }

        assert_eq!(
            error_type_name(&TestError::NotFound("test".to_string())),
            "NotFound"
        );
        assert_eq!(
            error_type_name(&TestError::InvalidRequest("msg".to_string())),
            "InvalidRequest"
        );
    }

    #[test]
    fn test_error_event_serialization() {
        let event = ErrorEvent {
            event_id: "test-id".to_string(),
            bundle_id: "com.example.app".to_string(),
            timestamp: 1234567890000,
            props: ErrorProps {
                event: "error",
                error_type: "PaymentNotFound".to_string(),
                properties: ErrorProperties {
                    topic: "pay_123".to_string(),
                    trace: "Error: Payment not found".to_string(),
                },
            },
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"eventId\":\"test-id\""));
        assert!(json.contains("\"bundleId\":\"com.example.app\""));
        assert!(json.contains("\"event\":\"error\""));
        assert!(json.contains("\"type\":\"PaymentNotFound\""));
        assert!(json.contains("\"topic\":\"pay_123\""));
        assert!(json.contains("\"trace\":\"Error: Payment not found\""));
    }
}
