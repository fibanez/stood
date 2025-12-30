//! Callback system for Agent execution events.
//!
//! This module provides a comprehensive callback system that allows users to
//! receive real-time updates during agent execution, including content streaming,
//! tool usage, and performance metrics.

pub mod config;
pub mod error;
pub mod events;
pub mod handlers;
pub mod traits;

#[cfg(feature = "benchmarks")]
pub mod benchmarks;

pub mod batching;

pub use batching::{BatchConfig, BatchingCallbackHandler, EventBatch};
pub use config::{CallbackHandlerConfig, PrintingConfig};
pub use error::CallbackError;
pub use events::{CallbackEvent, TokenUsage, ToolEvent};
pub use handlers::{
    CompositeCallbackHandler, NullCallbackHandler, PerformanceCallbackHandler,
    PrintingCallbackHandler,
};
pub use traits::{CallbackHandler, SyncCallbackHandler};
