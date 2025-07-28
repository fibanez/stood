//! Core traits for the LLM abstraction layer.
//!
//! This module defines the fundamental traits that enable a unified interface
//! across multiple LLM providers while maintaining type safety and performance.

use crate::types::Messages;
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core LLM provider trait that abstracts away provider-specific implementations
/// 
/// Providers own ALL implementation details including request formatting, response parsing,
/// streaming, error handling, and authentication. Models are passed as model_id strings.
#[async_trait]
pub trait LlmProvider: Send + Sync + std::fmt::Debug {
    /// Basic chat without tools
    async fn chat(
        &self, 
        model_id: &str, 
        messages: &Messages, 
        config: &ChatConfig
    ) -> Result<ChatResponse, LlmError>;
    
    /// Chat with tools
    async fn chat_with_tools(
        &self,
        model_id: &str,
        messages: &Messages, 
        tools: &[Tool], 
        config: &ChatConfig
    ) -> Result<ChatResponse, LlmError>;
    
    /// Streaming chat
    async fn chat_streaming(
        &self,
        model_id: &str,
        messages: &Messages, 
        config: &ChatConfig
    ) -> Result<Box<dyn Stream<Item = StreamEvent> + Send + Unpin>, LlmError>;
    
    /// Streaming chat with tools
    async fn chat_streaming_with_tools(
        &self,
        model_id: &str,
        messages: &Messages,
        tools: &[Tool],
        config: &ChatConfig
    ) -> Result<Box<dyn Stream<Item = StreamEvent> + Send + Unpin>, LlmError>;
    
    /// Health check
    async fn health_check(&self) -> Result<HealthStatus, LlmError>;
    
    /// Provider-specific capabilities
    fn capabilities(&self) -> ProviderCapabilities;
    
    /// Get provider type
    fn provider_type(&self) -> ProviderType;
    
    /// List of model IDs supported by this provider
    fn supported_models(&self) -> Vec<&'static str>;
}

/// Model abstraction - pure metadata only, no logic
/// 
/// Models are lightweight structs that only contain metadata about the model.
/// ALL formatting, parsing, and request handling logic belongs in the Provider.
/// This ensures clean separation of concerns in the provider-first architecture.
pub trait LlmModel: Send + Sync {
    /// Unique model identifier used by the provider
    fn model_id(&self) -> &'static str;
    
    /// Provider that hosts this model
    fn provider(&self) -> ProviderType;
    
    /// Maximum context window in tokens
    fn context_window(&self) -> usize;
    
    /// Maximum output tokens this model can generate
    fn max_output_tokens(&self) -> usize;
    
    /// Model capabilities (what features it supports)
    fn capabilities(&self) -> ModelCapabilities;
    
    /// Human-readable display name for the model (defaults to model_id)
    fn display_name(&self) -> &'static str {
        self.model_id()
    }
    
    /// Default temperature for this model
    fn default_temperature(&self) -> f32 {
        0.7
    }
    
    /// Default max tokens for this model
    fn default_max_tokens(&self) -> u32 {
        self.max_output_tokens() as u32
    }
    
    /// Check if this model supports tool use
    fn supports_tool_use(&self) -> bool {
        self.capabilities().supports_tools
    }
    
    /// Check if this model supports streaming
    fn supports_streaming(&self) -> bool {
        self.capabilities().supports_streaming
    }
    
    /// Check if this model supports thinking mode
    fn supports_thinking(&self) -> bool {
        self.capabilities().supports_thinking
    }
}

/// LLM provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderType {
    /// AWS Bedrock (existing)
    Bedrock,
    /// LM Studio (new)
    LmStudio,
    /// Anthropic Direct API (new)
    Anthropic,
    /// OpenAI API (new)
    OpenAI,
    /// Ollama (new)
    Ollama,
    /// OpenRouter (new)
    OpenRouter,
    /// Candle (new)
    Candle,
}

impl ProviderType {
    /// Get string representation of provider type
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderType::Bedrock => "bedrock",
            ProviderType::LmStudio => "lm_studio",
            ProviderType::Anthropic => "anthropic",
            ProviderType::OpenAI => "openai",
            ProviderType::Ollama => "ollama",
            ProviderType::OpenRouter => "openrouter",
            ProviderType::Candle => "candle",
        }
    }
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Internal chat configuration for LLM requests (not user-facing)
/// 
/// This is used internally to translate from the user-facing `AgentConfig` 
/// to provider-specific request parameters. Users should continue using
/// `AgentConfig` and `Agent::Builder` for configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    /// Model to use for the chat (provider-specific model ID)
    pub model_id: String,
    /// Provider type for this request
    pub provider: ProviderType,
    /// Temperature for model responses (0.0 to 1.0)
    pub temperature: Option<f32>,
    /// Maximum tokens in model response
    pub max_tokens: Option<u32>,
    /// Whether to enable thinking mode (if supported)
    pub enable_thinking: bool,
    /// Additional model-specific parameters
    #[serde(default)]
    pub additional_params: HashMap<String, serde_json::Value>,
}

impl ChatConfig {
    /// Create ChatConfig from AgentConfig
    pub fn from_agent_config(agent_config: &crate::types::AgentConfig) -> Self {
        Self {
            model_id: agent_config.model_id.clone(),
            provider: agent_config.provider,
            temperature: agent_config.temperature,
            max_tokens: agent_config.max_tokens,
            enable_thinking: agent_config.enable_thinking,
            additional_params: agent_config.additional_params.clone(),
        }
    }
}

impl Default for ChatConfig {
    fn default() -> Self {
        // Default to Claude Haiku 3.5 via Bedrock
        Self::from_agent_config(&crate::types::AgentConfig::default())
    }
}

/// Response from LLM chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Response content
    pub content: String,
    /// Tool calls if any
    pub tool_calls: Vec<ToolCall>,
    /// Thinking content if enabled
    pub thinking: Option<String>,
    /// Usage statistics
    pub usage: Option<Usage>,
    /// Provider-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call
    pub id: String,
    /// Name of the tool to call
    pub name: String,
    /// Input parameters for the tool
    pub input: serde_json::Value,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema
    pub input_schema: serde_json::Value,
}

/// Content block type for universal streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentBlockType {
    Text,
    ToolUse,
    Thinking,
}

/// Content block delta for universal streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentBlockDelta {
    Text { 
        text: String 
    },
    ToolUse { 
        tool_call_id: String, 
        input_delta: String 
    },
    Thinking { 
        reasoning_delta: String 
    },
}

/// Universal streaming events that work across all providers
/// Based on the proven content block pattern from master branch and Python reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEvent {
    /// Content block starts (universal pattern)
    ContentBlockStart { 
        block_type: ContentBlockType,
        block_index: usize,
    },
    /// Content block delta (universal pattern)
    ContentBlockDelta { 
        delta: ContentBlockDelta,
        block_index: usize,
    },
    /// Content block stops (universal pattern)
    ContentBlockStop { 
        block_index: usize,
    },
    /// Message starts
    MessageStart {
        role: crate::types::MessageRole,
    },
    /// Message stops
    MessageStop {
        stop_reason: Option<String>,
    },
    /// Stream metadata (usage, etc.)
    Metadata {
        usage: Option<Usage>,
    },
    /// Error in stream
    Error {
        error: String,
    },
    
    // Legacy events for backward compatibility - will be deprecated
    /// @deprecated Use ContentBlockDelta with Text variant
    ContentDelta { 
        delta: String,
        index: usize,
    },
    /// @deprecated Use ContentBlockStart with ToolUse type
    ToolCallStart {
        tool_call: ToolCall,
    },
    /// @deprecated Use ContentBlockDelta with ToolUse variant
    ToolCallDelta {
        tool_call_id: String,
        delta: String,
    },
    /// @deprecated Use ContentBlockDelta with Thinking variant
    ThinkingDelta {
        delta: String,
    },
    /// @deprecated Use MessageStop or Metadata
    Done {
        usage: Option<Usage>,
    },
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub provider: ProviderType,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

/// Provider capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_thinking: bool,
    pub supports_vision: bool,
    pub max_tokens: Option<u32>,
    pub available_models: Vec<String>,
}

/// Model capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub max_tokens: Option<u32>,
    pub supports_tools: bool,
    pub supports_streaming: bool,
    pub supports_thinking: bool,
    pub supports_vision: bool,
    pub context_window: Option<u32>,
}

/// LLM-specific error types
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("Provider error: {message}")]
    ProviderError { 
        provider: ProviderType,
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("Model not found: {model_id}")]
    ModelNotFound { 
        model_id: String,
        provider: ProviderType,
    },
    
    #[error("Authentication failed for provider {provider:?}")]
    AuthenticationError { 
        provider: ProviderType,
    },
    
    #[error("Rate limit exceeded for provider {provider:?}")]
    RateLimitError { 
        provider: ProviderType,
        retry_after: Option<u64>,
    },
    
    #[error("Configuration error: {message}")]
    ConfigurationError { 
        message: String,
    },
    
    #[error("Network error: {message}")]
    NetworkError { 
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("Serialization error: {message}")]
    SerializationError { 
        message: String,
    },
    
    #[error("Unsupported feature: {feature} for provider {provider:?}")]
    UnsupportedFeature { 
        feature: String,
        provider: ProviderType,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // Failing tests to be implemented as we build the architecture
    
    #[tokio::test]
    async fn test_llm_provider_trait_basic_chat() {
        // This test will fail until we implement a test provider
        panic!("LlmProvider trait basic chat not implemented yet");
    }
    
    #[tokio::test]
    async fn test_llm_provider_trait_streaming() {
        // This test will fail until we implement streaming
        panic!("LlmProvider trait streaming not implemented yet");
    }
    
    #[tokio::test]
    async fn test_llm_model_trait_capabilities() {
        // This test will fail until we implement model trait
        panic!("LlmModel trait capabilities not implemented yet");
    }
    
    #[test]
    fn test_stream_event_serialization() {
        // This test will fail until we implement proper serialization
        panic!("StreamEvent serialization not implemented yet");
    }
    
    #[test]
    fn test_error_type_conversion() {
        // This test will fail until we implement error conversion
        panic!("Error type conversion not implemented yet");
    }
}