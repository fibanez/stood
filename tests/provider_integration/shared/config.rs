//! Configuration management for verification tests
//!
//! This module provides utilities for managing test configurations,
//! environment detection, and provider-specific settings.

use super::*;
use std::env;
use std::time::Duration;

/// Environment variable names for test configuration
pub const ENV_LM_STUDIO_BASE_URL: &str = "LM_STUDIO_BASE_URL";
pub const ENV_VERIFICATION_MODEL: &str = "VERIFICATION_MODEL";
pub const ENV_VERIFICATION_TIMEOUT_SECONDS: &str = "VERIFICATION_TIMEOUT_SECONDS";
pub const ENV_VERIFICATION_MAX_PARALLEL_TOOLS: &str = "VERIFICATION_MAX_PARALLEL_TOOLS";
pub const ENV_ANTHROPIC_API_KEY: &str = "ANTHROPIC_API_KEY";
pub const ENV_OPENAI_API_KEY: &str = "OPENAI_API_KEY";
pub const ENV_AWS_ACCESS_KEY_ID: &str = "AWS_ACCESS_KEY_ID";
pub const ENV_AWS_SECRET_ACCESS_KEY: &str = "AWS_SECRET_ACCESS_KEY";
pub const ENV_AWS_PROFILE: &str = "AWS_PROFILE";

/// Test configuration builder with environment detection
pub struct TestConfigBuilder {
    provider: Option<ProviderType>,
    model_id: Option<String>,
    timeout: Option<Duration>,
    max_retries: u32,
    enable_telemetry: bool,
    enable_streaming: bool,
    max_parallel_tools: usize,
}

impl Default for TestConfigBuilder {
    fn default() -> Self {
        Self {
            provider: None,
            model_id: None,
            timeout: Some(Duration::from_secs(30)),
            max_retries: 3,
            enable_telemetry: true,
            enable_streaming: true,
            max_parallel_tools: 4,
        }
    }
}

impl TestConfigBuilder {
    /// Create a new test config builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the provider type
    pub fn provider(mut self, provider: ProviderType) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set the model ID
    pub fn model_id(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = Some(model_id.into());
        self
    }

    /// Set the timeout duration
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set max retries
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Enable or disable telemetry
    pub fn telemetry(mut self, enabled: bool) -> Self {
        self.enable_telemetry = enabled;
        self
    }

    /// Enable or disable streaming
    pub fn streaming(mut self, enabled: bool) -> Self {
        self.enable_streaming = enabled;
        self
    }

    /// Set max parallel tools
    pub fn max_parallel_tools(mut self, count: usize) -> Self {
        self.max_parallel_tools = count;
        self
    }

    /// Load configuration from environment variables
    pub fn from_env(mut self) -> Self {
        // Load timeout from environment
        if let Ok(timeout_str) = env::var(ENV_VERIFICATION_TIMEOUT_SECONDS) {
            if let Ok(timeout_secs) = timeout_str.parse::<u64>() {
                self.timeout = Some(Duration::from_secs(timeout_secs));
            }
        }

        // Load max parallel tools from environment
        if let Ok(parallel_str) = env::var(ENV_VERIFICATION_MAX_PARALLEL_TOOLS) {
            if let Ok(parallel_count) = parallel_str.parse::<usize>() {
                self.max_parallel_tools = parallel_count;
            }
        }

        // Load model ID from environment
        if let Ok(model_id) = env::var(ENV_VERIFICATION_MODEL) {
            self.model_id = Some(model_id);
        }

        self
    }

    /// Build the test configuration
    pub fn build(self) -> Result<TestConfig, String> {
        let provider = self.provider.ok_or("Provider must be specified")?;
        let model_id = self
            .model_id
            .unwrap_or_else(|| default_model_for_provider(provider));
        let timeout = self.timeout.unwrap_or(Duration::from_secs(30));

        let config = TestConfig {
            provider,
            model_id,
            timeout,
            max_retries: self.max_retries,
            enable_telemetry: self.enable_telemetry,
            enable_streaming: self.enable_streaming,
            max_parallel_tools: self.max_parallel_tools,
        };

        // Validate the configuration
        super::assertions::validate_test_config(&config)?;

        Ok(config)
    }
}

/// Get default model ID for a provider
fn default_model_for_provider(provider: ProviderType) -> String {
    match provider {
        ProviderType::LmStudio => "google/gemma-3-27b".to_string(),
        ProviderType::Bedrock => "us.anthropic.claude-haiku-4-5-20251001-v1:0".to_string(),
        ProviderType::Anthropic => "claude-haiku-4-5-20251001".to_string(),
        ProviderType::OpenAI => "gpt-4".to_string(),
        ProviderType::Ollama => "llama3.2".to_string(),
        _ => "unknown-model".to_string(),
    }
}

/// Create test configurations for all supported model combinations
pub fn create_integration_test_configs() -> Vec<TestConfig> {
    vec![
        // AWS Bedrock Claude Haiku 4.5
        TestConfig {
            provider: ProviderType::Bedrock,
            model_id: "us.anthropic.claude-haiku-4-5-20251001-v1:0".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            enable_telemetry: true,
            enable_streaming: true,
            max_parallel_tools: 4,
        },
        // AWS Bedrock Nova Micro
        TestConfig {
            provider: ProviderType::Bedrock,
            model_id: "us.amazon.nova-micro-v1:0".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            enable_telemetry: true,
            enable_streaming: true,
            max_parallel_tools: 4,
        },
        // LM Studio Gemma 3 27B
        TestConfig {
            provider: ProviderType::LmStudio,
            model_id: "google/gemma-3-27b".to_string(),
            timeout: Duration::from_secs(60), // Longer timeout for local model
            max_retries: 3,
            enable_telemetry: true,
            enable_streaming: true,
            max_parallel_tools: 4,
        },
        // LM Studio Tessa Rust 7B
        TestConfig {
            provider: ProviderType::LmStudio,
            model_id: "tessa-rust-t1-7b".to_string(),
            timeout: Duration::from_secs(60), // Longer timeout for local model
            max_retries: 3,
            enable_telemetry: true,
            enable_streaming: true,
            max_parallel_tools: 4,
        },
        // AWS Bedrock Mistral Large 2
        TestConfig {
            provider: ProviderType::Bedrock,
            model_id: "mistral.mistral-large-2407-v1:0".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            enable_telemetry: true,
            enable_streaming: true,
            max_parallel_tools: 4,
        },
        // AWS Bedrock Mistral Large 3
        TestConfig {
            provider: ProviderType::Bedrock,
            model_id: "mistral.mistral-large-3-675b-instruct".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            enable_telemetry: true,
            enable_streaming: true,
            max_parallel_tools: 4,
        },
    ]
}

/// Create test configuration for a specific model combination
pub fn create_config_for_model(provider: ProviderType, model_id: &str) -> TestConfig {
    let timeout = match provider {
        ProviderType::LmStudio => Duration::from_secs(60), // Local models need more time
        _ => Duration::from_secs(30),
    };

    TestConfig {
        provider,
        model_id: model_id.to_string(),
        timeout,
        max_retries: 3,
        enable_telemetry: true,
        enable_streaming: true,
        max_parallel_tools: 4,
    }
}

/// Get model-specific test configurations for token counting tests
pub fn get_token_counting_test_configs() -> Vec<TestConfig> {
    create_integration_test_configs()
}

/// Provider availability checker
pub struct ProviderChecker;

impl ProviderChecker {
    /// Check if LM Studio is available
    pub async fn check_lm_studio() -> bool {
        let base_url = env::var(ENV_LM_STUDIO_BASE_URL)
            .unwrap_or_else(|_| "http://localhost:1234".to_string());

        // Try to connect to LM Studio API
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/v1/models", base_url))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Check if AWS Bedrock credentials are available
    pub fn check_bedrock_credentials() -> bool {
        // Check for AWS credentials
        env::var(ENV_AWS_ACCESS_KEY_ID).is_ok() && env::var(ENV_AWS_SECRET_ACCESS_KEY).is_ok()
            || env::var(ENV_AWS_PROFILE).is_ok()
    }

    /// Check if Anthropic API key is available
    pub fn check_anthropic_credentials() -> bool {
        env::var(ENV_ANTHROPIC_API_KEY).is_ok()
    }

    /// Check if OpenAI API key is available
    pub fn check_openai_credentials() -> bool {
        env::var(ENV_OPENAI_API_KEY).is_ok()
    }

    /// Check if Ollama is available
    pub async fn check_ollama() -> bool {
        let client = reqwest::Client::new();
        let response = client
            .get("http://localhost:11434/api/tags")
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Get all available providers
    pub async fn get_available_providers() -> Vec<ProviderType> {
        let mut available = Vec::new();

        if Self::check_lm_studio().await {
            available.push(ProviderType::LmStudio);
        }

        if Self::check_bedrock_credentials() {
            available.push(ProviderType::Bedrock);
        }

        if Self::check_anthropic_credentials() {
            available.push(ProviderType::Anthropic);
        }

        if Self::check_openai_credentials() {
            available.push(ProviderType::OpenAI);
        }

        if Self::check_ollama().await {
            available.push(ProviderType::Ollama);
        }

        available
    }
}

/// Test environment information
#[derive(Debug, Clone)]
pub struct TestEnvironment {
    pub available_providers: Vec<ProviderType>,
    pub telemetry_endpoint: Option<String>,
    pub test_timeout: Duration,
    pub max_parallel_tools: usize,
}

impl TestEnvironment {
    /// Detect current test environment
    pub async fn detect() -> Self {
        let available_providers = ProviderChecker::get_available_providers().await;

        let telemetry_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();

        let test_timeout = env::var(ENV_VERIFICATION_TIMEOUT_SECONDS)
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(30));

        let max_parallel_tools = env::var(ENV_VERIFICATION_MAX_PARALLEL_TOOLS)
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(4);

        Self {
            available_providers,
            telemetry_endpoint,
            test_timeout,
            max_parallel_tools,
        }
    }

    /// Print environment summary
    pub fn print_summary(&self) {
        println!("🔍 Test Environment Detection");
        println!("============================");
        println!("Available Providers: {:?}", self.available_providers);
        println!("Telemetry Endpoint: {:?}", self.telemetry_endpoint);
        println!("Test Timeout: {:?}", self.test_timeout);
        println!("Max Parallel Tools: {}", self.max_parallel_tools);
        println!();
    }

    /// Check if a specific provider is available
    pub fn has_provider(&self, provider: ProviderType) -> bool {
        self.available_providers.contains(&provider)
    }

    /// Get recommended test configuration for a provider
    pub fn get_config_for_provider(&self, provider: ProviderType) -> Result<TestConfig, String> {
        if !self.has_provider(provider) {
            return Err(format!("Provider {:?} is not available", provider));
        }

        TestConfigBuilder::new()
            .provider(provider)
            .timeout(self.test_timeout)
            .max_parallel_tools(self.max_parallel_tools)
            .telemetry(self.telemetry_endpoint.is_some())
            .from_env()
            .build()
    }
}
