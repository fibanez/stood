//! Unified result types for the Agent execution interface.
//!
//! This module provides the [`AgentResult`] type that contains all information
//! from an agent execution, including the response text, execution metrics,
//! tool usage, and performance data.

use crate::agent::event_loop::EventLoopResult;
use crate::telemetry::EventLoopMetrics;
use std::time::Duration;

/// Unified result type that contains all information from execution
///
/// This is the single return type for the [`Agent::execute`] method, providing
/// both simple string access (via Display trait) and detailed execution analysis.
///
/// # Examples
///
/// Simple usage (Python-like string conversion):
/// ```no_run
/// # use stood::agent::{Agent, AgentResult};
/// # async fn example(mut agent: Agent) -> Result<(), Box<dyn std::error::Error>> {
/// let result = agent.execute("Tell me a joke").await?;
/// println!("{}", result); // Prints just the response text
/// # Ok(())
/// # }
/// ```
///
/// Detailed analysis:
/// ```no_run
/// # use stood::agent::{Agent, AgentResult};
/// # async fn example(mut agent: Agent) -> Result<(), Box<dyn std::error::Error>> {
/// let result = agent.execute("Complex analysis task").await?;
///
/// println!("Response: {}", result.response);
/// println!("Used {} tools in {} cycles", result.tools_called.len(), result.execution.cycles);
/// println!("Execution took {:?}", result.duration);
///
/// if result.used_tools {
///     println!("Tools used: {}", result.tools_called.join(", "));
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct AgentResult {
    /// Final response text (matches Python agent response)
    pub response: String,

    /// Execution metrics and details
    pub execution: ExecutionDetails,

    /// Whether tools were used during execution
    pub used_tools: bool,

    /// List of tools that were called (includes both successful and failed)
    pub tools_called: Vec<String>,

    /// List of tools that completed successfully
    pub tools_successful: Vec<String>,

    /// List of tools that failed
    pub tools_failed: Vec<String>,

    /// Detailed tool call summary
    pub tool_call_summary: ToolCallSummary,

    /// Total execution time
    pub duration: Duration,

    /// Whether execution completed successfully
    pub success: bool,

    /// Error message if execution failed
    pub error: Option<String>,
}

/// Detailed execution metrics and information
///
/// This struct provides comprehensive information about the agent's execution,
/// including performance metrics, token usage, and reasoning cycles.
#[derive(Debug, Clone)]
pub struct ExecutionDetails {
    /// Number of reasoning cycles executed
    pub cycles: u32,

    /// Number of model calls made
    pub model_calls: u32,

    /// Number of tool executions performed
    pub tool_executions: u32,

    /// Token usage information (if available)
    pub tokens: Option<TokenUsage>,

    /// Performance metrics
    pub performance: PerformanceMetrics,
}

/// Token usage information from model calls
#[derive(Debug, Clone)]
pub struct TokenUsage {
    /// Input tokens consumed
    pub input_tokens: u32,

    /// Output tokens generated
    pub output_tokens: u32,

    /// Total tokens used
    pub total_tokens: u32,
}

/// Performance metrics for execution analysis
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Average time per model interaction
    pub avg_cycle_time: Duration,

    /// Total time spent on model calls
    pub model_time: Duration,

    /// Total time spent on tool execution
    pub tool_time: Duration,

    /// Whether streaming was used
    pub was_streamed: bool,
}

/// Detailed breakdown of tool call attempts and results
#[derive(Debug, Clone)]
pub struct ToolCallSummary {
    /// Total number of tool calls attempted
    pub total_attempts: u32,

    /// Number of successful tool calls
    pub successful: u32,

    /// Number of failed tool calls
    pub failed: u32,

    /// Details of failed tool calls
    pub failed_calls: Vec<FailedToolCall>,
}

/// Information about a failed tool call
#[derive(Debug, Clone)]
pub struct FailedToolCall {
    /// Name of the tool that failed
    pub tool_name: String,

    /// Unique ID of the tool call
    pub tool_use_id: String,

    /// Error message describing the failure
    pub error_message: String,

    /// Duration of the failed attempt
    pub duration: Duration,
}

// Python-like string conversion - returns just the response text
impl std::fmt::Display for AgentResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.response)
    }
}

impl AgentResult {
    /// Convert from EventLoopResult to unified AgentResult
    pub fn from(event_result: EventLoopResult, total_duration: Duration) -> Self {
        let execution_details = ExecutionDetails {
            cycles: event_result.cycles_executed,
            model_calls: event_result.metrics.total_model_calls(),
            tool_executions: event_result.metrics.total_tool_calls(),
            tokens: {
                let total_tokens = event_result.metrics.total_tokens.total_tokens;
                if total_tokens > 0 {
                    Some(TokenUsage {
                        input_tokens: event_result.metrics.total_tokens.input_tokens,
                        output_tokens: event_result.metrics.total_tokens.output_tokens,
                        total_tokens,
                    })
                } else {
                    // Always include token usage even if 0 for debugging
                    Some(TokenUsage {
                        input_tokens: event_result.metrics.total_tokens.input_tokens,
                        output_tokens: event_result.metrics.total_tokens.output_tokens,
                        total_tokens,
                    })
                }
            },
            performance: PerformanceMetrics::from(&event_result.metrics, event_result.was_streamed),
        };

        let successful_tools = event_result.metrics.tools_successful();
        let failed_tools = event_result.metrics.tools_failed();
        let failed_calls = event_result.metrics.failed_tool_calls();

        let tool_call_summary = ToolCallSummary {
            total_attempts: event_result.metrics.total_tool_calls(),
            successful: event_result.metrics.summary().successful_tool_executions,
            failed: event_result.metrics.summary().failed_tool_executions,
            failed_calls,
        };

        Self {
            response: event_result.response,
            execution: execution_details,
            used_tools: event_result.metrics.total_tool_calls() > 0,
            tools_called: event_result.metrics.tools_used(),
            tools_successful: successful_tools,
            tools_failed: failed_tools,
            tool_call_summary,
            duration: total_duration,
            success: event_result.success,
            error: event_result.error,
        }
    }

    /// Create a simple success result (for non-agentic execution)
    pub fn simple_success(response: String, duration: Duration) -> Self {
        Self {
            response,
            execution: ExecutionDetails {
                cycles: 1,
                model_calls: 1,
                tool_executions: 0,
                tokens: None,
                performance: PerformanceMetrics {
                    avg_cycle_time: duration,
                    model_time: duration,
                    tool_time: Duration::ZERO,
                    was_streamed: false,
                },
            },
            used_tools: false,
            tools_called: Vec::new(),
            tools_successful: Vec::new(),
            tools_failed: Vec::new(),
            tool_call_summary: ToolCallSummary {
                total_attempts: 0,
                successful: 0,
                failed: 0,
                failed_calls: Vec::new(),
            },
            duration,
            success: true,
            error: None,
        }
    }

    /// Create an error result
    pub fn error(error_message: String, duration: Duration) -> Self {
        Self {
            response: String::new(),
            execution: ExecutionDetails {
                cycles: 0,
                model_calls: 0,
                tool_executions: 0,
                tokens: None,
                performance: PerformanceMetrics {
                    avg_cycle_time: Duration::ZERO,
                    model_time: Duration::ZERO,
                    tool_time: Duration::ZERO,
                    was_streamed: false,
                },
            },
            used_tools: false,
            tools_called: Vec::new(),
            tools_successful: Vec::new(),
            tools_failed: Vec::new(),
            tool_call_summary: ToolCallSummary {
                total_attempts: 0,
                successful: 0,
                failed: 0,
                failed_calls: Vec::new(),
            },
            duration,
            success: false,
            error: Some(error_message),
        }
    }
}

impl PerformanceMetrics {
    /// Convert from EventLoopMetrics
    pub fn from(metrics: &EventLoopMetrics, was_streamed: bool) -> Self {
        let total_cycles = metrics.total_cycles();
        let avg_cycle_time = if total_cycles > 0 {
            metrics.total_execution_time() / total_cycles
        } else {
            Duration::ZERO
        };

        Self {
            avg_cycle_time,
            model_time: metrics.total_model_time(),
            tool_time: metrics.total_tool_time(),
            was_streamed,
        }
    }
}

impl Default for AgentResult {
    fn default() -> Self {
        Self {
            response: String::new(),
            execution: ExecutionDetails {
                cycles: 0,
                model_calls: 0,
                tool_executions: 0,
                tokens: None,
                performance: PerformanceMetrics {
                    avg_cycle_time: Duration::ZERO,
                    model_time: Duration::ZERO,
                    tool_time: Duration::ZERO,
                    was_streamed: false,
                },
            },
            used_tools: false,
            tools_called: Vec::new(),
            tools_successful: Vec::new(),
            tools_failed: Vec::new(),
            tool_call_summary: ToolCallSummary {
                total_attempts: 0,
                successful: 0,
                failed: 0,
                failed_calls: Vec::new(),
            },
            duration: Duration::ZERO,
            success: false,
            error: None,
        }
    }
}
