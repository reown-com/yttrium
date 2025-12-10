pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";
pub const METHOD_NOT_FOUND: &str = "METHOD_NOT_FOUND";
pub const INVALID_PARAMS: &str = "INVALID_PARAMS";
pub const PARAMS_VALIDATION: &str = "PARAMS_VALIDATION";

// TODO differentiate between:
// - server errors (retryable)
// - client/validation errors (not retryable)
// - user-facing errors (not retryable)
