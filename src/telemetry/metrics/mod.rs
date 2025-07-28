//! Metrics collection and export for the Stood agent library
//!
//! This module provides comprehensive metrics collection for agentic workloads,
//! including token usage, request latency, tool execution, and system resources.
//! Metrics are exported via OpenTelemetry OTLP to Prometheus and other systems.

pub mod collector;
pub mod exporter;
pub mod semantic_conventions;
pub mod system;

pub use collector::*;
pub use exporter::*;
pub use semantic_conventions::*;
pub use system::*;

use std::time::Duration;

/// Core metrics types for agent operations
#[derive(Debug, Clone)]
pub struct TokenMetrics {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

impl TokenMetrics {
    pub fn new(input: u64, output: u64) -> Self {
        Self {
            input_tokens: input,
            output_tokens: output,
            total_tokens: input + output,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub duration: Duration,
    pub success: bool,
    pub model_invocations: u32,
    pub token_metrics: Option<TokenMetrics>,
    pub error_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ToolMetrics {
    pub tool_name: String,
    pub duration: Duration,
    pub success: bool,
    pub retry_attempts: u32,
    pub error_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub memory_usage_bytes: u64,
    pub active_connections: u64,
    pub concurrent_requests: u64,
    pub thread_utilization: f64,
}