//! Candle provider implementation.
//!
//! This provider uses the Candle ML framework for running models locally
//! with hardware acceleration (CPU, CUDA, Metal).
//!
//! **Status: NOT YET IMPLEMENTED** - See README.md "TODO - Work in Progress" section
//! This is a placeholder implementation that returns appropriate errors.
//! Future implementation will support local Rust-based inference via Candle.

use crate::llm::traits::{
    ChatConfig, ChatResponse, HealthStatus, LlmError, LlmProvider, ProviderCapabilities,
    ProviderType, StreamEvent, Tool,
};
use crate::types::Messages;
use async_trait::async_trait;
use futures::Stream;

/// Candle provider - NOT YET IMPLEMENTED
///
/// This provider uses Candle for local model inference with hardware acceleration.
/// See README.md "🚧 Planned Providers (Not Yet Implemented)" section.
#[derive(Debug)]
#[allow(dead_code)] // Planned for future implementation
pub struct CandleProvider {
    #[allow(dead_code)] // Planned for future implementation
    cache_dir: Option<String>,
    #[allow(dead_code)] // Planned for future implementation
    device: String, // "cpu", "cuda", "metal"
}

impl CandleProvider {
    /// Create a new Candle provider
    pub async fn new(cache_dir: Option<String>, device: Option<String>) -> Result<Self, LlmError> {
        let device = device.unwrap_or_else(|| {
            // Auto-detect best available device
            #[cfg(feature = "cuda")]
            if candle_core::Device::cuda_if_available(0).is_ok() {
                return "cuda".to_string();
            }

            #[cfg(feature = "metal")]
            if candle_core::Device::metal_if_available().is_ok() {
                return "metal".to_string();
            }

            "cpu".to_string()
        });

        Ok(Self { cache_dir, device })
    }
}

#[async_trait]
impl LlmProvider for CandleProvider {
    async fn chat(
        &self,
        _model_id: &str,
        _messages: &Messages,
        _config: &ChatConfig,
    ) -> Result<ChatResponse, LlmError> {
        Err(LlmError::UnsupportedFeature {
            feature: "Candle provider not yet implemented".to_string(),
            provider: ProviderType::Candle,
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
            feature: "Candle tool calling not yet implemented".to_string(),
            provider: ProviderType::Candle,
        })
    }

    async fn chat_streaming(
        &self,
        _model_id: &str,
        _messages: &Messages,
        _config: &ChatConfig,
    ) -> Result<Box<dyn Stream<Item = StreamEvent> + Send + Unpin>, LlmError> {
        Err(LlmError::UnsupportedFeature {
            feature: "Candle streaming not yet implemented".to_string(),
            provider: ProviderType::Candle,
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
            feature: "Candle streaming with tools not yet implemented".to_string(),
            provider: ProviderType::Candle,
        })
    }

    async fn health_check(&self) -> Result<HealthStatus, LlmError> {
        Ok(HealthStatus {
            healthy: false,
            provider: ProviderType::Candle,
            latency_ms: None,
            error: Some("Candle provider not yet implemented".to_string()),
        })
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_streaming: false, // Will be true when implemented
            supports_tools: false,     // Will be true when implemented
            supports_thinking: false,
            supports_vision: false,
            max_tokens: None, // Varies by model
            available_models: vec![
                "microsoft/DialoGPT-medium".to_string(),
                "microsoft/DialoGPT-large".to_string(),
                "distilbert-base-uncased".to_string(),
            ],
        }
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Candle
    }

    fn supported_models(&self) -> Vec<&'static str> {
        vec![
            "microsoft/DialoGPT-medium",
            "microsoft/DialoGPT-large",
            "distilbert-base-uncased",
        ]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
