use {
    reqwest::Client as HttpClient,
    serde::Serialize,
    std::{
        panic::PanicHookInfo,
        sync::{mpsc, OnceLock},
        time::{SystemTime, UNIX_EPOCH},
    },
    uuid::Uuid,
};

#[cfg(test)]
use std::sync::Mutex;

const PULSE_BASE_URL: &str = "https://pulse.walletconnect.org/e";

static PANIC_CONFIG: OnceLock<PanicConfig> = OnceLock::new();
static PANIC_SENDER: OnceLock<mpsc::Sender<PanicEvent>> = OnceLock::new();

/// Captured panic events for testing (only populated in test builds)
#[cfg(test)]
static CAPTURED_PANIC_EVENTS: OnceLock<Mutex<Vec<CapturedPanicEvent>>> = OnceLock::new();

/// A captured panic event for test verification
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct CapturedPanicEvent {
    pub bundle_id: String,
    pub project_id: String,
    pub sdk_name: String,
    pub sdk_version: String,
    pub message: String,
    pub location: String,
}

struct PanicConfig {
    bundle_id: String,
    project_id: String,
    sdk_name: String,
    sdk_version: String,
}

struct PanicEvent {
    event: ErrorEvent,
    url: String,
    user_agent: String,
}

fn build_pulse_url(project_id: &str, sdk_version: &str) -> String {
    format!(
        "{}?projectId={}&st=pay_sdk&sv={}",
        PULSE_BASE_URL, project_id, sdk_version
    )
}

/// Install a global panic hook that reports panics to the pulse endpoint.
/// Should be called once during initialization (e.g., in WalletConnectPay::new).
/// Only the first call will have effect; subsequent calls are ignored.
/// Spawns a single background thread to handle panic reports via a bounded channel.
#[cfg(not(test))]
pub fn install_panic_hook(
    bundle_id: String,
    project_id: String,
    sdk_name: String,
    sdk_version: String,
) {
    if PANIC_CONFIG
        .set(PanicConfig { bundle_id, project_id, sdk_name, sdk_version })
        .is_ok()
    {
        // Create a bounded channel to prevent unbounded memory growth
        let (sender, receiver) = mpsc::channel::<PanicEvent>();
        let _ = PANIC_SENDER.set(sender);

        // Spawn a single background thread to process panic events
        std::thread::spawn(move || {
            if let Ok(rt) =
                tokio::runtime::Builder::new_current_thread().enable_all().build()
            {
                rt.block_on(async {
                    let client = HttpClient::new();
                    while let Ok(panic_event) = receiver.recv() {
                        let _ = client
                            .post(&panic_event.url)
                            .header("User-Agent", &panic_event.user_agent)
                            .json(&panic_event.event)
                            .send()
                            .await;
                    }
                });
            }
        });

        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            report_panic(panic_info);
            previous_hook(panic_info);
        }));
    }
}

/// Test version: installs a panic hook that captures events for verification.
/// Use `get_captured_panic_events()` to retrieve captured events in tests.
#[cfg(test)]
pub fn install_panic_hook(
    bundle_id: String,
    project_id: String,
    sdk_name: String,
    sdk_version: String,
) {
    // Initialize storage and config
    let _ = CAPTURED_PANIC_EVENTS.get_or_init(|| Mutex::new(Vec::new()));
    if PANIC_CONFIG
        .set(PanicConfig {
            bundle_id: bundle_id.clone(),
            project_id: project_id.clone(),
            sdk_name: sdk_name.clone(),
            sdk_version: sdk_version.clone(),
        })
        .is_ok()
    {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            // Capture panic info for test verification
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

            if let Some(storage) = CAPTURED_PANIC_EVENTS.get() {
                if let Ok(mut events) = storage.lock() {
                    events.push(CapturedPanicEvent {
                        bundle_id: bundle_id.clone(),
                        project_id: project_id.clone(),
                        sdk_name: sdk_name.clone(),
                        sdk_version: sdk_version.clone(),
                        message,
                        location,
                    });
                }
            }

            previous_hook(panic_info);
        }));
    }
}

/// Get captured panic events (test only).
/// Returns a clone of all captured events.
#[cfg(test)]
pub fn get_captured_panic_events() -> Vec<CapturedPanicEvent> {
    CAPTURED_PANIC_EVENTS
        .get()
        .and_then(|storage| storage.lock().ok())
        .map(|events| events.clone())
        .unwrap_or_default()
}

/// Clear captured panic events (test only).
#[cfg(test)]
pub fn clear_captured_panic_events() {
    if let Some(storage) = CAPTURED_PANIC_EVENTS.get() {
        if let Ok(mut events) = storage.lock() {
            events.clear();
        }
    }
}

fn report_panic(panic_info: &PanicHookInfo<'_>) {
    let Some(config) = PANIC_CONFIG.get() else {
        return;
    };
    let Some(sender) = PANIC_SENDER.get() else {
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

    let url = build_pulse_url(&config.project_id, &config.sdk_version);
    let user_agent = format!("{}/{}", config.sdk_name, config.sdk_version);

    // Send event to the background thread via channel (non-blocking)
    let _ = sender.send(PanicEvent { event, url, user_agent });
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
    project_id: &str,
    sdk_name: &str,
    sdk_version: &str,
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

    let url = build_pulse_url(project_id, sdk_version);
    let user_agent = format!("{}/{}", sdk_name, sdk_version);
    let client = http_client.clone();
    let fut = async move {
        match client
            .post(&url)
            .header("User-Agent", user_agent)
            .json(&event)
            .send()
            .await
        {
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
    drop(fut); // Don't send in tests
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
    fn test_build_pulse_url() {
        let url = build_pulse_url("123", "rust-0.1.0");
        assert_eq!(
            url,
            "https://pulse.walletconnect.org/e?projectId=123&st=pay_sdk&sv=rust-0.1.0"
        );
    }

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

    #[tokio::test]
    #[ignore] // Run with: PROJECT_ID=your_project_id cargo +nightly test -p yttrium --features=pay test_real_error_event -- --ignored --nocapture
    async fn test_real_error_event() {
        let project_id = std::env::var("PROJECT_ID")
            .expect("PROJECT_ID environment variable must be set");
        let sdk_name = "pay_sdk";
        let sdk_version = "rust-0.1.0";
        let bundle_id = "com.test.yttrium";

        let event = ErrorEvent {
            event_id: Uuid::new_v4().to_string(),
            bundle_id: bundle_id.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            props: ErrorProps {
                event: "ERROR",
                error_type: "TestError".to_string(),
                properties: ErrorProperties {
                    topic: "pay_sdk_integration_test".to_string(),
                    trace: "This is a test error from yttrium pay SDK integration test".to_string(),
                },
            },
        };

        let url = build_pulse_url(&project_id, sdk_version);
        println!("Sending to URL: {}", url);
        println!("Event JSON: {}", serde_json::to_string_pretty(&event).unwrap());

        let client = HttpClient::builder()
            .user_agent(format!("{}/{}", sdk_name, sdk_version))
            .build()
            .unwrap();
        let response = client
            .post(&url)
            .header("Origin", "https://test.walletconnect.com")
            .header("x-sdk-type", sdk_name)
            .header("x-sdk-version", sdk_version)
            .header("x-bundle-id", bundle_id)
            .json(&event)
            .send()
            .await;

        assert!(response.is_ok(), "Failed to send error event: {:?}", response.err());
        let resp = response.unwrap();
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        assert!(
            status.is_success(),
            "Error event request failed with status: {}, body: {}",
            status,
            body
        );
        println!("Success! Response: {}", body);
    }

    #[test]
    fn test_panic_hook_captures_event() {
        // Clear any previous events
        clear_captured_panic_events();

        // Install panic hook
        install_panic_hook(
            "com.test.panic".to_string(),
            "test-project-id".to_string(),
            "test-sdk".to_string(),
            "1.0.0".to_string(),
        );

        // Trigger a panic in catch_unwind
        let result = std::panic::catch_unwind(|| {
            panic!("Test panic message for hook verification");
        });

        assert!(result.is_err(), "Panic should have been caught");

        // Verify the panic was captured
        let events = get_captured_panic_events();
        assert!(!events.is_empty(), "Panic event should have been captured");

        let event = &events[events.len() - 1]; // Get the last event
        assert_eq!(event.bundle_id, "com.test.panic");
        assert_eq!(event.project_id, "test-project-id");
        assert_eq!(event.sdk_name, "test-sdk");
        assert_eq!(event.sdk_version, "1.0.0");
        assert!(
            event.message.contains("Test panic message"),
            "Message should contain panic text: {}",
            event.message
        );
        assert!(
            event.location.contains("error_reporting.rs"),
            "Location should contain file name: {}",
            event.location
        );
    }
}
