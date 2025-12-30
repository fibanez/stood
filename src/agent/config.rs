//! Execution configuration for the Agent interface.
//!
//! This module provides the [`ExecutionConfig`] type that configures how
//! the agent executes tasks, including callback handlers and EventLoop settings.

use crate::agent::callbacks::{CallbackHandler, CallbackHandlerConfig, PrintingConfig};
use crate::agent::event_loop::EventLoopConfig;
use std::sync::Arc;
use std::time::Duration;

/// Log level for controlling debug output from the agent.
///
/// This controls the tracing::debug! statements within the Agent and EventLoop
/// to provide visibility into execution without the need for println! statements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogLevel {
    /// No debug logging
    #[default]
    Off,
    /// Basic execution flow logging
    Info,
    /// Detailed step-by-step logging
    Debug,
    /// Verbose logging with full details
    Trace,
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Off => tracing::Level::ERROR, // Suppress all warnings and info messages
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
        }
    }
}

/// Execution configuration with built-in callback support
///
/// This configuration is set during Agent construction via the builder pattern
/// and controls all aspects of execution behavior, including callbacks,
/// streaming, timeouts, and EventLoop settings.
///
/// # Examples
///
/// Create default config (silent execution):
/// ```
/// use stood::agent::config::ExecutionConfig;
///
/// let config = ExecutionConfig::default(); // No callbacks, streaming enabled
/// ```
///
/// Create config with printing callbacks:
/// ```
/// use stood::agent::config::ExecutionConfig;
///
/// let config = ExecutionConfig::with_printing();
/// ```
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Callback handler configuration (defaults to None)
    pub callback_handler: CallbackHandlerConfig,

    /// EventLoop configuration for agentic execution
    pub event_loop: EventLoopConfig,

    /// Whether to enable streaming responses
    pub streaming: bool,

    /// Maximum execution time
    pub timeout: Option<Duration>,

    /// Log level for controlling debug output from the agent
    pub log_level: LogLevel,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            callback_handler: CallbackHandlerConfig::None, // No-op by default
            event_loop: EventLoopConfig::default(),
            streaming: true,
            timeout: Some(Duration::from_secs(300)), // 5 minutes
            log_level: LogLevel::default(),
        }
    }
}

impl ExecutionConfig {
    /// Create config with printing callbacks (matches Python's PrintingCallbackHandler)
    pub fn with_printing() -> Self {
        Self {
            callback_handler: CallbackHandlerConfig::Printing(PrintingConfig::default()),
            ..Default::default()
        }
    }

    /// Create config with verbose printing (matches Python's detailed output)
    pub fn verbose() -> Self {
        Self {
            callback_handler: CallbackHandlerConfig::Printing(PrintingConfig::verbose()),
            ..Default::default()
        }
    }

    /// Create config with silent execution (no callbacks)
    pub fn silent() -> Self {
        Self {
            callback_handler: CallbackHandlerConfig::None,
            ..Default::default()
        }
    }

    /// Create config with minimal printing
    pub fn minimal() -> Self {
        Self {
            callback_handler: CallbackHandlerConfig::Printing(PrintingConfig::minimal()),
            ..Default::default()
        }
    }

    /// Create config with custom handler
    pub fn with_handler(handler: Arc<dyn CallbackHandler>) -> Self {
        Self {
            callback_handler: CallbackHandlerConfig::Custom(handler),
            ..Default::default()
        }
    }

    /// Create config with multiple handlers (matches Python's CompositeCallbackHandler)
    pub fn with_composite(handlers: Vec<CallbackHandlerConfig>) -> Self {
        Self {
            callback_handler: CallbackHandlerConfig::Composite(handlers),
            ..Default::default()
        }
    }

    /// Create config with specific timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout: Some(timeout),
            ..Default::default()
        }
    }

    /// Create config with performance callbacks
    pub fn with_performance(level: tracing::Level) -> Self {
        Self {
            callback_handler: CallbackHandlerConfig::Performance(level),
            ..Default::default()
        }
    }

    /// Builder pattern methods for chaining
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn streaming(mut self, enabled: bool) -> Self {
        self.streaming = enabled;
        self
    }

    pub fn event_loop_config(mut self, config: EventLoopConfig) -> Self {
        self.event_loop = config;
        self
    }

    /// Set the log level for debug output
    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }

    /// Create config with specific log level
    pub fn with_log_level(level: LogLevel) -> Self {
        Self {
            log_level: level,
            ..Default::default()
        }
    }
}
