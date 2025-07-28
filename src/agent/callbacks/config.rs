//! Configuration types for the callback system.
//!
//! This module provides configuration types that enable type-safe setup
//! of callback handlers during agent construction.

use std::sync::Arc;
use super::traits::CallbackHandler;

/// Callback handler configuration enum for type safety
///
/// This enum provides type-safe configuration of callback handlers during
/// agent construction, enabling easy setup of different callback scenarios.
#[derive(Clone)]
pub enum CallbackHandlerConfig {
    /// No callbacks (default)
    None,
    
    /// Built-in printing handler with configuration
    Printing(PrintingConfig),
    
    /// Custom handler (type-erased for flexibility)
    Custom(Arc<dyn CallbackHandler>),
    
    /// Multiple handlers composed together
    Composite(Vec<CallbackHandlerConfig>),
    
    /// Performance logging handler
    Performance(tracing::Level),
    
    /// Batching wrapper around another handler for performance optimization
    Batching {
        inner: Box<CallbackHandlerConfig>,
        batch_config: super::batching::BatchConfig,
    },
}

impl std::fmt::Debug for CallbackHandlerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallbackHandlerConfig::None => write!(f, "None"),
            CallbackHandlerConfig::Printing(config) => f.debug_tuple("Printing").field(config).finish(),
            CallbackHandlerConfig::Custom(_) => write!(f, "Custom(Arc<dyn CallbackHandler>)"),
            CallbackHandlerConfig::Composite(handlers) => f.debug_tuple("Composite").field(handlers).finish(),
            CallbackHandlerConfig::Performance(level) => f.debug_tuple("Performance").field(level).finish(),
            CallbackHandlerConfig::Batching { inner, batch_config } => {
                f.debug_struct("Batching")
                    .field("inner", inner)
                    .field("batch_config", batch_config)
                    .finish()
            },
        }
    }
}

/// Configuration for printing callback handler
///
/// This struct controls how the built-in printing handler displays
/// information during agent execution.
#[derive(Debug, Clone)]
pub struct PrintingConfig {
    /// Show reasoning text (matches Python's reasoningText handling)
    pub show_reasoning: bool,
    
    /// Show tool execution details (matches Python's current_tool_use handling)
    pub show_tools: bool,
    
    /// Show performance metrics at completion
    pub show_performance: bool,
    
    /// Stream output in real-time (matches Python's data streaming)
    pub stream_output: bool,
}

impl Default for PrintingConfig {
    fn default() -> Self {
        Self {
            show_reasoning: false,
            show_tools: true,
            show_performance: false,
            stream_output: true,
        }
    }
}

impl PrintingConfig {
    /// Create config optimized for development/debugging
    pub fn verbose() -> Self {
        Self {
            show_reasoning: true,
            show_tools: true,
            show_performance: true,
            stream_output: true,
        }
    }
    
    /// Create config for production/clean output
    pub fn minimal() -> Self {
        Self {
            show_reasoning: false,
            show_tools: false,
            show_performance: false,
            stream_output: true,
        }
    }
    
    /// Create config for silent operation
    pub fn silent() -> Self {
        Self {
            show_reasoning: false,
            show_tools: false,
            show_performance: false,
            stream_output: false,
        }
    }
}

/// Validation for callback configurations
impl CallbackHandlerConfig {
    /// Validate the configuration for correctness
    pub fn validate(&self) -> Result<(), String> {
        if let CallbackHandlerConfig::Composite(handlers) = self {
            if handlers.is_empty() {
                return Err("Composite handler requires at least one handler".to_string());
            }
            for handler in handlers {
                handler.validate()?;
            }
        }
        // Other variants are always valid
        Ok(())
    }
}