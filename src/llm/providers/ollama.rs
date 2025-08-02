//! Ollama provider implementation.
//!
//! This provider connects to a local Ollama instance for running
//! local open-source models.
//! 
//! **Status: NOT YET IMPLEMENTED** - See README.md "TODO - Work in Progress" section
//! This is a placeholder implementation that returns appropriate errors.
//! Future implementation will support local LLM hosting via Ollama.

use crate::llm::traits::{LlmProvider, ProviderType, LlmError, ChatResponse, ChatConfig, Tool, StreamEvent, ProviderCapabilities, HealthStatus};
use crate::types::Messages;
use async_trait::async_trait;
use futures::Stream;

/// Ollama provider - NOT YET IMPLEMENTED
/// 
/// This provider connects to a local Ollama instance for running local models.
/// See README.md "🚧 Planned Providers (Not Yet Implemented)" section.
#[derive(Debug)]
#[allow(dead_code)] // Planned for future implementation
pub struct OllamaProvider {
    #[allow(dead_code)] // Planned for future implementation
    base_url: String,
    #[allow(dead_code)] // Planned for future implementation
    client: reqwest::Client,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    pub async fn new(base_url: String) -> Result<Self, LlmError> {
        let client = reqwest::Client::new();
        
        Ok(Self {
            base_url,
            client,
        })
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn chat(
        &self,
        _model_id: &str,
        _messages: &Messages,
        _config: &ChatConfig,
    ) -> Result<ChatResponse, LlmError> {
        Err(LlmError::UnsupportedFeature {
            feature: "Ollama provider not yet implemented".to_string(),
            provider: ProviderType::Ollama,
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
            feature: "Ollama tool calling not yet implemented".to_string(),
            provider: ProviderType::Ollama,
        })
    }
    
    async fn chat_streaming(
        &self,
        _model_id: &str,
        _messages: &Messages,
        _config: &ChatConfig,
    ) -> Result<Box<dyn Stream<Item = StreamEvent> + Send + Unpin>, LlmError> {
        Err(LlmError::UnsupportedFeature {
            feature: "Ollama streaming not yet implemented".to_string(),
            provider: ProviderType::Ollama,
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
            feature: "Ollama streaming with tools not yet implemented".to_string(),
            provider: ProviderType::Ollama,
        })
    }
    
    async fn health_check(&self) -> Result<HealthStatus, LlmError> {
        Ok(HealthStatus {
            healthy: false,
            provider: ProviderType::Ollama,
            latency_ms: None,
            error: Some("Ollama provider not yet implemented".to_string()),
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
                "llama3.2".to_string(),
                "llama3.1".to_string(),
                "mistral".to_string(),
                "codellama".to_string(),
            ],
        }
    }
    
    fn provider_type(&self) -> ProviderType {
        ProviderType::Ollama
    }
    
    fn supported_models(&self) -> Vec<&'static str> {
        vec![
            "llama3.2",
            "llama3.1", 
            "mistral",
            "codellama",
        ]
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}