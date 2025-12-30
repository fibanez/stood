//! Tool middleware for intercepting and modifying tool execution.
//!
//! This module provides a middleware system for intercepting tool calls
//! before and after execution. Middleware can:
//!
//! - Modify tool parameters before execution
//! - Abort tool execution with synthetic results
//! - Skip tool execution entirely
//! - Modify tool results after execution
//! - Inject additional context after tool execution
//!
//! # Architecture
//!
//! ```text
//! Tool Request → before_tool() → Tool Execution → after_tool() → Result
//!                     ↓                                  ↓
//!              Can: Modify params           Can: Modify result
//!                   Abort/Skip                   Inject context
//! ```
//!
//! # Example
//!
//! ```no_run
//! use stood::tools::middleware::{ToolMiddleware, ToolMiddlewareAction, AfterToolAction, ToolContext};
//! use stood::tools::ToolResult;
//! use async_trait::async_trait;
//! use serde_json::Value;
//!
//! #[derive(Debug)]
//! struct LoggingMiddleware;
//!
//! #[async_trait]
//! impl ToolMiddleware for LoggingMiddleware {
//!     async fn before_tool(
//!         &self,
//!         tool_name: &str,
//!         params: &Value,
//!         ctx: &ToolContext,
//!     ) -> ToolMiddlewareAction {
//!         println!("Executing tool: {} with params: {:?}", tool_name, params);
//!         ToolMiddlewareAction::Continue
//!     }
//!
//!     async fn after_tool(
//!         &self,
//!         tool_name: &str,
//!         result: &ToolResult,
//!         ctx: &ToolContext,
//!     ) -> AfterToolAction {
//!         println!("Tool {} completed with success: {}", tool_name, result.success);
//!         AfterToolAction::PassThrough
//!     }
//! }
//! ```

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;

use super::ToolResult;

/// Context provided to middleware during tool execution.
///
/// Contains information about the current execution environment
/// that middleware can use to make decisions.
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// Unique identifier for the agent making the tool call
    pub agent_id: String,
    /// Name of the agent (if set)
    pub agent_name: Option<String>,
    /// Type of agent (e.g., "orchestrator", "worker")
    pub agent_type: String,
    /// When this tool execution started
    pub execution_start: Instant,
    /// Number of tools executed in this agent turn
    pub tool_count_this_turn: usize,
    /// Total conversation message count
    pub message_count: usize,
}

impl ToolContext {
    /// Create a new ToolContext
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            agent_name: None,
            agent_type: "unknown".to_string(),
            execution_start: Instant::now(),
            tool_count_this_turn: 0,
            message_count: 0,
        }
    }

    /// Create context from an AgentContext
    pub fn from_agent_context(ctx: &crate::agent::AgentContext) -> Self {
        Self {
            agent_id: ctx.agent_id.clone(),
            agent_name: ctx.agent_name.clone(),
            agent_type: ctx.agent_type.clone(),
            execution_start: Instant::now(),
            tool_count_this_turn: 0,
            message_count: 0,
        }
    }

    /// Get elapsed time since execution started
    pub fn elapsed_ms(&self) -> u128 {
        self.execution_start.elapsed().as_millis()
    }

    /// Builder pattern: set agent name
    pub fn with_agent_name(mut self, name: Option<String>) -> Self {
        self.agent_name = name;
        self
    }

    /// Builder pattern: set agent type
    pub fn with_agent_type(mut self, agent_type: String) -> Self {
        self.agent_type = agent_type;
        self
    }

    /// Builder pattern: set tool count
    pub fn with_tool_count(mut self, count: usize) -> Self {
        self.tool_count_this_turn = count;
        self
    }

    /// Builder pattern: set message count
    pub fn with_message_count(mut self, count: usize) -> Self {
        self.message_count = count;
        self
    }
}

/// Action to take before tool execution.
///
/// Returned by `ToolMiddleware::before_tool()` to control
/// how the tool execution proceeds.
#[derive(Debug, Clone)]
pub enum ToolMiddlewareAction {
    /// Continue with the original parameters
    Continue,

    /// Continue with modified parameters
    ModifyParams(Value),

    /// Abort this tool call with a synthetic result
    Abort {
        /// Reason for aborting (logged but not sent to model)
        reason: String,
        /// Optional synthetic result to use instead of executing
        synthetic_result: Option<ToolResult>,
    },

    /// Skip this tool entirely (no result added to conversation)
    Skip,
}

/// Action to take after tool execution.
///
/// Returned by `ToolMiddleware::after_tool()` to control
/// how the tool result is processed.
#[derive(Debug, Clone)]
pub enum AfterToolAction {
    /// Pass the result through unchanged
    PassThrough,

    /// Modify the result before adding to conversation
    ModifyResult(ToolResult),

    /// Inject additional context after the result
    /// This context is added as a system message
    InjectContext(String),
}

/// Trait for implementing tool execution middleware.
///
/// Tool middleware intercepts tool execution at two points:
/// - Before execution: Can modify parameters, abort, or skip
/// - After execution: Can modify results or inject context
///
/// # Thread Safety
///
/// Middleware must be `Send + Sync` for use across async boundaries.
///
/// # Example
///
/// ```no_run
/// use stood::tools::middleware::{ToolMiddleware, ToolMiddlewareAction, AfterToolAction, ToolContext};
/// use stood::tools::ToolResult;
/// use async_trait::async_trait;
/// use serde_json::Value;
/// use std::collections::HashMap;
/// use std::sync::Mutex;
///
/// /// Middleware that caches tool results
/// #[derive(Debug)]
/// struct CachingMiddleware {
///     cache: Mutex<HashMap<String, ToolResult>>,
/// }
///
/// #[async_trait]
/// impl ToolMiddleware for CachingMiddleware {
///     async fn before_tool(
///         &self,
///         tool_name: &str,
///         params: &Value,
///         _ctx: &ToolContext,
///     ) -> ToolMiddlewareAction {
///         let cache_key = format!("{}:{}", tool_name, params);
///
///         if let Some(cached) = self.cache.lock().unwrap().get(&cache_key) {
///             return ToolMiddlewareAction::Abort {
///                 reason: "Cache hit".to_string(),
///                 synthetic_result: Some(cached.clone()),
///             };
///         }
///
///         ToolMiddlewareAction::Continue
///     }
///
///     async fn after_tool(
///         &self,
///         tool_name: &str,
///         result: &ToolResult,
///         _ctx: &ToolContext,
///     ) -> AfterToolAction {
///         // Could cache the result here
///         AfterToolAction::PassThrough
///     }
/// }
/// ```
#[async_trait]
pub trait ToolMiddleware: Send + Sync + std::fmt::Debug {
    /// Called before tool execution.
    ///
    /// Return `ToolMiddlewareAction` to control execution:
    /// - `Continue`: Execute with original parameters
    /// - `ModifyParams(Value)`: Execute with modified parameters
    /// - `Abort { reason, synthetic_result }`: Don't execute, use synthetic result
    /// - `Skip`: Don't execute, don't add any result
    async fn before_tool(
        &self,
        tool_name: &str,
        params: &Value,
        ctx: &ToolContext,
    ) -> ToolMiddlewareAction;

    /// Called after tool execution completes.
    ///
    /// Return `AfterToolAction` to control result processing:
    /// - `PassThrough`: Use result as-is
    /// - `ModifyResult(ToolResult)`: Replace the result
    /// - `InjectContext(String)`: Add context message after result
    async fn after_tool(
        &self,
        tool_name: &str,
        result: &ToolResult,
        ctx: &ToolContext,
    ) -> AfterToolAction;

    /// Name of this middleware for logging/debugging
    fn name(&self) -> &str {
        "unnamed_middleware"
    }
}

/// Stack of middleware that processes tool calls in order.
///
/// Middleware is executed in registration order for `before_tool`
/// and reverse order for `after_tool`.
#[derive(Debug, Default)]
pub struct MiddlewareStack {
    layers: Vec<Arc<dyn ToolMiddleware>>,
}

impl MiddlewareStack {
    /// Create a new empty middleware stack
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Add middleware to the stack
    pub fn add(&mut self, middleware: Arc<dyn ToolMiddleware>) {
        tracing::debug!("Adding middleware: {}", middleware.name());
        self.layers.push(middleware);
    }

    /// Check if the stack has any middleware
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Get number of middleware layers
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Process before_tool through all middleware layers.
    ///
    /// Returns the final action and potentially modified parameters.
    /// Short-circuits on Abort or Skip.
    pub async fn process_before_tool(
        &self,
        tool_name: &str,
        params: &Value,
        ctx: &ToolContext,
    ) -> (ToolMiddlewareAction, Value) {
        let mut current_params = params.clone();

        for middleware in &self.layers {
            let action = middleware.before_tool(tool_name, &current_params, ctx).await;

            match action {
                ToolMiddlewareAction::Continue => {
                    // Continue to next middleware with current params
                }
                ToolMiddlewareAction::ModifyParams(new_params) => {
                    tracing::debug!(
                        "Middleware {} modified params for tool {}",
                        middleware.name(),
                        tool_name
                    );
                    current_params = new_params;
                }
                ToolMiddlewareAction::Abort { ref reason, .. } => {
                    tracing::info!(
                        "Middleware {} aborted tool {}: {}",
                        middleware.name(),
                        tool_name,
                        reason
                    );
                    return (action, current_params);
                }
                ToolMiddlewareAction::Skip => {
                    tracing::info!(
                        "Middleware {} skipped tool {}",
                        middleware.name(),
                        tool_name
                    );
                    return (action, current_params);
                }
            }
        }

        (ToolMiddlewareAction::Continue, current_params)
    }

    /// Process after_tool through all middleware layers (reverse order).
    ///
    /// Returns the final action and potentially modified result.
    pub async fn process_after_tool(
        &self,
        tool_name: &str,
        result: &ToolResult,
        ctx: &ToolContext,
    ) -> (AfterToolAction, ToolResult) {
        let mut current_result = result.clone();
        let mut final_action = AfterToolAction::PassThrough;

        // Process in reverse order
        for middleware in self.layers.iter().rev() {
            let action = middleware.after_tool(tool_name, &current_result, ctx).await;

            match action {
                AfterToolAction::PassThrough => {
                    // Continue with current result
                }
                AfterToolAction::ModifyResult(new_result) => {
                    tracing::debug!(
                        "Middleware {} modified result for tool {}",
                        middleware.name(),
                        tool_name
                    );
                    current_result = new_result;
                }
                AfterToolAction::InjectContext(ref context) => {
                    tracing::debug!(
                        "Middleware {} injecting context after tool {}: {}",
                        middleware.name(),
                        tool_name,
                        context
                    );
                    // Keep the inject action - it will be handled by the caller
                    final_action = action;
                }
            }
        }

        (final_action, current_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[derive(Debug)]
    struct PassThroughMiddleware;

    #[async_trait]
    impl ToolMiddleware for PassThroughMiddleware {
        async fn before_tool(
            &self,
            _tool_name: &str,
            _params: &Value,
            _ctx: &ToolContext,
        ) -> ToolMiddlewareAction {
            ToolMiddlewareAction::Continue
        }

        async fn after_tool(
            &self,
            _tool_name: &str,
            _result: &ToolResult,
            _ctx: &ToolContext,
        ) -> AfterToolAction {
            AfterToolAction::PassThrough
        }

        fn name(&self) -> &str {
            "pass_through"
        }
    }

    #[derive(Debug)]
    struct ModifyingMiddleware {
        add_field: String,
    }

    #[async_trait]
    impl ToolMiddleware for ModifyingMiddleware {
        async fn before_tool(
            &self,
            _tool_name: &str,
            params: &Value,
            _ctx: &ToolContext,
        ) -> ToolMiddlewareAction {
            let mut new_params = params.clone();
            if let Some(obj) = new_params.as_object_mut() {
                obj.insert(
                    "middleware_added".to_string(),
                    Value::String(self.add_field.clone()),
                );
            }
            ToolMiddlewareAction::ModifyParams(new_params)
        }

        async fn after_tool(
            &self,
            _tool_name: &str,
            result: &ToolResult,
            _ctx: &ToolContext,
        ) -> AfterToolAction {
            let mut new_result = result.clone();
            if let Some(obj) = new_result.content.as_object_mut() {
                obj.insert(
                    "middleware_processed".to_string(),
                    Value::Bool(true),
                );
            }
            AfterToolAction::ModifyResult(new_result)
        }

        fn name(&self) -> &str {
            "modifying"
        }
    }

    #[derive(Debug)]
    struct AbortingMiddleware {
        abort_tool: String,
    }

    #[async_trait]
    impl ToolMiddleware for AbortingMiddleware {
        async fn before_tool(
            &self,
            tool_name: &str,
            _params: &Value,
            _ctx: &ToolContext,
        ) -> ToolMiddlewareAction {
            if tool_name == self.abort_tool {
                ToolMiddlewareAction::Abort {
                    reason: "Tool blocked by policy".to_string(),
                    synthetic_result: Some(ToolResult::error("Tool execution blocked")),
                }
            } else {
                ToolMiddlewareAction::Continue
            }
        }

        async fn after_tool(
            &self,
            _tool_name: &str,
            _result: &ToolResult,
            _ctx: &ToolContext,
        ) -> AfterToolAction {
            AfterToolAction::PassThrough
        }

        fn name(&self) -> &str {
            "aborting"
        }
    }

    #[tokio::test]
    async fn test_middleware_stack_empty() {
        let stack = MiddlewareStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[tokio::test]
    async fn test_middleware_stack_add() {
        let mut stack = MiddlewareStack::new();
        stack.add(Arc::new(PassThroughMiddleware));
        assert!(!stack.is_empty());
        assert_eq!(stack.len(), 1);
    }

    #[tokio::test]
    async fn test_pass_through_middleware() {
        let mut stack = MiddlewareStack::new();
        stack.add(Arc::new(PassThroughMiddleware));

        let ctx = ToolContext::new("test-agent".to_string());
        let params = json!({"key": "value"});

        let (action, result_params) = stack.process_before_tool("test_tool", &params, &ctx).await;

        assert!(matches!(action, ToolMiddlewareAction::Continue));
        assert_eq!(result_params, params);
    }

    #[tokio::test]
    async fn test_modifying_middleware() {
        let mut stack = MiddlewareStack::new();
        stack.add(Arc::new(ModifyingMiddleware {
            add_field: "test_value".to_string(),
        }));

        let ctx = ToolContext::new("test-agent".to_string());
        let params = json!({"key": "value"});

        let (action, result_params) = stack.process_before_tool("test_tool", &params, &ctx).await;

        assert!(matches!(action, ToolMiddlewareAction::Continue));
        assert_eq!(result_params["key"], "value");
        assert_eq!(result_params["middleware_added"], "test_value");
    }

    #[tokio::test]
    async fn test_aborting_middleware() {
        let mut stack = MiddlewareStack::new();
        stack.add(Arc::new(AbortingMiddleware {
            abort_tool: "blocked_tool".to_string(),
        }));

        let ctx = ToolContext::new("test-agent".to_string());
        let params = json!({"key": "value"});

        // Test with blocked tool
        let (action, _) = stack
            .process_before_tool("blocked_tool", &params, &ctx)
            .await;
        assert!(matches!(action, ToolMiddlewareAction::Abort { .. }));

        // Test with allowed tool
        let (action, _) = stack
            .process_before_tool("allowed_tool", &params, &ctx)
            .await;
        assert!(matches!(action, ToolMiddlewareAction::Continue));
    }

    #[tokio::test]
    async fn test_middleware_chain() {
        let mut stack = MiddlewareStack::new();
        stack.add(Arc::new(ModifyingMiddleware {
            add_field: "first".to_string(),
        }));
        stack.add(Arc::new(ModifyingMiddleware {
            add_field: "second".to_string(),
        }));

        let ctx = ToolContext::new("test-agent".to_string());
        let params = json!({"key": "value"});

        let (action, result_params) = stack.process_before_tool("test_tool", &params, &ctx).await;

        assert!(matches!(action, ToolMiddlewareAction::Continue));
        // Second middleware overwrites the field
        assert_eq!(result_params["middleware_added"], "second");
    }

    #[tokio::test]
    async fn test_after_tool_processing() {
        let mut stack = MiddlewareStack::new();
        stack.add(Arc::new(ModifyingMiddleware {
            add_field: "test".to_string(),
        }));

        let ctx = ToolContext::new("test-agent".to_string());
        let result = ToolResult::success(json!({"data": "value"}));

        let (action, modified_result) = stack.process_after_tool("test_tool", &result, &ctx).await;

        assert!(matches!(action, AfterToolAction::PassThrough));
        assert!(modified_result.success);
        assert_eq!(modified_result.content["middleware_processed"], true);
    }

    #[tokio::test]
    async fn test_tool_context_elapsed() {
        let ctx = ToolContext::new("test-agent".to_string());

        // Sleep a tiny bit to ensure elapsed time > 0
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

        assert!(ctx.elapsed_ms() >= 1);
    }

    #[tokio::test]
    async fn test_tool_context_builder() {
        let ctx = ToolContext::new("test-agent".to_string())
            .with_agent_name(Some("TestAgent".to_string()))
            .with_agent_type("orchestrator".to_string())
            .with_tool_count(5)
            .with_message_count(10);

        assert_eq!(ctx.agent_id, "test-agent");
        assert_eq!(ctx.agent_name, Some("TestAgent".to_string()));
        assert_eq!(ctx.agent_type, "orchestrator");
        assert_eq!(ctx.tool_count_this_turn, 5);
        assert_eq!(ctx.message_count, 10);
    }
}
