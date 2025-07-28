//! Callback system for Agent execution events.
//!
//! This module provides a comprehensive callback system that allows users to
//! receive real-time updates during agent execution, including content streaming,
//! tool usage, and performance metrics.

pub mod traits;
pub mod events;
pub mod handlers;
pub mod config;
pub mod error;

#[cfg(feature = "benchmarks")]
pub mod benchmarks;

pub mod batching;

pub use traits::{CallbackHandler, SyncCallbackHandler};
pub use events::{CallbackEvent, ToolEvent, TokenUsage};
pub use handlers::{NullCallbackHandler, PrintingCallbackHandler, CompositeCallbackHandler, PerformanceCallbackHandler};
pub use config::{CallbackHandlerConfig, PrintingConfig};
pub use error::CallbackError;
pub use batching::{BatchingCallbackHandler, BatchConfig, EventBatch};