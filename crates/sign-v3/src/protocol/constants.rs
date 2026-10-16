//! sign-v3-spec reference/constants.md. Absolute times are Unix seconds; `t` and `n` are milliseconds.

pub const ANSWER_WINDOW_S: u64 = 5 * 60;
pub const MAX_LIFETIME_S: u64 = 7 * 24 * 60 * 60;
pub const LOG_RETENTION_S: u64 = 7 * 24 * 60 * 60;
pub const MAX_DEADLINE_S: u64 = 7 * 24 * 60 * 60;
pub const DEDUP_WINDOW_S: u64 = 5 * 60;
pub const EVENT_LOG_CAP: u32 = 100;
pub const MIN_TTL_S: u64 = 30;
pub const MAX_TTL_S: u64 = 30 * 24 * 60 * 60;
pub const CLOCK_SKEW_TOLERANCE_S: u64 = 5 * 60;
pub const FIRST_LINE_TIMEOUT_MS: u64 = 10_000;
pub const MAX_HOLD_MS: u64 = 5 * 60_000;
pub const FAST_FAILURE_MS: u64 = 200;
pub const JWT_LIFETIME_S: u64 = 24 * 60 * 60;
pub const SUPPORTED_VERSIONS: &[u32] = &[3];
pub const DEFAULT_DEADLINE_S: u64 = 5 * 60;
pub const REPLAY_WINDOW: u64 = 1024;
/// The Relay announces no expiry: past a deadline or the answer window, plus this, the SDK gives up.
pub const LOCAL_GRACE_S: u64 = 30;
