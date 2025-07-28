//! OpenRouter provider implementation.
//!
//! This provider connects to OpenRouter's API which provides access to
//! multiple models from various providers through a unified interface.
//! 
//! **Status: NOT YET IMPLEMENTED** - See README.md "TODO - Work in Progress" section
//! This is a placeholder implementation that returns appropriate errors.
//! Future implementation will support multi-provider proxy access via OpenRouter.

use crate::llm::traits::{LlmProvider, ProviderType, LlmError, ChatResponse, ChatConfig, Tool, StreamEvent, ProviderCapabilities, HealthStatus};
use crate::types::Messages;
use async_trait::async_trait;
use futures::Stream;

/// OpenRouter provider - NOT YET IMPLEMENTED
/// 
/// This provider connects to OpenRouter's API for access to multiple models.
/// See README.md "🚧 Planned Providers (Not Yet Implemented)" section.
#[derive(Debug)]
#[allow(dead_code)] // Planned for future implementation
pub struct OpenRouterProvider {
    #[allow(dead_code)] // Planned for future implementation
    api_key: String,
    #[allow(dead_code)] // Planned for future implementation
    base_url: String,
    #[allow(dead_code)] // Planned for future implementation
    client: reqwest::Client,
}

impl OpenRouterProvider {
    /// Create a new OpenRouter provider
    pub async fn new(api_key: String, base_url: Option<String>) -> Result<Self, LlmError> {
        let client = reqwest::Client::new();
        let base_url = base_url.unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string());
        
        Ok(Self {
            api_key,
            base_url,
            client,
        })
    }
}

#[async_trait]
impl LlmProvider for OpenRouterProvider {
    async fn chat(
        &self,
        _model_id: &str,
        _messages: &Messages,
        _config: &ChatConfig,
    ) -> Result<ChatResponse, LlmError> {
        Err(LlmError::UnsupportedFeature {
            feature: "OpenRouter provider not yet implemented".to_string(),
            provider: ProviderType::OpenRouter,
        })
    }
    
    async fn chat_with_tools(
        &self,
        _model_id: &str,
        _messages: &Messages,
        _tools: &[Tool],
        _config: &ChatConfig,
    ) -> Result<ChatResponse, LlmError> {
        Err(LlmError::UnsupportedFeature {
            feature: "OpenRouter tool calling not yet implemented".to_string(),
            provider: ProviderType::OpenRouter,
        })
    }
    
    async fn chat_streaming(
        &self,
        _model_id: &str,
        _messages: &Messages,
        _config: &ChatConfig,
    ) -> Result<Box<dyn Stream<Item = StreamEvent> + Send + Unpin>, LlmError> {
        Err(LlmError::UnsupportedFeature {
            feature: "OpenRouter streaming not yet implemented".to_string(),
            provider: ProviderType::OpenRouter,
        })
    }
    
    async fn chat_streaming_with_tools(
        &self,
        _model_id: &str,
        _messages: &Messages,
        _tools: &[Tool],
        _config: &ChatConfig,
    ) -> Result<Box<dyn Stream<Item = StreamEvent> + Send + Unpin>, LlmError> {
        Err(LlmError::UnsupportedFeature {
            feature: "OpenRouter streaming with tools not yet implemented".to_string(),
            provider: ProviderType::OpenRouter,
        })
    }
    
    async fn health_check(&self) -> Result<HealthStatus, LlmError> {
        Ok(HealthStatus {
            healthy: false,
            provider: ProviderType::OpenRouter,
            latency_ms: None,
            error: Some("OpenRouter provider not yet implemented".to_string()),
        })
    }
    
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_streaming: false, // Will be true when implemented
            supports_tools: false, // Will be true when implemented
            supports_thinking: false,
            supports_vision: false,
            max_tokens: None, // Varies by model
            available_models: vec![
                "anthropic/claude-3.5-sonnet".to_string(),
                "openai/gpt-4o".to_string(),
                "meta-llama/llama-3.1-405b-instruct".to_string(),
                "google/gemini-pro-1.5".to_string(),
            ],
        }
    }
    
    fn provider_type(&self) -> ProviderType {
        ProviderType::OpenRouter
    }
    
    fn supported_models(&self) -> Vec<&'static str> {
        vec![
            "anthropic/claude-3.5-sonnet",
            "openai/gpt-4o",
            "meta-llama/llama-3.1-405b-instruct",
            "google/gemini-pro-1.5",
        ]
    }
}