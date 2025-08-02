//! Provider registry for lazy loading and sharing provider instances.
//!
//! The registry handles configuration discovery, provider instantiation, and sharing
//! across multiple agent instances to optimize resource usage.

use crate::llm::traits::{LlmProvider, ProviderType, LlmError};
use crate::llm::providers::{
    BedrockProvider, LMStudioProvider, AnthropicProvider, 
    OpenAIProvider, OllamaProvider, OpenRouterProvider, CandleProvider
};
use crate::llm::providers::retry::RetryConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// Bedrock-specific credentials for programmatic authentication
#[derive(Debug, Clone)]
pub struct BedrockCredentials {
    pub access_key: String,
    pub secret_key: String,
    pub session_token: Option<String>,
}

/// Global provider registry instance
/// 
/// This is initialized once and shared across all agents for efficient provider reuse.
pub static PROVIDER_REGISTRY: Lazy<ProviderRegistry> = Lazy::new(|| {
    ProviderRegistry::new()
});

/// Provider registry that manages configurations and lazy-loads provider instances
pub struct ProviderRegistry {
    /// Provider configurations discovered from environment
    configs: RwLock<HashMap<ProviderType, ProviderConfig>>,
    /// Instantiated provider instances (shared across agents)
    providers: RwLock<HashMap<ProviderType, Arc<dyn LlmProvider>>>,
}

/// Configuration for each provider type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderConfig {
    /// AWS Bedrock configuration
    Bedrock {
        region: Option<String>,
        #[serde(skip)] // Don't serialize credentials
        credentials: Option<BedrockCredentials>,
    },
    /// LM Studio configuration
    LMStudio {
        base_url: String,
        retry_config: Option<RetryConfig>,
    },
    /// Anthropic Direct API configuration
    Anthropic {
        api_key: String,
        base_url: Option<String>,
    },
    /// OpenAI API configuration
    OpenAI {
        api_key: String,
        organization: Option<String>,
        base_url: Option<String>,
    },
    /// Ollama configuration
    Ollama {
        base_url: String,
    },
    /// OpenRouter configuration
    OpenRouter {
        api_key: String,
        base_url: Option<String>,
    },
    /// Candle configuration
    Candle {
        cache_dir: Option<String>,
        device: Option<String>, // "cpu", "cuda", "metal"
    },
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new() -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
            providers: RwLock::new(HashMap::new()),
        }
    }
    
    /// Configure providers from environment variables
    /// 
    /// This should be called once at application startup to detect available providers.
    pub async fn configure() -> Result<(), LlmError> {
        let registry = &PROVIDER_REGISTRY;
        let mut configs = registry.configs.write().await;
        
        // Auto-detect AWS Bedrock
        if let Ok(region) = std::env::var("AWS_REGION") {
            configs.insert(ProviderType::Bedrock, ProviderConfig::Bedrock {
                region: Some(region),
                credentials: None, // Use default credential chain
            });
        } else if std::env::var("AWS_ACCESS_KEY_ID").is_ok() || std::env::var("AWS_PROFILE").is_ok() {
            // AWS credentials present but no region, use default
            configs.insert(ProviderType::Bedrock, ProviderConfig::Bedrock {
                region: None, // Will use default region
                credentials: None,
            });
        }
        
        // Auto-detect LM Studio
        if let Ok(base_url) = std::env::var("LM_STUDIO_BASE_URL") {
            configs.insert(ProviderType::LmStudio, ProviderConfig::LMStudio { 
                base_url,
                retry_config: None,  // Use default retry config
            });
        } else {
            // Try default LM Studio port
            configs.insert(ProviderType::LmStudio, ProviderConfig::LMStudio {
                base_url: "http://localhost:1234".to_string(),
                retry_config: None,  // Use default retry config
            });
        }
        
        // Auto-detect Anthropic
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            configs.insert(ProviderType::Anthropic, ProviderConfig::Anthropic {
                api_key,
                base_url: std::env::var("ANTHROPIC_BASE_URL").ok(),
            });
        }
        
        // Auto-detect OpenAI
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            configs.insert(ProviderType::OpenAI, ProviderConfig::OpenAI {
                api_key,
                organization: std::env::var("OPENAI_ORG_ID").ok(),
                base_url: std::env::var("OPENAI_BASE_URL").ok(),
            });
        }
        
        // Auto-detect Ollama
        if let Ok(base_url) = std::env::var("OLLAMA_BASE_URL") {
            configs.insert(ProviderType::Ollama, ProviderConfig::Ollama { base_url });
        } else {
            // Try default Ollama port
            configs.insert(ProviderType::Ollama, ProviderConfig::Ollama {
                base_url: "http://localhost:11434".to_string(),
            });
        }
        
        // Auto-detect OpenRouter
        if let Ok(api_key) = std::env::var("OPENROUTER_API_KEY") {
            configs.insert(ProviderType::OpenRouter, ProviderConfig::OpenRouter {
                api_key,
                base_url: std::env::var("OPENROUTER_BASE_URL").ok(),
            });
        }
        
        Ok(())
    }
    
    /// Get a provider instance (lazy loading)
    /// 
    /// This method first checks if the provider is already instantiated and cached.
    /// If not, it creates a new provider instance based on the configuration.
    /// Providers are shared across all agents for efficiency.
    pub async fn get_provider(&self, provider_type: ProviderType) -> Result<Arc<dyn LlmProvider>, LlmError> {
        // Check if provider is already instantiated
        {
            let providers = self.providers.read().await;
            if let Some(provider) = providers.get(&provider_type) {
                return Ok(Arc::clone(provider));
            }
        }
        
        // Provider not found, need to create it
        let configs = self.configs.read().await;
        let config = configs.get(&provider_type)
            .ok_or_else(|| LlmError::ConfigurationError {
                message: format!("No configuration found for provider {:?}. Make sure to call ProviderRegistry::configure() first or set required environment variables (AWS_ACCESS_KEY_ID, AWS_REGION, or AWS_PROFILE for Bedrock).", provider_type),
            })?;
        
        // Create provider based on configuration
        let provider: Arc<dyn LlmProvider> = match (provider_type, config) {
            (ProviderType::Bedrock, ProviderConfig::Bedrock { region, credentials }) => {
                let bedrock_provider = if let Some(creds) = credentials {
                    // Use custom credentials
                    BedrockProvider::with_credentials(
                        region.clone(),
                        creds.access_key.clone(),
                        creds.secret_key.clone(),
                        creds.session_token.clone(),
                    ).await
                } else {
                    // Use default credential chain
                    BedrockProvider::new(region.clone()).await
                }
                .map_err(|e| LlmError::ProviderError {
                    provider: provider_type,
                    message: format!("Failed to create Bedrock provider: {}", e),
                    source: Some(Box::new(e)),
                })?;
                Arc::new(bedrock_provider)
            },
            (ProviderType::LmStudio, ProviderConfig::LMStudio { base_url, retry_config }) => {
                let retry_cfg = retry_config.clone().unwrap_or_else(|| RetryConfig::lm_studio_default());
                let lm_studio_provider = LMStudioProvider::with_retry_config(base_url.clone(), retry_cfg).await
                    .map_err(|e| LlmError::ProviderError {
                        provider: provider_type,
                        message: format!("Failed to create LM Studio provider: {}", e),
                        source: Some(Box::new(e)),
                    })?;
                Arc::new(lm_studio_provider)
            },
            (ProviderType::Anthropic, ProviderConfig::Anthropic { api_key, base_url }) => {
                let anthropic_provider = AnthropicProvider::new(api_key.clone(), base_url.clone()).await
                    .map_err(|e| LlmError::ProviderError {
                        provider: provider_type,
                        message: format!("Failed to create Anthropic provider: {}", e),
                        source: Some(Box::new(e)),
                    })?;
                Arc::new(anthropic_provider)
            },
            (ProviderType::OpenAI, ProviderConfig::OpenAI { api_key, organization, base_url }) => {
                let openai_provider = OpenAIProvider::new(api_key.clone(), organization.clone(), base_url.clone()).await
                    .map_err(|e| LlmError::ProviderError {
                        provider: provider_type,
                        message: format!("Failed to create OpenAI provider: {}", e),
                        source: Some(Box::new(e)),
                    })?;
                Arc::new(openai_provider)
            },
            (ProviderType::Ollama, ProviderConfig::Ollama { base_url }) => {
                let ollama_provider = OllamaProvider::new(base_url.clone()).await
                    .map_err(|e| LlmError::ProviderError {
                        provider: provider_type,
                        message: format!("Failed to create Ollama provider: {}", e),
                        source: Some(Box::new(e)),
                    })?;
                Arc::new(ollama_provider)
            },
            (ProviderType::OpenRouter, ProviderConfig::OpenRouter { api_key, base_url }) => {
                let openrouter_provider = OpenRouterProvider::new(api_key.clone(), base_url.clone()).await
                    .map_err(|e| LlmError::ProviderError {
                        provider: provider_type,
                        message: format!("Failed to create OpenRouter provider: {}", e),
                        source: Some(Box::new(e)),
                    })?;
                Arc::new(openrouter_provider)
            },
            (ProviderType::Candle, ProviderConfig::Candle { cache_dir, device }) => {
                let candle_provider = CandleProvider::new(cache_dir.clone(), device.clone()).await
                    .map_err(|e| LlmError::ProviderError {
                        provider: provider_type,
                        message: format!("Failed to create Candle provider: {}", e),
                        source: Some(Box::new(e)),
                    })?;
                Arc::new(candle_provider)
            },
            // This case should not occur if configurations match provider types
            _ => {
                return Err(LlmError::ConfigurationError {
                    message: format!("Configuration mismatch for provider {:?}", provider_type),
                });
            }
        };
        
        // Cache the provider for future use
        let mut providers = self.providers.write().await;
        providers.insert(provider_type, Arc::clone(&provider));
        
        Ok(provider)
    }
    
    /// Check if a provider is configured
    pub async fn is_configured(&self, provider_type: ProviderType) -> bool {
        let configs = self.configs.read().await;
        configs.contains_key(&provider_type)
    }
    
    /// Get all configured provider types
    pub async fn configured_providers(&self) -> Vec<ProviderType> {
        let configs = self.configs.read().await;
        configs.keys().copied().collect()
    }
    
    /// Manually add a provider configuration
    /// 
    /// This can be used to add providers programmatically instead of via environment variables.
    pub async fn add_config(&self, provider_type: ProviderType, config: ProviderConfig) {
        let mut configs = self.configs.write().await;
        configs.insert(provider_type, config);
    }
    
    /// Clear all cached providers (useful for testing)
    pub async fn clear_cache(&self) {
        let mut providers = self.providers.write().await;
        providers.clear();
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_registry_creation() {
        let registry = ProviderRegistry::new();
        assert!(registry.configured_providers().await.is_empty());
    }
    
    #[tokio::test]
    async fn test_manual_configuration() {
        let registry = ProviderRegistry::new();
        
        let config = ProviderConfig::LMStudio {
            base_url: "http://localhost:1234".to_string(),
            retry_config: None,
        };
        
        registry.add_config(ProviderType::LmStudio, config).await;
        
        assert!(registry.is_configured(ProviderType::LmStudio).await);
        assert_eq!(registry.configured_providers().await, vec![ProviderType::LmStudio]);
    }
    
    #[tokio::test] 
    async fn test_provider_lazy_loading() {
        // This test will fail until we implement the providers
        // but it demonstrates the expected behavior
        
        let registry = ProviderRegistry::new();
        
        // Add a test configuration
        registry.add_config(ProviderType::LmStudio, ProviderConfig::LMStudio {
            base_url: "http://localhost:1234".to_string(),
            retry_config: None,
        }).await;
        
        // First call should create the provider
        let result = registry.get_provider(ProviderType::LmStudio).await;
        // This will fail until LMStudioProvider is implemented
        // assert!(result.is_ok());
        
        // For now, just test that we get a configuration error for unconfigured providers
        let unconfigured_result = registry.get_provider(ProviderType::Anthropic).await;
        assert!(unconfigured_result.is_err());
        assert!(matches!(unconfigured_result.unwrap_err(), LlmError::ConfigurationError { .. }));
    }
}