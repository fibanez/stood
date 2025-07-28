//! Built-in callback handler implementations.
//!
//! This module provides several pre-built callback handlers that cover
//! common use cases like printing output, performance logging, and
//! composing multiple handlers.

use super::config::PrintingConfig;
use super::error::CallbackError;
use super::events::{CallbackEvent, ToolEvent};
use super::traits::CallbackHandler;
use crate::agent::result::{AgentResult};
#[allow(unused_imports)] // Used in future callback features
use crate::agent::result::ToolCallSummary;
use crate::error::StoodError;
use async_trait::async_trait;
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;

/// No-op callback handler (equivalent to Python's null_callback_handler)
///
/// This handler does nothing and is used as the default when no callbacks
/// are configured. It implements all trait methods as no-ops.
#[derive(Debug, Default)]
pub struct NullCallbackHandler;

#[async_trait]
impl CallbackHandler for NullCallbackHandler {
    // All methods use default implementations (no-op)
}

/// Enhanced printing handler with configurable output (equivalent to Python's PrintingCallbackHandler)
///
/// This handler provides console output for various execution events,
/// with configurable verbosity and formatting options.
#[derive(Debug)]
pub struct PrintingCallbackHandler {
    config: PrintingConfig,
}

impl PrintingCallbackHandler {
    /// Create a new printing handler with the given configuration
    pub fn new(config: PrintingConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl CallbackHandler for PrintingCallbackHandler {
    async fn on_content(&self, content: &str, is_complete: bool) -> Result<(), CallbackError> {
        if self.config.stream_output {
            if is_complete {
                println!("{}", content);
            } else {
                print!("{}", content);
                io::stdout().flush().unwrap();
            }
        }
        Ok(())
    }

    async fn on_tool(&self, event: ToolEvent) -> Result<(), CallbackError> {
        if !self.config.show_tools {
            return Ok(());
        }

        match event {
            ToolEvent::Started { name, .. } => {
                println!("\n🔧 Executing tool: {}", name);
            }
            ToolEvent::Completed { name, duration, .. } => {
                println!("✅ Tool {} completed in {:?}", name, duration);
            }
            ToolEvent::Failed {
                name,
                error,
                duration,
            } => {
                println!("❌ Tool {} failed after {:?}: {}", name, duration, error);
            }
        }
        Ok(())
    }

    async fn on_complete(&self, result: &AgentResult) -> Result<(), CallbackError> {
        if self.config.show_performance {
            println!("\\n📊 Execution Summary:");
            println!("   Duration: {:?}", result.duration);
            println!("   Cycles: {}", result.execution.cycles);
            println!("   Tools used: {}", result.tools_called.len());
            if !result.tools_called.is_empty() {
                println!("   Tools: {}", result.tools_called.join(", "));
            }
        }
        Ok(())
    }

    async fn on_evaluation(
        &self,
        strategy: &str,
        decision: bool,
        reasoning: &str,
        duration: Duration,
    ) -> Result<(), CallbackError> {
        let decision_emoji = if decision { "🔄" } else { "🛑" };
        let decision_text = if decision { "CONTINUE" } else { "STOP" };

        println!();
        println!(
            "🤔 Evaluation ({}) {} -> {} (took {:?})",
            strategy, decision_emoji, decision_text, duration
        );

        if self.config.show_reasoning && !reasoning.is_empty() {
            // Try to parse as JSON and extract meaningful content
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(reasoning) {
                if let Some(response) = json.get("response").and_then(|r| r.as_str()) {
                    if !response.trim().is_empty() {
                        println!("   Additional content: {}", response.trim());
                    }
                }
            } else {
                // Fallback to showing raw reasoning
                println!("   Reasoning: {}", reasoning.trim());
            }
        }

        Ok(())
    }

    /// Handle reasoning text (matches Python's reasoningText kwarg)
    async fn handle_event(&self, event: CallbackEvent) -> Result<(), CallbackError> {
        match event {
            CallbackEvent::ContentDelta {
                delta,
                complete,
                reasoning,
            } => {
                if reasoning && !self.config.show_reasoning {
                    return Ok(()); // Skip reasoning if not configured to show it
                }
                if reasoning && self.config.show_reasoning {
                    print!("💭 {}", delta); // Prefix reasoning with thinking emoji
                } else {
                    print!("{}", delta);
                }
                if complete {
                    println!();
                } else {
                    io::stdout().flush().unwrap();
                }
            }
            _ => {
                // Delegate to base trait implementation for other events
                match event {
                    CallbackEvent::ToolStart {
                        tool_name, input, ..
                    } => {
                        self.on_tool(ToolEvent::Started {
                            name: tool_name,
                            input,
                        })
                        .await?;
                    }
                    CallbackEvent::ToolComplete {
                        tool_name,
                        output,
                        error,
                        duration,
                        ..
                    } => {
                        if let Some(err) = error {
                            self.on_tool(ToolEvent::Failed {
                                name: tool_name,
                                error: err,
                                duration,
                            })
                            .await?;
                        } else {
                            self.on_tool(ToolEvent::Completed {
                                name: tool_name,
                                output,
                                duration,
                            })
                            .await?;
                        }
                    }
                    CallbackEvent::EventLoopComplete { result, .. } => {
                        // Convert EventLoopResult to AgentResult for callback
                        let agent_result = AgentResult::from(result, std::time::Duration::ZERO);
                        self.on_complete(&agent_result).await?;
                    }
                    CallbackEvent::Error { error, .. } => {
                        self.on_error(&error).await?;
                    }
                    CallbackEvent::ParallelStart {
                        tool_count,
                        max_parallel,
                    } => {
                        self.on_parallel_start(tool_count, max_parallel).await?;
                    }
                    CallbackEvent::ParallelProgress {
                        completed,
                        total,
                        running,
                    } => {
                        self.on_parallel_progress(completed, total, running).await?;
                    }
                    CallbackEvent::ParallelComplete {
                        total_duration,
                        success_count,
                        failure_count,
                    } => {
                        self.on_parallel_complete(total_duration, success_count, failure_count)
                            .await?;
                    }
                    CallbackEvent::EvaluationStart { strategy, .. } => {
                        // Optionally handle evaluation start events
                        if self.config.show_reasoning {
                            println!(" ");
                            println!("🤔 Starting evaluation using {} strategy...", strategy);
                        }
                    }
                    CallbackEvent::EvaluationComplete {
                        strategy,
                        decision,
                        reasoning,
                        duration,
                    } => {
                        self.on_evaluation(&strategy, decision, &reasoning, duration)
                            .await?;
                    }
                    _ => {} // Ignore other events
                }
            }
        }
        Ok(())
    }
}

/// Composite handler (equivalent to Python's CompositeCallbackHandler)
///
/// This handler allows combining multiple callback handlers so that
/// events are sent to all of them.
pub struct CompositeCallbackHandler {
    handlers: Vec<Arc<dyn CallbackHandler>>,
}

impl Default for CompositeCallbackHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositeCallbackHandler {
    /// Create a new empty composite handler
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Add a handler to the composite
    pub fn add_handler(mut self, handler: Arc<dyn CallbackHandler>) -> Self {
        self.handlers.push(handler);
        self
    }

    /// Create a composite handler with the given handlers
    pub fn with_handlers(handlers: Vec<Arc<dyn CallbackHandler>>) -> Self {
        Self { handlers }
    }
}

#[async_trait]
impl CallbackHandler for CompositeCallbackHandler {
    async fn on_content(&self, content: &str, is_complete: bool) -> Result<(), CallbackError> {
        for handler in &self.handlers {
            handler.on_content(content, is_complete).await?;
        }
        Ok(())
    }

    async fn on_tool(&self, event: ToolEvent) -> Result<(), CallbackError> {
        for handler in &self.handlers {
            handler.on_tool(event.clone()).await?;
        }
        Ok(())
    }

    async fn on_complete(&self, result: &AgentResult) -> Result<(), CallbackError> {
        for handler in &self.handlers {
            handler.on_complete(result).await?;
        }
        Ok(())
    }

    async fn on_error(&self, error: &StoodError) -> Result<(), CallbackError> {
        for handler in &self.handlers {
            handler.on_error(error).await?;
        }
        Ok(())
    }

    async fn handle_event(&self, event: CallbackEvent) -> Result<(), CallbackError> {
        for handler in &self.handlers {
            handler.handle_event(event.clone()).await?;
        }
        Ok(())
    }
}

/// Performance logging callback handler
///
/// This handler logs performance metrics using the tracing system,
/// useful for production monitoring and debugging.
#[derive(Debug)]
pub struct PerformanceCallbackHandler {
    log_level: tracing::Level,
}

impl PerformanceCallbackHandler {
    /// Create a new performance logging handler
    pub fn new(log_level: tracing::Level) -> Self {
        Self { log_level }
    }
}

#[async_trait]
impl CallbackHandler for PerformanceCallbackHandler {
    async fn on_tool(&self, event: ToolEvent) -> Result<(), CallbackError> {
        match event {
            ToolEvent::Completed { name, duration, .. } => match self.log_level {
                tracing::Level::ERROR => {
                    tracing::error!(tool = %name, duration = ?duration, "Tool execution completed")
                }
                tracing::Level::WARN => {
                    tracing::warn!(tool = %name, duration = ?duration, "Tool execution completed")
                }
                tracing::Level::INFO => {
                    tracing::info!(tool = %name, duration = ?duration, "Tool execution completed")
                }
                tracing::Level::DEBUG => {
                    tracing::debug!(tool = %name, duration = ?duration, "Tool execution completed")
                }
                tracing::Level::TRACE => {
                    tracing::trace!(tool = %name, duration = ?duration, "Tool execution completed")
                }
            },
            ToolEvent::Failed {
                name,
                duration,
                error,
            } => match self.log_level {
                tracing::Level::ERROR => {
                    tracing::error!(tool = %name, duration = ?duration, error = %error, "Tool execution failed")
                }
                tracing::Level::WARN => {
                    tracing::warn!(tool = %name, duration = ?duration, error = %error, "Tool execution failed")
                }
                tracing::Level::INFO => {
                    tracing::info!(tool = %name, duration = ?duration, error = %error, "Tool execution failed")
                }
                tracing::Level::DEBUG => {
                    tracing::debug!(tool = %name, duration = ?duration, error = %error, "Tool execution failed")
                }
                tracing::Level::TRACE => {
                    tracing::trace!(tool = %name, duration = ?duration, error = %error, "Tool execution failed")
                }
            },
            _ => {}
        }
        Ok(())
    }

    async fn on_complete(&self, result: &AgentResult) -> Result<(), CallbackError> {
        match self.log_level {
            tracing::Level::ERROR => {
                tracing::error!(duration = ?result.duration, cycles = result.execution.cycles, tools = result.tools_called.len(), "Agent execution completed")
            }
            tracing::Level::WARN => {
                tracing::warn!(duration = ?result.duration, cycles = result.execution.cycles, tools = result.tools_called.len(), "Agent execution completed")
            }
            tracing::Level::INFO => {
                tracing::info!(duration = ?result.duration, cycles = result.execution.cycles, tools = result.tools_called.len(), "Agent execution completed")
            }
            tracing::Level::DEBUG => {
                tracing::debug!(duration = ?result.duration, cycles = result.execution.cycles, tools = result.tools_called.len(), "Agent execution completed")
            }
            tracing::Level::TRACE => {
                tracing::trace!(duration = ?result.duration, cycles = result.execution.cycles, tools = result.tools_called.len(), "Agent execution completed")
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::callbacks::events::{CallbackEvent, ToolEvent};
    use crate::agent::result::{AgentResult, ExecutionDetails, PerformanceMetrics, TokenUsage};
    use serde_json::json;
    use std::time::Duration;

    /// Helper to create a mock AgentResult for testing
    fn create_mock_agent_result() -> AgentResult {
        AgentResult {
            response: "Test response".to_string(),
            execution: ExecutionDetails {
                cycles: 1,
                model_calls: 1,
                tool_executions: 1,
                tokens: Some(TokenUsage {
                    input_tokens: 10,
                    output_tokens: 20,
                    total_tokens: 30,
                }),
                performance: PerformanceMetrics {
                    avg_cycle_time: Duration::from_millis(1000),
                    model_time: Duration::from_millis(800),
                    tool_time: Duration::from_millis(200),
                    was_streamed: true,
                },
            },
            tools_called: vec!["calculator".to_string()],
            tools_successful: vec!["calculator".to_string()],
            tools_failed: vec![],
            tool_call_summary: ToolCallSummary {
                total_attempts: 1,
                successful: 1,
                failed: 0,
                failed_calls: vec![],
            },
            duration: Duration::from_millis(1000),
            used_tools: true,
            success: true,
            error: None,
        }
    }

    #[tokio::test]
    async fn test_null_callback_handler() {
        let handler = NullCallbackHandler;

        // All methods should be no-ops and return Ok(())
        assert!(handler.on_content("test", true).await.is_ok());
        assert!(handler
            .on_tool(ToolEvent::Started {
                name: "test".to_string(),
                input: json!({})
            })
            .await
            .is_ok());
        assert!(handler
            .on_complete(&create_mock_agent_result())
            .await
            .is_ok());
        assert!(handler
            .on_error(&crate::StoodError::invalid_input("test"))
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_printing_callback_handler() {
        let config = PrintingConfig {
            show_reasoning: true,
            show_tools: true,
            show_performance: true,
            stream_output: true,
        };
        let handler = PrintingCallbackHandler::new(config);

        // Test content handling
        assert!(handler.on_content("Hello", false).await.is_ok());
        assert!(handler.on_content(" World", true).await.is_ok());

        // Test tool events
        assert!(handler
            .on_tool(ToolEvent::Started {
                name: "calculator".to_string(),
                input: json!({"operation": "add", "a": 2, "b": 3})
            })
            .await
            .is_ok());

        assert!(handler
            .on_tool(ToolEvent::Completed {
                name: "calculator".to_string(),
                output: Some(json!({"result": 5})),
                duration: Duration::from_millis(100)
            })
            .await
            .is_ok());

        assert!(handler
            .on_tool(ToolEvent::Failed {
                name: "calculator".to_string(),
                error: "Division by zero".to_string(),
                duration: Duration::from_millis(50)
            })
            .await
            .is_ok());

        // Test completion
        assert!(handler
            .on_complete(&create_mock_agent_result())
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_printing_callback_content_delta_events() {
        let config = PrintingConfig::default();
        let handler = PrintingCallbackHandler::new(config);

        // Test regular content delta
        let content_event = CallbackEvent::ContentDelta {
            delta: "Hello".to_string(),
            complete: false,
            reasoning: false,
        };
        assert!(handler.handle_event(content_event).await.is_ok());

        // Test reasoning content delta
        let reasoning_event = CallbackEvent::ContentDelta {
            delta: "Let me think...".to_string(),
            complete: true,
            reasoning: true,
        };
        assert!(handler.handle_event(reasoning_event).await.is_ok());
    }

    #[tokio::test]
    async fn test_composite_callback_handler() {
        let handler1 = Arc::new(NullCallbackHandler);
        let handler2 = Arc::new(PrintingCallbackHandler::new(PrintingConfig::default()));

        let composite = CompositeCallbackHandler::with_handlers(vec![handler1, handler2]);

        // Test that events are sent to all handlers
        assert!(composite.on_content("test", true).await.is_ok());
        assert!(composite
            .on_complete(&create_mock_agent_result())
            .await
            .is_ok());

        // Test tool events
        assert!(composite
            .on_tool(ToolEvent::Started {
                name: "test".to_string(),
                input: json!({})
            })
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_composite_callback_handler_builder() {
        let mut composite = CompositeCallbackHandler::new();
        composite = composite.add_handler(Arc::new(NullCallbackHandler));
        composite = composite.add_handler(Arc::new(PerformanceCallbackHandler::new(
            tracing::Level::DEBUG,
        )));

        // Test that the composite works with multiple handlers
        assert!(composite.on_content("test", false).await.is_ok());
        assert!(composite
            .on_complete(&create_mock_agent_result())
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_callback_error_handling() {
        let handler = PrintingCallbackHandler::new(PrintingConfig::default());

        // Test with an error event
        let error = crate::StoodError::invalid_input("Test error message");
        assert!(handler.on_error(&error).await.is_ok());

        // Test handle_event with error event
        let error_event = CallbackEvent::Error {
            error,
            context: "Test context".to_string(),
        };
        assert!(handler.handle_event(error_event).await.is_ok());
    }
}

