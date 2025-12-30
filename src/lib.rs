//! Build AI agents with AWS Bedrock integration and type-safe tool execution.
//!
//! Stood is an AWS Bedrock-focused agent framework that enables you to create
//! production-ready AI agents with Rust's performance and type safety. You'll get
//! compile-time tool validation, streaming responses, and robust error handling
//! for real-world deployments.
//!
//! # Quick Start
//!
//! ## Simple Chat Agent
//!
//! Create a basic chat agent with minimal configuration:
//!
//! ```no_run
//! use stood::agent::Agent;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Simplest possible agent creation - everything auto-configured
//!     let mut agent = Agent::builder().build().await?;
//!
//!     let response = agent.chat("What is the capital of France?").await?;
//!     println!("Agent: {}", response);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Example with Tools
//!
//! Create an agent with custom tools and built-in tools:
//!
//! ```no_run
//! use stood::tools::builtin::CalculatorTool;
//! use stood::{agent::Agent, tool};
//!
//! #[tool]
//! /// Get weather information for a given location
//! async fn get_weather(location: String) -> Result<String, String> {
//!     // Mock weather data - in real usage this would call a weather API
//!     let weather_info = format!(
//!         "The weather in {} is sunny, 72°F with light winds.",
//!         location
//!     );
//!     Ok(weather_info)
//! }
//!
//! #[tool]
//! /// Calculate a percentage of a value
//! async fn calculate_percentage(value: f64, percentage: f64) -> Result<f64, String> {
//!     if percentage < 0.0 || percentage > 100.0 {
//!         return Err("Percentage must be between 0 and 100".to_string());
//!     }
//!     Ok(value * percentage / 100.0)
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // ✅ Hybrid approach: mix macro tools with struct tools seamlessly
//!     let tools = vec![
//!         get_weather(),                                                  // ✅ Macro tool
//!         calculate_percentage(),                                         // ✅ Macro tool
//!         Box::new(CalculatorTool::new()) as Box<dyn stood::tools::Tool>, // ✅ Struct tool
//!     ];
//!
//!     // Create agent with builder pattern
//!     let mut agent = Agent::builder().tools(tools).build().await?;
//!
//!     // non-agentic chat - simple conversation
//!     println!("=== non-agentic chat ===");
//!     let chat_response = agent.chat("what is the capital of france?").await?;
//!     println!("agent: {}", chat_response);
//!
//!     // agentic execution - complex task that will use multiple tool types
//!     println!("\n=== agentic execution ===");
//!     let agentic_result = agent.execute_agentic(
//!         "what's the weather like in san francisco and what's 15% of a $67 restaurant bill? also calculate 2+3 using the calculator."
//!     ).await?;
//!
//!     println!("agent: {}", agentic_result.response);
//!     println!("cycles executed: {}", agentic_result.cycles_executed);
//!     println!("duration: {:?}", agentic_result.total_duration);
//!
//!     println!("\n✅ successfully demonstrated hybrid tool approach:");
//!     println!("   - macro tools: get_weather(), calculate_percentage()");
//!     println!("   - struct tools: calculatortool");
//!     println!("   - all work seamlessly with Agent::builder().tools(tools).build().await?");
//!
//!     Ok(())
//! }
//! ```
//!
//! # Architecture Overview
//!
//! Stood consists of three core components that work together:
//!
//! - **[`agent::Agent`]** - Orchestrates the agentic loop between Bedrock and tools
//! - **[`bedrock::BedrockClient`]** - Direct AWS Bedrock service integration
//! - **[`tools::ToolRegistry`]** - Type-safe tool execution with compile-time validation
//!
//! The agent handles conversation management, tool orchestration, and error recovery
//! while maintaining full compatibility with Claude's reasoning capabilities and AWS
//! Bedrock's enterprise features.
//!
//! # Key Features
//!
//! - **AWS Bedrock Native** - Optimized for Claude 3/4 and Nova models with streaming
//! - **Type-Safe Tools** - Compile-time validation prevents runtime tool errors
//! - **Agentic Execution** - Multi-step reasoning with automatic tool orchestration
//! - **Production Ready** - Comprehensive error handling, retries, and observability
//! - **MCP Protocol** - Connect to external tools via Model Context Protocol
//! - **Performance Optimized** - Parallel execution, connection pooling, memory efficiency
//!
//! # Performance Characteristics
//!
//! - Processes tool calls in parallel when possible
//! - Maintains connection pools for optimal AWS Bedrock latency
//! - Zero-copy message handling where feasible
//! - Streaming responses for real-time user experience
//! - Memory-efficient conversation management
//!
//! # Feature Flags
//!
//! - `telemetry` - OpenTelemetry integration for observability
//! - `http` - HTTP health endpoints for service monitoring
//! - `examples` - Additional example code and development utilities
//!
//! # Module Organization
//!
//! - [`agent`] - Core agent implementation and conversation management
//! - [`bedrock`] - AWS Bedrock client with retry logic and error handling
//! - [`tools`] - Tool system with macro-based registration and execution
//! - [`types`] - Shared data structures for messages, models, and configurations
//! - [`mcp`] - Model Context Protocol client and server implementations
//! - [`error`] - Comprehensive error types and recovery strategies
//! - [`performance`] - Optimization utilities and metrics collection
//! - [`telemetry`] - Logging and observability integration

pub mod agent;
// Bedrock functionality now in llm::providers::bedrock
pub mod config;
pub mod context_manager;
pub mod conversation_manager;
pub mod error;
pub mod error_recovery;
pub mod health;
pub mod llm;
pub mod mcp;
pub mod message_processor;
pub mod parallel;
pub mod performance;
pub mod shutdown;
pub mod streaming;
pub mod telemetry;
pub mod tools;
pub mod types;
pub mod utils;

pub use error::StoodError;
pub use types::*;

pub use stood_macros::tool;

pub type Result<T> = std::result::Result<T, StoodError>;

#[cfg(feature = "verification")]
pub mod verification;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
