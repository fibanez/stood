//! Enterprise-grade observability with smart auto-detection and OpenTelemetry integration.
//!
//! This module provides comprehensive telemetry capabilities following OpenTelemetry
//! standards and GenAI semantic conventions. It's designed to be the primary
//! observability strategy for production AI agent deployments with intelligent
//! endpoint detection and graceful degradation.
//!
//! # Quick Start
//!
//! Enable telemetry with automatic endpoint detection:
//! ```no_run
//! use stood::agent::Agent;
//! use stood::llm::models::Bedrock;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Telemetry with smart auto-detection (recommended)
//!     let mut agent = Agent::builder()
//!         .model(Bedrock::Claude35Haiku)
//!         .with_telemetry_from_env()  // Auto-detects OTLP endpoints
//!         .build().await?;
//!     
//!     let result = agent.execute("Hello, world!").await?;
//!     println!("Response: {}", result.response);
//!     println!("Tokens used: {}", result.execution.token_usage.total_tokens);
//!     
//!     Ok(())
//! }
//! ```
//!
//! Configure telemetry explicitly for production:
//! ```no_run
//! use stood::agent::Agent;
//! use stood::telemetry::TelemetryConfig;
//! use stood::llm::models::Bedrock;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = TelemetryConfig::default()
//!         .with_otlp_endpoint("https://api.honeycomb.io")
//!         .with_service_name("my-ai-agent")
//!         .with_batch_processing()  // Higher throughput
//!         .with_log_level(crate::telemetry::LogLevel::INFO);
//!     
//!     let mut agent = Agent::builder()
//!         .model(Bedrock::Claude35Haiku)
//!         .with_telemetry(config)
//!         .build().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Key Features
//!
//! - **Smart Auto-Detection** - Automatically discovers available OTLP endpoints
//! - **Graceful Degradation** - Falls back to console export or disables cleanly
//! - **Production Integrations** - Native support for major observability platforms
//! - **GenAI Semantic Conventions** - Industry-standard AI workload observability
//! - **Debug Visibility** - Complete OTLP export logging for troubleshooting
//! - **Performance Metrics** - Token usage, latency, and throughput tracking
//! - **Multi-Provider Support** - Works with all LLM providers (Bedrock, LM Studio, etc.)
//!
//! # Key Types
//!
//! - [`TelemetryConfig`] - Configuration with smart auto-detection
//! - [`EventLoopMetrics`] - Agent performance and execution metrics
//! - [`TokenUsage`] - Token consumption tracking across providers
//! - [`ToolExecutionMetric`] - Individual tool performance monitoring

use crate::StoodError;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;
use std::fmt;

pub mod logging;
pub mod otlp_debug;


pub mod metrics;


pub mod otel;

#[cfg(test)]
pub mod test_harness;

pub use metrics::*;


pub use otel::*;


pub use opentelemetry::KeyValue;

pub use logging::*;

/// Log level for telemetry output control
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// No telemetry output at all
    OFF,
    /// Only error messages
    ERROR,
    /// Error and warning messages
    WARN,
    /// Error, warning, and info messages
    INFO,
    /// Error, warning, info, and debug messages
    DEBUG,
    /// All messages including trace
    TRACE,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::INFO
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::OFF => write!(f, "OFF"),
            LogLevel::ERROR => write!(f, "ERROR"),
            LogLevel::WARN => write!(f, "WARN"),
            LogLevel::INFO => write!(f, "INFO"),
            LogLevel::DEBUG => write!(f, "DEBUG"),
            LogLevel::TRACE => write!(f, "TRACE"),
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "OFF" => Ok(LogLevel::OFF),
            "ERROR" => Ok(LogLevel::ERROR),
            "WARN" | "WARNING" => Ok(LogLevel::WARN),
            "INFO" => Ok(LogLevel::INFO),
            "DEBUG" => Ok(LogLevel::DEBUG),
            "TRACE" => Ok(LogLevel::TRACE),
            _ => Err(format!("Invalid log level: {}", s)),
        }
    }
}

/// Configuration for telemetry and observability
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled
    pub enabled: bool,
    /// OTLP endpoint for production telemetry export
    pub otlp_endpoint: Option<String>,
    /// Whether to export to console for development
    pub console_export: bool,
    /// Service name for telemetry identification
    pub service_name: String,
    /// Service version for telemetry identification
    pub service_version: String,
    /// Whether to use batch processor for better performance
    pub enable_batch_processor: bool,
    /// Export mode: "batch" (default), "immediate", or "simple"
    pub export_mode: String,
    /// Additional service attributes for telemetry
    pub service_attributes: HashMap<String, String>,
    /// Whether to enable detailed tracing for debugging
    pub enable_debug_tracing: bool,
    /// Log level for telemetry console output
    pub log_level: LogLevel,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,  // Developer must explicitly enable telemetry
            otlp_endpoint: None,
            console_export: false,
            service_name: "stood-agent".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            enable_batch_processor: false,  // Default to simple spans for development
            export_mode: "simple".to_string(),
            service_attributes: HashMap::new(),
            enable_debug_tracing: false,
            log_level: LogLevel::default(),
        }
    }
}

impl TelemetryConfig {
    /// Enable batch processing for production environments (higher throughput)
    pub fn with_batch_processing(mut self) -> Self {
        self.enable_batch_processor = true;
        self.export_mode = "batch".to_string();
        self
    }
    
    /// Enable simple processing for development/debugging (immediate export)
    pub fn with_simple_processing(mut self) -> Self {
        self.enable_batch_processor = false;
        self.export_mode = "simple".to_string();
        self
    }
    
    /// Set service name for telemetry identification
    pub fn with_service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = name.into();
        self
    }
    
    /// Set service version for telemetry identification
    pub fn with_service_version(mut self, version: impl Into<String>) -> Self {
        self.service_version = version.into();
        self
    }
    
    /// Set explicit OTLP endpoint
    pub fn with_otlp_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.otlp_endpoint = Some(endpoint.into());
        self
    }
    
    /// Enable console export for local debugging
    pub fn with_console_export(mut self) -> Self {
        self.console_export = true;
        self
    }
    
    /// Set log level for telemetry output
    pub fn with_log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }
    
    /// Set log level from string
    pub fn with_log_level_str(mut self, level: &str) -> Result<Self, String> {
        let parsed_level = level.parse::<LogLevel>()?;
        self.log_level = parsed_level;
        Ok(self)
    }
    
    /// Check if the given log level should be printed
    pub fn should_log(&self, level: LogLevel) -> bool {
        self.log_level >= level
    }
    
    /// Print an info message if log level allows
    pub fn log_info(&self, message: &str) {
        if self.should_log(LogLevel::INFO) {
            eprintln!("{}", message);
        }
    }
    
    /// Print a debug message if log level allows
    pub fn log_debug(&self, message: &str) {
        if self.should_log(LogLevel::DEBUG) {
            eprintln!("{}", message);
        }
    }
    
    /// Print a warning message if log level allows
    pub fn log_warn(&self, message: &str) {
        if self.should_log(LogLevel::WARN) {
            eprintln!("{}", message);
        }
    }
    
    /// Print an error message if log level allows
    pub fn log_error(&self, message: &str) {
        if self.should_log(LogLevel::ERROR) {
            eprintln!("{}", message);
        }
    }

    /// Create smart telemetry configuration that auto-detects available endpoints
    ///
    /// This method implements intelligent endpoint detection and graceful degradation:
    /// 1. Checks for OTLP endpoints (environment or auto-detect common ports)
    /// 2. Falls back to console export in development
    /// 3. Disables telemetry only if explicitly requested
    ///
    /// Environment variables (all optional):
    /// - `OTEL_ENABLED`: Enable/disable telemetry (default: auto-detect)
    /// - `OTEL_EXPORTER_OTLP_ENDPOINT`: OTLP endpoint URL (default: auto-detect)
    /// - `OTEL_SERVICE_NAME`: Service name
    /// - `OTEL_SERVICE_VERSION`: Service version
    /// - `STOOD_OTEL_ENABLE_CONSOLE_EXPORT`: Force console export
    /// - `OTEL_BATCH_PROCESSOR`: Enable/disable batch processor
    /// - `STOOD_OTEL_DEBUG_TRACING`: Enable detailed debug tracing
    pub fn from_env() -> Self {
        Self::from_env_with_detection()
    }


    /// Create smart telemetry configuration with endpoint auto-detection
    pub fn from_env_with_detection() -> Self {
        // Check if telemetry is explicitly disabled
        if let Ok(enabled) = std::env::var("OTEL_ENABLED") {
            if enabled.to_lowercase() == "false" || enabled == "0" {
                return Self {
                    enabled: false,
                    ..Self::default()
                };
            }
        }

        let mut config = Self {
            enabled: true, // Default to enabled unless explicitly disabled
            otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            console_export: std::env::var("STOOD_OTEL_ENABLE_CONSOLE_EXPORT")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            service_name: std::env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "stood-agent".to_string()),
            service_version: std::env::var("OTEL_SERVICE_VERSION")
                .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string()),
            enable_batch_processor: std::env::var("OTEL_BATCH_PROCESSOR")
                .unwrap_or_else(|_| "false".to_string())  // Default to simple
                .parse()
                .unwrap_or(false),
            export_mode: std::env::var("STOOD_OTEL_EXPORT_MODE")
                .unwrap_or_else(|_| "simple".to_string()),
            service_attributes: HashMap::new(),
            enable_debug_tracing: std::env::var("STOOD_OTEL_DEBUG_TRACING")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            log_level: LogLevel::INFO,
        };

        // Collect service attributes from environment
        for (key, value) in std::env::vars() {
            if let Some(attr_name) = key.strip_prefix("OTEL_SERVICE_ATTRIBUTE_") {
                config
                    .service_attributes
                    .insert(attr_name.to_lowercase(), value);
            }
        }

        // Smart endpoint detection if no explicit endpoint provided
        if config.otlp_endpoint.is_none() {
            config.otlp_endpoint = Self::detect_otlp_endpoint(&config);
        }
        
        // Support for cloud provider auto-detection
        if config.otlp_endpoint.is_none() {
            config.otlp_endpoint = Self::detect_cloud_endpoints(&config);
        }

        // Graceful degradation strategy
        if config.otlp_endpoint.is_none() && !config.console_export {
            // In development, default to console export
            #[cfg(debug_assertions)]
            {
                config.console_export = true;
                config.log_debug("🔍 Stood: No OTLP endpoint detected, using console export for development");
            }
            
            // In production, disable telemetry gracefully
            #[cfg(not(debug_assertions))]
            {
                config.enabled = false;
                config.log_warn("⚠️  Stood: No telemetry endpoint available, disabling telemetry");
            }
        }

        config
    }

    /// Auto-detect available OTLP endpoints by checking common ports
    fn detect_otlp_endpoint(config: &TelemetryConfig) -> Option<String> {
        let common_endpoints = [
            // Standard OTLP HTTP endpoints
            "http://localhost:4318",              // Standard OTLP HTTP
            "http://localhost:4320",              // Common Docker mapping
            "http://127.0.0.1:4318",             // Explicit localhost
            
            // Docker and container environments
            "http://otel-collector:4318",        // Docker compose
            "http://jaeger-collector:14268",     // Jaeger HTTP
            "http://tempo:3200",                 // Grafana Tempo
            
            // Kubernetes service discovery
            "http://opentelemetry-collector:4318",
            "http://jaeger-collector.observability:14268",
            "http://tempo.observability:3200",
            
            // Development alternatives
            "http://localhost:8080",             // Custom dev setups
            "http://localhost:9411",             // Zipkin
        ];

        config.log_debug("🔍 Stood: Starting OTLP endpoint detection...");
        
        for endpoint in &common_endpoints {
            config.log_debug(&format!("🔍 Stood: Testing endpoint: {}", endpoint));
            if Self::check_endpoint_availability(endpoint, config) {
                config.log_info(&format!("🎯 Stood: Auto-detected OTLP endpoint: {}", endpoint));
                
                // Also log to our debug system
                crate::telemetry::otlp_debug::log_otlp_export(
                    "telemetry::TelemetryConfig::detect_otlp_endpoint",
                    crate::telemetry::otlp_debug::OtlpExportType::Traces,
                    endpoint,
                    "Endpoint auto-detection SUCCESS",
                    0,
                    &Ok(()),
                );
                
                return Some(endpoint.to_string());
            } else {
                config.log_debug(&format!("❌ Stood: Endpoint not available: {}", endpoint));
            }
        }

        config.log_warn("⚠️ Stood: No OTLP endpoints detected on common ports");
        
        // Log failed detection
        crate::telemetry::otlp_debug::log_otlp_export(
            "telemetry::TelemetryConfig::detect_otlp_endpoint",
            crate::telemetry::otlp_debug::OtlpExportType::Traces,
            "none",
            "Endpoint auto-detection FAILED - no endpoints available",
            0,
            &Err("No endpoints responded on ports 4318, 4320".to_string()),
        );

        None
    }

    /// Check if an OTLP endpoint is available (non-blocking)
    fn check_endpoint_availability(endpoint: &str, config: &TelemetryConfig) -> bool {
        // Quick TCP connection test to see if the port is open
        if let Ok(url) = url::Url::parse(endpoint) {
            if let Some(host) = url.host_str() {
                let port = url.port().unwrap_or(4318);
                
                // Use a very short timeout for detection
                use std::net::TcpStream;
                use std::time::Duration;
                
                // Use ToSocketAddrs to handle hostname resolution
                let addr_str = format!("{}:{}", host, port);
                match std::net::ToSocketAddrs::to_socket_addrs(&addr_str) {
                    Ok(mut addrs) => {
                        if let Some(addr) = addrs.next() {
                            let result = TcpStream::connect_timeout(&addr, Duration::from_millis(100));
                            match result {
                                Ok(_) => {
                                    config.log_debug(&format!("✅ Stood: TCP connection successful to {}:{}", host, port));
                                    return true;
                                }
                                Err(e) => {
                                    config.log_debug(&format!("❌ Stood: TCP connection failed to {}:{} - {}", host, port, e));
                                    return false;
                                }
                            }
                        } else {
                            config.log_debug(&format!("❌ Stood: No socket addresses resolved for: {}:{}", host, port));
                        }
                    }
                    Err(e) => {
                        config.log_debug(&format!("❌ Stood: Failed to resolve hostname {}:{} - {}", host, port, e));
                    }
                }
            } else {
                config.log_debug(&format!("❌ Stood: No host found in URL: {}", endpoint));
            }
        } else {
            config.log_debug(&format!("❌ Stood: Invalid URL format: {}", endpoint));
        }
        false
    }

    /// Detect cloud provider OTLP endpoints based on environment
    fn detect_cloud_endpoints(config: &TelemetryConfig) -> Option<String> {
        // AWS X-Ray OTLP support
        if let Ok(aws_region) = std::env::var("AWS_REGION") {
            if std::env::var("AWS_ACCESS_KEY_ID").is_ok() || std::env::var("AWS_PROFILE").is_ok() {
                let aws_endpoint = format!("https://otlp.{}.amazonaws.com", aws_region);
                config.log_debug(&format!("🔍 Stood: Detected AWS environment, trying: {}", aws_endpoint));
                return Some(aws_endpoint);
            }
        }

        // Honeycomb detection
        if std::env::var("HONEYCOMB_API_KEY").is_ok() {
            config.log_debug("🔍 Stood: Detected Honeycomb environment");
            return Some("https://api.honeycomb.io".to_string());
        }

        // New Relic detection
        if std::env::var("NEW_RELIC_LICENSE_KEY").is_ok() {
            config.log_debug("🔍 Stood: Detected New Relic environment");
            return Some("https://otlp.nr-data.net".to_string());
        }

        // Datadog detection  
        if std::env::var("DD_API_KEY").is_ok() {
            let region = std::env::var("DD_SITE").unwrap_or_else(|_| "datadoghq.com".to_string());
            let dd_endpoint = format!("https://api.{}", region);
            config.log_debug(&format!("🔍 Stood: Detected Datadog environment: {}", dd_endpoint));
            return Some(dd_endpoint);
        }

        // Google Cloud Trace (if running on GCP)
        if std::env::var("GOOGLE_CLOUD_PROJECT").is_ok() {
            config.log_debug("🔍 Stood: Detected Google Cloud environment");
            return Some("https://cloudtrace.googleapis.com".to_string());
        }

        // Custom enterprise endpoints from environment
        if let Ok(custom_endpoints) = std::env::var("STOOD_OTLP_ENDPOINTS") {
            let endpoints: Vec<&str> = custom_endpoints.split(',').collect();
            for endpoint in endpoints {
                let endpoint = endpoint.trim();
                config.log_debug(&format!("🔍 Stood: Testing custom endpoint: {}", endpoint));
                if Self::check_endpoint_availability(endpoint, config) {
                    config.log_info(&format!("🎯 Stood: Custom endpoint available: {}", endpoint));
                    return Some(endpoint.to_string());
                }
            }
        }

        None
    }

    /// Create a minimal configuration for testing
    pub fn for_testing() -> Self {
        Self {
            enabled: true,
            console_export: true,
            service_name: "stood-agent-test".to_string(),
            enable_debug_tracing: true,
            ..Self::default()
        }
    }

    /// Validate the telemetry configuration with smart fallbacks
    pub fn validate(&self) -> Result<(), StoodError> {
        if !self.enabled {
            return Ok(());
        }

        // Smart validation - allow configurations without explicit export methods
        // as the system will auto-detect or gracefully degrade
        
        if let Some(endpoint) = &self.otlp_endpoint {
            if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
                return Err(StoodError::configuration_error(format!(
                    "Invalid OTLP endpoint: {}. Must start with http:// or https://",
                    endpoint
                )));
            }
        }

        if self.service_name.is_empty() {
            return Err(StoodError::configuration_error(
                "Service name cannot be empty when telemetry is enabled",
            ));
        }

        Ok(())
    }

}

/// Metrics collected during event loop execution
///
/// Follows the Python reference implementation's EventLoopMetrics structure
/// for compatibility and comprehensive monitoring.
#[derive(Debug, Clone, Default)]
pub struct EventLoopMetrics {
    /// Individual model interaction cycle metrics for detailed analysis
    pub cycles: Vec<CycleMetrics>,
    /// Total token usage across all model interaction cycles
    pub total_tokens: TokenUsage,
    /// Total duration of the event loop
    pub total_duration: Duration,
    /// All tool executions with timing and status
    pub tool_executions: Vec<ToolExecutionMetric>,
    /// Trace information for correlation with external systems
    pub traces: Vec<TraceInfo>,
    /// Accumulated metrics for summary reporting
    pub accumulated_usage: AccumulatedMetrics,
}

impl EventLoopMetrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a model interaction cycle to the metrics
    pub fn add_cycle(&mut self, cycle: CycleMetrics) {
        self.total_tokens.input_tokens += cycle.tokens_used.input_tokens;
        self.total_tokens.output_tokens += cycle.tokens_used.output_tokens;
        self.total_tokens.total_tokens += cycle.tokens_used.total_tokens;
        self.total_duration += cycle.duration;

        self.accumulated_usage.total_cycles += 1; // Increment model interaction cycle count
        self.accumulated_usage.total_model_invocations += cycle.model_invocations;
        self.accumulated_usage.total_tool_calls += cycle.tool_calls;

        self.cycles.push(cycle);
    }

    /// Add a tool execution to the metrics
    pub fn add_tool_execution(&mut self, execution: ToolExecutionMetric) {
        self.tool_executions.push(execution);
    }

    /// Add trace information
    pub fn add_trace(&mut self, trace: TraceInfo) {
        self.traces.push(trace);
    }

    /// Get total number of model interaction cycles executed
    pub fn total_cycles(&self) -> u32 {
        self.cycles.len() as u32
    }

    /// Get total number of model calls across all model interaction cycles
    pub fn total_model_calls(&self) -> u32 {
        self.accumulated_usage.total_model_invocations
    }

    /// Get total number of tool calls across all model interaction cycles
    pub fn total_tool_calls(&self) -> u32 {
        self.accumulated_usage.total_tool_calls
    }

    /// Get total execution time
    pub fn total_execution_time(&self) -> Duration {
        self.total_duration
    }
    
    /// Get total input tokens across all model interaction cycles
    pub fn total_input_tokens(&self) -> u32 {
        self.total_tokens.input_tokens
    }
    
    /// Get total output tokens across all model interaction cycles
    pub fn total_output_tokens(&self) -> u32 {
        self.total_tokens.output_tokens
    }
    
    /// Get total tokens (input + output)
    pub fn total_tokens(&self) -> u32 {
        self.total_tokens.total_tokens
    }

    /// Get total time spent on model calls
    pub fn total_model_time(&self) -> Duration {
        // For now, approximate as a portion of total time
        // In a real implementation, this would track model time specifically
        self.total_duration / 2
    }

    /// Get total time spent on tool execution
    pub fn total_tool_time(&self) -> Duration {
        self.tool_executions.iter().map(|t| t.duration).sum()
    }

    /// Get list of unique tools used (both successful and failed)
    pub fn tools_used(&self) -> Vec<String> {
        let mut tools: Vec<String> = self
            .tool_executions
            .iter()
            .map(|t| t.tool_name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        tools.sort();
        tools
    }

    /// Get list of tools that completed successfully
    pub fn tools_successful(&self) -> Vec<String> {
        let mut tools: Vec<String> = self
            .tool_executions
            .iter()
            .filter(|t| t.success)
            .map(|t| t.tool_name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        tools.sort();
        tools
    }

    /// Get list of tools that failed
    pub fn tools_failed(&self) -> Vec<String> {
        let mut tools: Vec<String> = self
            .tool_executions
            .iter()
            .filter(|t| !t.success)
            .map(|t| t.tool_name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        tools.sort();
        tools
    }

    /// Get detailed information about failed tool calls
    pub fn failed_tool_calls(&self) -> Vec<crate::agent::result::FailedToolCall> {
        self.tool_executions
            .iter()
            .filter(|t| !t.success)
            .map(|t| crate::agent::result::FailedToolCall {
                tool_name: t.tool_name.clone(),
                tool_use_id: t.tool_use_id.clone().unwrap_or_else(|| 
                    format!("execution_{}", t.start_time.timestamp_nanos_opt().unwrap_or(0))
                ),
                error_message: t.error.clone().unwrap_or_else(|| "Unknown error".to_string()),
                duration: t.duration,
            })
            .collect()
    }

    /// Get summary statistics
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            total_cycles: self.cycles.len() as u32,
            total_duration: self.total_duration,
            total_tokens: self.total_tokens.clone(),
            average_cycle_duration: if self.cycles.is_empty() {
                Duration::ZERO
            } else {
                self.total_duration / self.cycles.len() as u32
            },
            successful_tool_executions: self.tool_executions.iter().filter(|t| t.success).count()
                as u32,
            failed_tool_executions: self.tool_executions.iter().filter(|t| !t.success).count()
                as u32,
            unique_tools_used: self
                .tool_executions
                .iter()
                .map(|t| &t.tool_name)
                .collect::<std::collections::HashSet<_>>()
                .len() as u32,
        }
    }
}

/// Metrics for an individual event loop cycle
#[derive(Debug, Clone)]
pub struct CycleMetrics {
    /// Unique identifier for this cycle
    pub cycle_id: Uuid,
    /// Duration of this cycle
    pub duration: Duration,
    /// Number of model invocations in this cycle
    pub model_invocations: u32,
    /// Number of tool calls in this cycle
    pub tool_calls: u32,
    /// Tokens used in this cycle
    pub tokens_used: TokenUsage,
    /// Associated trace ID for correlation
    pub trace_id: Option<String>,
    /// Associated span ID for correlation
    pub span_id: Option<String>,
    /// Timestamp when the cycle started
    pub start_time: DateTime<Utc>,
    /// Whether the cycle completed successfully
    pub success: bool,
    /// Error message if the cycle failed
    pub error: Option<String>,
}

impl CycleMetrics {
    /// Create new cycle metrics
    pub fn new(cycle_id: Uuid) -> Self {
        Self {
            cycle_id,
            duration: Duration::ZERO,
            model_invocations: 0,
            tool_calls: 0,
            tokens_used: TokenUsage::default(),
            trace_id: None,
            span_id: None,
            start_time: Utc::now(),
            success: false,
            error: None,
        }
    }

    /// Mark the cycle as completed successfully
    pub fn complete_success(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self.success = true;
        self
    }

    /// Mark the cycle as failed
    pub fn complete_error(mut self, duration: Duration, error: String) -> Self {
        self.duration = duration;
        self.success = false;
        self.error = Some(error);
        self
    }
}

/// Token usage information
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

impl TokenUsage {
    /// Create new token usage
    pub fn new(input_tokens: u32, output_tokens: u32) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
        }
    }

    /// Add token usage
    pub fn add(&mut self, other: &TokenUsage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.total_tokens += other.total_tokens;
    }
}

/// Metrics for tool execution
#[derive(Debug, Clone)]
pub struct ToolExecutionMetric {
    /// Name of the tool that was executed
    pub tool_name: String,
    /// Unique ID of the tool use/call
    pub tool_use_id: Option<String>,
    /// Duration of the tool execution
    pub duration: Duration,
    /// Whether the execution was successful
    pub success: bool,
    /// Error message if execution failed
    pub error: Option<String>,
    /// Associated trace ID for correlation
    pub trace_id: Option<String>,
    /// Associated span ID for correlation
    pub span_id: Option<String>,
    /// Timestamp when execution started
    pub start_time: DateTime<Utc>,
    /// Tool input size (for performance analysis)
    pub input_size_bytes: Option<usize>,
    /// Tool output size (for performance analysis)
    pub output_size_bytes: Option<usize>,
}

/// Trace information for correlation with external observability systems
#[derive(Debug, Clone)]
pub struct TraceInfo {
    /// OpenTelemetry trace ID
    pub trace_id: String,
    /// OpenTelemetry span ID
    pub span_id: String,
    /// Operation name for the span
    pub operation: String,
    /// When the span started
    pub start_time: DateTime<Utc>,
    /// Duration of the span
    pub duration: Duration,
    /// Final status of the span
    pub status: SpanStatus,
    /// Custom attributes attached to the span
    pub attributes: HashMap<String, String>,
}

/// Status of a telemetry span
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanStatus {
    /// Operation completed successfully
    Ok,
    /// Operation failed with an error
    Error {
        /// Error message
        message: String,
    },
    /// Operation was cancelled
    Cancelled,
}

/// Accumulated metrics for summary reporting
#[derive(Debug, Clone, Default)]
pub struct AccumulatedMetrics {
    /// Total number of model interaction cycles
    pub total_cycles: u32,
    /// Total number of model invocations across all cycles
    pub total_model_invocations: u32,
    /// Total number of tool calls across all cycles
    pub total_tool_calls: u32,
    /// Total processing time
    pub total_processing_time: Duration,
    /// Number of successful cycles
    pub successful_cycles: u32,
    /// Number of failed cycles
    pub failed_cycles: u32,
}

/// Summary of metrics for quick reporting
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub total_cycles: u32,
    pub total_duration: Duration,
    pub total_tokens: TokenUsage,
    pub average_cycle_duration: Duration,
    pub successful_tool_executions: u32,
    pub failed_tool_executions: u32,
    pub unique_tools_used: u32,
}

/// GenAI semantic conventions for AI workload observability
///
/// Based on OpenTelemetry semantic conventions for Generative AI systems:
/// <https://opentelemetry.io/docs/specs/semconv/gen-ai/>
pub mod semantic_conventions {
    /// GenAI system attributes
    pub const GEN_AI_SYSTEM: &str = "gen_ai.system";
    pub const GEN_AI_REQUEST_MODEL: &str = "gen_ai.request.model";
    pub const GEN_AI_REQUEST_MAX_TOKENS: &str = "gen_ai.request.max_tokens";
    pub const GEN_AI_REQUEST_TEMPERATURE: &str = "gen_ai.request.temperature";
    pub const GEN_AI_REQUEST_TOP_P: &str = "gen_ai.request.top_p";
    pub const GEN_AI_REQUEST_TOP_K: &str = "gen_ai.request.top_k";
    pub const GEN_AI_REQUEST_PRESENCE_PENALTY: &str = "gen_ai.request.presence_penalty";
    pub const GEN_AI_REQUEST_FREQUENCY_PENALTY: &str = "gen_ai.request.frequency_penalty";

    /// GenAI response attributes
    pub const GEN_AI_RESPONSE_ID: &str = "gen_ai.response.id";
    pub const GEN_AI_RESPONSE_MODEL: &str = "gen_ai.response.model";
    pub const GEN_AI_RESPONSE_FINISH_REASONS: &str = "gen_ai.response.finish_reasons";
    
    /// Enhanced GenAI response attributes (Phase 5)
    pub const GEN_AI_RESPONSE_CONTENT_PREVIEW: &str = "gen_ai.response.content_preview";
    pub const GEN_AI_RESPONSE_CONTENT_LENGTH: &str = "gen_ai.response.content_length";
    pub const GEN_AI_RESPONSE_TOOL_CALLS_COUNT: &str = "gen_ai.response.tool_calls_count";
    pub const GEN_AI_RESPONSE_TOOL_NAMES: &str = "gen_ai.response.tool_names";
    pub const GEN_AI_RESPONSE_FINISH_REASON: &str = "gen_ai.response.finish_reason";
    pub const GEN_AI_RESPONSE_TYPE: &str = "gen_ai.response.type";

    /// GenAI usage attributes
    pub const GEN_AI_USAGE_INPUT_TOKENS: &str = "gen_ai.usage.input_tokens";
    pub const GEN_AI_USAGE_OUTPUT_TOKENS: &str = "gen_ai.usage.output_tokens";
    pub const GEN_AI_USAGE_TOTAL_TOKENS: &str = "gen_ai.usage.total_tokens";

    /// GenAI operation attributes  
    pub const GEN_AI_OPERATION_NAME: &str = "gen_ai.operation.name";
    pub const GEN_AI_TOOL_NAME: &str = "gen_ai.tool.name";

    /// GenAI prompt and content attributes
    pub const GEN_AI_PROMPT: &str = "gen_ai.prompt";
    pub const GEN_AI_COMPLETION: &str = "gen_ai.completion";
    pub const GEN_AI_USER: &str = "gen_ai.user.id";
    pub const GEN_AI_SESSION: &str = "gen_ai.session.id";

    /// GenAI provider-specific attributes
    pub const GEN_AI_PROVIDER: &str = "gen_ai.provider";
    pub const GEN_AI_ENDPOINT: &str = "gen_ai.endpoint";
    pub const GEN_AI_MODEL_VERSION: &str = "gen_ai.model.version";
    
    /// Dynamic Provider.Model System Attributes (Phase 7)
    pub const GEN_AI_PROVIDER_TYPE: &str = "gen_ai.provider.type";
    pub const GEN_AI_PROVIDER_NAME: &str = "gen_ai.provider.name";
    pub const GEN_AI_MODEL_FAMILY: &str = "gen_ai.model.family";
    pub const GEN_AI_MODEL_DISPLAY_NAME: &str = "gen_ai.model.display_name";
    pub const GEN_AI_MODEL_CAPABILITIES: &str = "gen_ai.model.capabilities";
    pub const STOOD_PROVIDER_TYPE: &str = "stood.provider.type";
    pub const STOOD_MODEL_SUPPORTS_TOOLS: &str = "stood.model.supports_tools";
    pub const STOOD_MODEL_SUPPORTS_STREAMING: &str = "stood.model.supports_streaming";
    pub const STOOD_MODEL_MAX_TOKENS: &str = "stood.model.max_tokens";

    /// Stood-specific attributes
    pub const STOOD_AGENT_ID: &str = "stood.agent.id";
    pub const STOOD_CONVERSATION_ID: &str = "stood.conversation.id";
    pub const STOOD_TOOL_EXECUTION_ID: &str = "stood.tool.execution_id";
    pub const STOOD_CYCLE_ID: &str = "stood.cycle.id";
    pub const STOOD_VERSION: &str = "stood.version";

    /// Enhanced Semantic Conventions (Phase 8) - Advanced Event Tracking
    pub const GEN_AI_EVENT_NAME: &str = "gen_ai.event.name";
    pub const GEN_AI_EVENT_TIMESTAMP: &str = "gen_ai.event.timestamp";
    pub const GEN_AI_EVENT_DURATION: &str = "gen_ai.event.duration_ms";
    pub const GEN_AI_EVENT_TYPE: &str = "gen_ai.event.type";
    pub const GEN_AI_EVENT_STATUS: &str = "gen_ai.event.status";
    
    /// Enhanced Error Classification
    pub const GEN_AI_ERROR_TYPE: &str = "gen_ai.error.type";
    pub const GEN_AI_ERROR_CODE: &str = "gen_ai.error.code";
    pub const GEN_AI_ERROR_MESSAGE: &str = "gen_ai.error.message";
    pub const GEN_AI_ERROR_RETRY_COUNT: &str = "gen_ai.error.retry_count";
    pub const GEN_AI_ERROR_RECOVERABLE: &str = "gen_ai.error.recoverable";
    
    /// Performance Metrics Conventions  
    pub const GEN_AI_LATENCY_FIRST_TOKEN: &str = "gen_ai.latency.first_token_ms";
    pub const GEN_AI_LATENCY_TOTAL: &str = "gen_ai.latency.total_ms";
    pub const GEN_AI_THROUGHPUT_TOKENS_PER_SECOND: &str = "gen_ai.throughput.tokens_per_second";
    pub const GEN_AI_QUEUE_TIME: &str = "gen_ai.queue_time_ms";
    pub const GEN_AI_PROCESSING_TIME: &str = "gen_ai.processing_time_ms";
    
    /// Extended GenAI Request Attributes
    pub const GEN_AI_REQUEST_STREAMING: &str = "gen_ai.request.streaming";
    pub const GEN_AI_REQUEST_TOOLS_COUNT: &str = "gen_ai.request.tools_count";
    pub const GEN_AI_REQUEST_TOOLS_NAMES: &str = "gen_ai.request.tools_names";
    pub const GEN_AI_REQUEST_PROMPT_TOKENS: &str = "gen_ai.request.prompt_tokens";
    pub const GEN_AI_REQUEST_CONVERSATION_LENGTH: &str = "gen_ai.request.conversation_length";
    
    /// Extended GenAI Response Attributes
    pub const GEN_AI_RESPONSE_LATENCY: &str = "gen_ai.response.latency_ms";
    pub const GEN_AI_RESPONSE_CACHED: &str = "gen_ai.response.cached";
    pub const GEN_AI_RESPONSE_STREAMING: &str = "gen_ai.response.streaming";
    pub const GEN_AI_RESPONSE_TRUNCATED: &str = "gen_ai.response.truncated";
    pub const GEN_AI_RESPONSE_SAFETY_FILTERED: &str = "gen_ai.response.safety_filtered";
    
    /// Enterprise Observability Patterns
    pub const SERVICE_NAME: &str = "service.name";
    pub const SERVICE_VERSION: &str = "service.version";
    pub const SERVICE_INSTANCE_ID: &str = "service.instance.id";
    pub const DEPLOYMENT_ENVIRONMENT: &str = "deployment.environment";
    pub const USER_ID: &str = "user.id";
    pub const SESSION_ID: &str = "session.id";
    pub const REQUEST_ID: &str = "request.id";
    pub const CORRELATION_ID: &str = "correlation.id";
    
    /// Stood-specific Enhanced Attributes
    pub const STOOD_AGENT_NAME: &str = "stood.agent.name";
    pub const STOOD_CONVERSATION_TURNS: &str = "stood.conversation.turns";
    pub const STOOD_MODEL_INTERACTION_ID: &str = "stood.model_interaction.id";
    pub const STOOD_TOOL_PARALLEL_GROUP_ID: &str = "stood.tool.parallel_group_id";
    pub const STOOD_TOOL_EXECUTION_INDEX: &str = "stood.tool.execution_index";
    pub const STOOD_STREAMING_ENABLED: &str = "stood.streaming.enabled";
    pub const STOOD_RETRY_ATTEMPT: &str = "stood.retry.attempt";
    pub const STOOD_RETRY_MAX_ATTEMPTS: &str = "stood.retry.max_attempts";

    /// Common values
    pub const SYSTEM_ANTHROPIC_BEDROCK: &str = "anthropic.bedrock";
    pub const OPERATION_CHAT: &str = "chat";
    pub const OPERATION_TOOL_CALL: &str = "tool_call";
    pub const OPERATION_AGENT_CYCLE: &str = "agent_cycle";
    
    /// Enhanced Operation Values
    pub const OPERATION_MODEL_INFERENCE: &str = "model_inference";
    pub const OPERATION_TOOL_EXECUTION: &str = "tool_execution";
    pub const OPERATION_CONVERSATION_MANAGEMENT: &str = "conversation_management";
    pub const OPERATION_STREAM_PROCESSING: &str = "stream_processing";
    
    /// Event Types
    pub const EVENT_TYPE_REQUEST: &str = "request";
    pub const EVENT_TYPE_RESPONSE: &str = "response";
    pub const EVENT_TYPE_ERROR: &str = "error";
    pub const EVENT_TYPE_RETRY: &str = "retry";
    pub const EVENT_TYPE_TIMEOUT: &str = "timeout";
    pub const EVENT_TYPE_CACHE_HIT: &str = "cache_hit";
    pub const EVENT_TYPE_CACHE_MISS: &str = "cache_miss";
    
    /// Event Status Values
    pub const EVENT_STATUS_SUCCESS: &str = "success";
    pub const EVENT_STATUS_ERROR: &str = "error";
    pub const EVENT_STATUS_TIMEOUT: &str = "timeout";
    pub const EVENT_STATUS_CANCELLED: &str = "cancelled";
    pub const EVENT_STATUS_RETRY: &str = "retry";
    
    /// Error Types
    pub const ERROR_TYPE_NETWORK: &str = "network";
    pub const ERROR_TYPE_AUTHENTICATION: &str = "authentication";
    pub const ERROR_TYPE_AUTHORIZATION: &str = "authorization";
    pub const ERROR_TYPE_RATE_LIMIT: &str = "rate_limit";
    pub const ERROR_TYPE_MODEL_ERROR: &str = "model_error";
    pub const ERROR_TYPE_TOOL_ERROR: &str = "tool_error";
    pub const ERROR_TYPE_VALIDATION: &str = "validation";
    pub const ERROR_TYPE_TIMEOUT: &str = "timeout";
    pub const ERROR_TYPE_INTERNAL: &str = "internal";
}

/// Timer for measuring operation duration
#[derive(Debug)]
pub struct Timer {
    start: Instant,
}

impl Timer {
    /// Start a new timer
    pub fn start(_label: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get the elapsed duration
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Finish the timer and get the duration
    pub fn finish(self) -> Duration {
        self.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.service_name, "stood-agent");
        assert!(!config.enable_batch_processor);  // Default is false for simple processing
        assert_eq!(config.log_level, LogLevel::INFO);
    }

    #[test]
    fn test_telemetry_config_from_env() {
        // Test with minimal environment
        std::env::set_var("OTEL_ENABLED", "true");
        std::env::set_var("OTEL_SERVICE_NAME", "test-service");

        let config = TelemetryConfig::from_env();
        assert!(config.enabled);
        assert_eq!(config.service_name, "test-service");

        // Clean up
        std::env::remove_var("OTEL_ENABLED");
        std::env::remove_var("OTEL_SERVICE_NAME");
    }

    #[test]
    fn test_telemetry_config_validation() {
        let mut config = TelemetryConfig::default();

        // Disabled config should validate
        assert!(config.validate().is_ok());

        // Enabled without explicit export should now validate (graceful degradation)
        config.enabled = true;
        assert!(config.validate().is_ok());

        // Enabled with console export should work
        config.console_export = true;
        assert!(config.validate().is_ok());

        // Enabled with OTLP endpoint should work
        config.console_export = false;
        config.otlp_endpoint = Some("http://localhost:4318".to_string());
        assert!(config.validate().is_ok());

        // Invalid endpoint should fail
        config.otlp_endpoint = Some("invalid-endpoint".to_string());
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_event_loop_metrics() {
        let mut metrics = EventLoopMetrics::new();

        let cycle = CycleMetrics::new(Uuid::new_v4()).complete_success(Duration::from_millis(100));

        metrics.add_cycle(cycle);

        let summary = metrics.summary();
        assert_eq!(summary.total_cycles, 1);
        assert_eq!(summary.total_duration, Duration::from_millis(100));
    }

    #[test]
    fn test_token_usage() {
        let mut usage = TokenUsage::new(100, 50);
        assert_eq!(usage.total_tokens, 150);

        let other = TokenUsage::new(25, 25);
        usage.add(&other);

        assert_eq!(usage.input_tokens, 125);
        assert_eq!(usage.output_tokens, 75);
        assert_eq!(usage.total_tokens, 200);
    }

    #[test]
    fn test_timer() {
        let timer = Timer::start("test_operation");
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.finish();

        assert!(elapsed >= Duration::from_millis(10));
        assert!(elapsed < Duration::from_millis(50)); // Should be fast
    }

    #[test]
    fn test_cycle_metrics() {
        let cycle_id = Uuid::new_v4();
        let cycle = CycleMetrics::new(cycle_id);

        assert_eq!(cycle.cycle_id, cycle_id);
        assert!(!cycle.success);
        assert_eq!(cycle.model_invocations, 0);

        let completed = cycle.complete_success(Duration::from_millis(200));
        assert!(completed.success);
        assert_eq!(completed.duration, Duration::from_millis(200));
    }

    #[test]
    fn test_log_level_functionality() {
        let mut config = TelemetryConfig::default();
        
        // Test default log level
        assert_eq!(config.log_level, LogLevel::INFO);
        assert!(config.should_log(LogLevel::ERROR));
        assert!(config.should_log(LogLevel::WARN));
        assert!(config.should_log(LogLevel::INFO));
        assert!(!config.should_log(LogLevel::DEBUG));
        assert!(!config.should_log(LogLevel::TRACE));

        // Test OFF level
        config.log_level = LogLevel::OFF;
        assert!(!config.should_log(LogLevel::ERROR));
        assert!(!config.should_log(LogLevel::WARN));
        assert!(!config.should_log(LogLevel::INFO));
        assert!(!config.should_log(LogLevel::DEBUG));
        assert!(!config.should_log(LogLevel::TRACE));

        // Test DEBUG level
        config.log_level = LogLevel::DEBUG;
        assert!(config.should_log(LogLevel::ERROR));
        assert!(config.should_log(LogLevel::WARN));
        assert!(config.should_log(LogLevel::INFO));
        assert!(config.should_log(LogLevel::DEBUG));
        assert!(!config.should_log(LogLevel::TRACE));
    }

    #[test]
    fn test_log_level_parsing() {
        let mut config = TelemetryConfig::default();
        
        // Test successful parsing
        config = config.with_log_level_str("OFF").unwrap();
        assert_eq!(config.log_level, LogLevel::OFF);
        
        config = config.with_log_level_str("DEBUG").unwrap();
        assert_eq!(config.log_level, LogLevel::DEBUG);
        
        config = config.with_log_level_str("warn").unwrap();
        assert_eq!(config.log_level, LogLevel::WARN);
        
        // Test error case
        let result = TelemetryConfig::default().with_log_level_str("INVALID");
        assert!(result.is_err());
    }

}
