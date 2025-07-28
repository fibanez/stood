//! Core callback traits for the Agent execution system.
//!
//! This module provides the main [`CallbackHandler`] trait and related types
//! that enable real-time monitoring and interaction during agent execution,
//! including support for parallel tool execution monitoring and streaming responses.
//!
//! # Features
//!
//! - **Real-time tool monitoring**: Track tool execution start, completion, and failure
//! - **Streaming content handling**: Process content as it's generated
//! - **Parallel execution events**: Monitor parallel tool execution progress
//! - **Error handling**: Receive and handle execution errors
//! - **Performance metrics**: Access execution duration and completion statistics
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use stood::agent::callbacks::{CallbackHandler, ToolEvent, CallbackError};
//! use async_trait::async_trait;
//!
//! struct MyCallback;
//!
//! #[async_trait]
//! impl CallbackHandler for MyCallback {
//!     async fn on_tool(&self, event: ToolEvent) -> Result<(), CallbackError> {
//!         match event {
//!             ToolEvent::Started { name, .. } => {
//!                 println!("🚀 Starting tool: {}", name);
//!             }
//!             ToolEvent::Completed { name, duration, .. } => {
//!                 println!("✅ Completed tool '{}' in {:?}", name, duration);
//!             }
//!             ToolEvent::Failed { name, error, .. } => {
//!                 println!("❌ Tool '{}' failed: {}", name, error);
//!             }
//!         }
//!         Ok(())
//!     }
//!
//!     async fn on_parallel_start(&self, tool_count: usize, max_parallel: usize) -> Result<(), CallbackError> {
//!         println!("🔄 Starting parallel execution: {} tools, max {} concurrent", tool_count, max_parallel);
//!         Ok(())
//!     }
//! }
//! ```

use async_trait::async_trait;
use crate::agent::result::AgentResult;
use crate::error::StoodError;
use super::events::{CallbackEvent, ToolEvent};
use super::error::CallbackError;
use std::time::Duration;

/// Core callback handler trait - simplified from Python's flexible kwargs
///
/// This trait provides the main interface for handling events during agent execution.
/// All methods have default implementations that do nothing, allowing implementations
/// to only override the events they care about.
#[async_trait]
pub trait CallbackHandler: Send + Sync {
    /// Handle streaming text content as it's generated (matches Python's 'data' kwarg)
    ///
    /// This method is called as the agent generates content, enabling real-time
    /// streaming of responses to users.
    ///
    /// # Arguments
    /// * `content` - The text content being generated
    /// * `is_complete` - Whether this is the final piece of content
    async fn on_content(&self, content: &str, is_complete: bool) -> Result<(), CallbackError> {
        let _ = (content, is_complete);
        Ok(()) // Default no-op
    }
    
    /// Handle tool execution events (matches Python's 'current_tool_use' kwarg)
    ///
    /// This method is called for all tool-related events, including start,
    /// completion, and failure events.
    async fn on_tool(&self, event: ToolEvent) -> Result<(), CallbackError> {
        let _ = event;
        Ok(()) // Default no-op
    }
    
    /// Handle execution completion (matches Python's completion pattern)
    ///
    /// This method is called when the entire agent execution completes,
    /// providing access to the final results and metrics.
    async fn on_complete(&self, result: &AgentResult) -> Result<(), CallbackError> {
        let _ = result;
        Ok(()) // Default no-op
    }
    
    /// Handle errors (matches Python's error handling)
    ///
    /// This method is called when errors occur during execution, allowing
    /// for custom error logging or recovery actions.
    async fn on_error(&self, error: &StoodError) -> Result<(), CallbackError> {
        let _ = error;
        Ok(()) // Default no-op
    }
    
    /// Handle parallel execution start
    ///
    /// This method is called when parallel tool execution begins.
    async fn on_parallel_start(&self, tool_count: usize, max_parallel: usize) -> Result<(), CallbackError> {
        let _ = (tool_count, max_parallel);
        Ok(()) // Default no-op
    }
    
    /// Handle parallel execution progress
    ///
    /// This method is called during parallel tool execution to report progress.
    async fn on_parallel_progress(&self, completed: usize, total: usize, running: usize) -> Result<(), CallbackError> {
        let _ = (completed, total, running);
        Ok(()) // Default no-op
    }
    
    /// Handle parallel execution completion
    ///
    /// This method is called when parallel tool execution completes.
    async fn on_parallel_complete(&self, total_duration: Duration, success_count: usize, failure_count: usize) -> Result<(), CallbackError> {
        let _ = (total_duration, success_count, failure_count);
        Ok(()) // Default no-op
    }
    
    /// Handle evaluation events
    ///
    /// This method is called when the agent evaluates whether to continue with more cycles.
    async fn on_evaluation(&self, strategy: &str, decision: bool, reasoning: &str, duration: Duration) -> Result<(), CallbackError> {
        let _ = (strategy, decision, reasoning, duration);
        Ok(()) // Default no-op
    }
    
    /// Full event handler for advanced usage (matches Python's flexibility)
    ///
    /// This method receives all events and can be used for comprehensive
    /// monitoring or when you need access to event details not available
    /// in the simplified handlers above.
    async fn handle_event(&self, event: CallbackEvent) -> Result<(), CallbackError> {
        match event {
            CallbackEvent::ContentDelta { delta, complete, .. } => {
                self.on_content(&delta, complete).await
            }
            CallbackEvent::ToolStart { tool_name, input, .. } => {
                self.on_tool(ToolEvent::Started { name: tool_name, input }).await
            }
            CallbackEvent::ToolComplete { tool_name, output, error, duration, .. } => {
                if let Some(err) = error {
                    self.on_tool(ToolEvent::Failed { name: tool_name, error: err, duration }).await
                } else {
                    self.on_tool(ToolEvent::Completed { name: tool_name, output, duration }).await
                }
            }
            CallbackEvent::EventLoopComplete { result, .. } => {
                // Convert EventLoopResult to AgentResult for callback
                let agent_result = AgentResult::from(result, Duration::ZERO);
                self.on_complete(&agent_result).await
            }
            CallbackEvent::Error { error, .. } => {
                self.on_error(&error).await
            }
            CallbackEvent::ParallelStart { tool_count, max_parallel } => {
                self.on_parallel_start(tool_count, max_parallel).await
            }
            CallbackEvent::ParallelProgress { completed, total, running } => {
                self.on_parallel_progress(completed, total, running).await
            }
            CallbackEvent::ParallelComplete { total_duration, success_count, failure_count } => {
                self.on_parallel_complete(total_duration, success_count, failure_count).await
            }
            CallbackEvent::EvaluationStart { .. } => {
                // Start events are handled in the implementation if needed
                Ok(())
            }
            CallbackEvent::EvaluationComplete { strategy, decision, reasoning, duration } => {
                self.on_evaluation(&strategy, decision, &reasoning, duration).await
            }
            _ => Ok(()), // Ignore other events by default
        }
    }
}

/// Sync callback handler for non-async scenarios
///
/// This trait allows for synchronous callback handlers that don't need
/// async capabilities. An automatic async wrapper is provided.
pub trait SyncCallbackHandler: Send + Sync {
    /// Synchronous content handler
    fn on_content_sync(&self, content: &str, is_complete: bool) -> Result<(), CallbackError> {
        let _ = (content, is_complete);
        Ok(())
    }
    
    /// Synchronous tool event handler
    fn on_tool_sync(&self, event: ToolEvent) -> Result<(), CallbackError> {
        let _ = event;
        Ok(())
    }
    
    /// Synchronous event handler (required)
    fn handle_event_sync(&self, event: CallbackEvent) -> Result<(), CallbackError>;
}

/// Automatic async wrapper for sync handlers
///
/// This implementation allows any `SyncCallbackHandler` to be used as a
/// `CallbackHandler` automatically.
#[async_trait]
impl<T: SyncCallbackHandler> CallbackHandler for T {
    async fn handle_event(&self, event: CallbackEvent) -> Result<(), CallbackError> {
        self.handle_event_sync(event)
    }
}