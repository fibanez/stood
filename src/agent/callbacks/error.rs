//! Error types for the callback system.

/// Errors that can occur during callback execution
#[derive(Debug, thiserror::Error)]
pub enum CallbackError {
    #[error("Callback execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Callback handler not found: {0}")]
    HandlerNotFound(String),

    #[error("Callback configuration error: {0}")]
    ConfigurationError(String),
}
