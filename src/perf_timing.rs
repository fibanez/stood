//! Debug-only performance timing for the stood library.
//!
//! When the `perf-timing` feature is enabled, this module outputs timing
//! data to a log file for performance analysis.
//!
//! # Configuration
//!
//! Log path can be customized via the `PERF_TIMING_LOG_PATH` environment variable.
//! If not set, defaults to `{data_local_dir}/stood/logs/perf_timing.log`
//!
//! # Usage
//!
//! ```rust,ignore
//! use stood::{perf_timed, perf_checkpoint, perf_guard};
//!
//! // Time a block of code
//! let result = perf_timed!("stood.operation_name", {
//!     expensive_operation().await
//! });
//!
//! // Mark a checkpoint
//! perf_checkpoint!("stood.checkpoint_name");
//!
//! // Create a timing guard (RAII)
//! let _guard = perf_guard!("stood.guarded_operation");
//! // ... code here is timed until _guard is dropped
//! ```

use once_cell::sync::Lazy;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

/// Global log file handle
static LOG_FILE: Lazy<Mutex<Option<std::fs::File>>> = Lazy::new(|| {
    let log_path = get_log_path();
    // Ensure parent directory exists
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok();
    Mutex::new(file)
});

fn get_log_path() -> PathBuf {
    // Allow custom path via environment variable
    if let Ok(custom_path) = std::env::var("PERF_TIMING_LOG_PATH") {
        return PathBuf::from(custom_path);
    }

    // Default to generic stood directory (not application-specific)
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("stood")
        .join("logs")
        .join("perf_timing.log")
}

/// Write a timing entry to the log
pub fn log_timing(name: &str, duration_ms: f64, context: Option<&str>) {
    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(ref mut file) = *guard {
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let ctx = context.map(|c| format!(" [{}]", c)).unwrap_or_default();
            let _ = writeln!(file, "[{}] {} = {:.3}ms{}", timestamp, name, duration_ms, ctx);
            let _ = file.flush();
        }
    }
}

/// Write a checkpoint (marker) to the log
pub fn log_checkpoint(name: &str) {
    log_checkpoint_with_context(name, None);
}

/// Write a checkpoint with optional context to the log
pub fn log_checkpoint_with_context(name: &str, context: Option<&str>) {
    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(ref mut file) = *guard {
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let ctx = context.map(|c| format!(" [{}]", c)).unwrap_or_default();
            let _ = writeln!(file, "[{}] CHECKPOINT: {}{}", timestamp, name, ctx);
            let _ = file.flush();
        }
    }
}

/// RAII timing guard that logs duration when dropped
pub struct TimingGuard {
    name: String,
    start: Instant,
    context: Option<String>,
}

impl TimingGuard {
    /// Create a new timing guard
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            context: None,
        }
    }

    /// Create a new timing guard with context
    pub fn with_context(name: impl Into<String>, context: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            context: Some(context.into()),
        }
    }
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        log_timing(
            &self.name,
            duration.as_secs_f64() * 1000.0,
            self.context.as_deref(),
        );
    }
}

/// Macro for timing a block of code
///
/// # Example
/// ```rust,ignore
/// let result = perf_timed!("stood.bedrock.aws_config_load", {
///     config_loader.load().await
/// });
/// ```
#[macro_export]
macro_rules! perf_timed {
    ($name:expr, $expr:expr) => {{
        let _guard = $crate::perf_timing::TimingGuard::new($name);
        $expr
    }};
}

/// Macro for checkpoints (markers in the log)
///
/// # Example
/// ```rust,ignore
/// perf_checkpoint!("stood.agent_builder.build.start");
/// perf_checkpoint!("stood.operation", &format!("count={}", count));
/// ```
#[macro_export]
macro_rules! perf_checkpoint {
    ($name:expr) => {
        $crate::perf_timing::log_checkpoint($name);
    };
    ($name:expr, $context:expr) => {
        $crate::perf_timing::log_checkpoint_with_context($name, Some($context));
    };
}

/// Macro for creating a timing guard (RAII pattern)
///
/// # Example
/// ```rust,ignore
/// let _guard = perf_guard!("stood.operation");
/// // ... code here is timed until _guard is dropped
/// ```
#[macro_export]
macro_rules! perf_guard {
    ($name:expr) => {
        $crate::perf_timing::TimingGuard::new($name)
    };
    ($name:expr, $context:expr) => {
        $crate::perf_timing::TimingGuard::with_context($name, $context)
    };
}
