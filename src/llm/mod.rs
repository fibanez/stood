//! Multi-provider LLM integration with unified interface and enterprise-grade reliability.
//!
//! This module provides a comprehensive abstraction layer for interacting with multiple LLM providers
//! including AWS Bedrock, LM Studio, Anthropic, OpenAI, Ollama, OpenRouter, and Candle. You'll get
//! consistent APIs, automatic provider selection, streaming support, and robust error handling
//! across all supported providers.
//!
//! # Quick Start
//!
//! Use different providers with the same agent API:
//!
//! ```no_run
//! use stood::agent::Agent;
//! use stood::llm::models::{Bedrock, LMStudio};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Use AWS Bedrock (cloud)
//!     let mut bedrock_agent = Agent::builder()
//!         .model(Bedrock::Claude35Haiku)
//!         .build().await?;
//!     
//!     // Use LM Studio (local)
//!     let mut local_agent = Agent::builder()
//!         .model(LMStudio::Gemma3_12B)
//!         .build().await?;
//!     
//!     // Same API for both providers
//!     let result1 = bedrock_agent.execute("Hello from cloud").await?;
//!     let result2 = local_agent.execute("Hello from local").await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Architecture
//!
//! ```text
//! Agent → ProviderRegistry → LlmProvider (Bedrock | LMStudio | Anthropic | OpenAI)
//!   ↓           ↓                  ↓
//! Model    LlmModel enum     Provider-specific API calls
//! Config   (Claude35Haiku,   (aws-sdk-bedrockruntime, reqwest, etc.)
//!          Gemma3_12B, etc.)
//! ```
//!
//! # Key Features
//!
//! - **Unified Interface** - Same API across all providers via `LlmProvider` trait
//! - **Type-Safe Models** - `LlmModel` enum prevents invalid model/provider combinations
//! - **Automatic Configuration** - `ProviderRegistry` handles provider setup and discovery
//! - **Streaming Support** - Real-time response streaming for all providers
//! - **Tool Integration** - Native tool calling support where providers support it
//! - **Error Handling** - Comprehensive error types with provider-specific context
//! - **Performance Optimization** - Connection pooling and request batching
//!
//! # Supported Providers
//!
//! **AWS Bedrock:**
//! - Claude 3.5 Haiku/Sonnet/Opus
//! - Amazon Nova Pro/Lite/Micro
//! - Streaming and tool support
//!
//! **LM Studio:**
//! - Gemma 3 12B/27B
//! - Llama 3 70B
//! - Mistral 7B
//! - Local model hosting
//!
//! **Coming Soon:**
//! - Anthropic Direct API
//! - OpenAI GPT-4/3.5
//! - Ollama local models
//!
//! # Key Types
//!
//! - [`LlmProvider`] - Core trait for provider implementations
//! - [`LlmModel`] - Type-safe model selection enum
//! - [`ProviderRegistry`] - Central registry for provider configuration
//! - [`ChatConfig`] - Provider-agnostic chat configuration
//! - [`ChatResponse`] - Unified response format across providers

pub mod traits;
pub mod client;
pub mod error;
pub mod config;
pub mod streaming;
pub mod models;
pub mod providers;
pub mod registry;

#[cfg(test)]
pub mod integration_test;

#[cfg(test)]
pub mod tests;

// Re-export core types for convenience
pub use traits::{
    LlmProvider, LlmModel, ProviderType, ChatConfig, ChatResponse, 
    ToolCall, Tool, StreamEvent, Usage, HealthStatus, 
    ProviderCapabilities, ModelCapabilities, LlmError
};

// Re-export model provider modules for the single API pattern
pub use models::{Bedrock, LMStudio};

// Re-export registry for configuration
pub use registry::{ProviderRegistry, ProviderConfig, PROVIDER_REGISTRY};