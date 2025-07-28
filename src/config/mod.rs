//! Production Configuration Management
//!
//! This module provides enterprise-grade configuration management for the Stood agent library,
//! including file-based configuration, environment variable overrides, validation, and
//! production deployment patterns adapted from the Python reference implementation.

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};
use thiserror::Error;

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Environment variable parsing error: {0}")]
    EnvVarParse(String),
    #[error("File parsing error: {0}")]
    FileParse(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Main configuration structure for the Stood agent library
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoodConfig {
    /// AWS Bedrock configuration
    #[serde(default)]
    pub bedrock: BedrockConfig,
    /// OpenTelemetry configuration
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    /// Tool execution configuration
    #[serde(default)]
    pub tools: ToolsConfig,
    /// Agent behavior configuration
    #[serde(default)]
    pub agent: AgentConfig,
    /// Feature flags for deployment control
    #[serde(default)]
    pub features: FeatureFlags,
    /// Graceful shutdown configuration
    #[serde(default)]
    pub shutdown: ShutdownConfig,
}

/// AWS Bedrock service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedrockConfig {
    /// AWS region for Bedrock service
    #[serde(default = "default_region")]
    pub region: String,
    /// Default model to use
    #[serde(default = "default_model")]
    pub default_model: String,
    /// Request timeout in seconds
    #[serde(with = "duration_seconds", default = "default_timeout")]
    pub timeout: Duration,
    /// Maximum retries for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Custom endpoint URL (for testing)
    pub endpoint_url: Option<String>,
}

/// OpenTelemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled
    #[serde(default)]
    pub enabled: bool,
    /// OTLP endpoint URL
    pub otlp_endpoint: Option<String>,
    /// OTLP headers for authentication
    #[serde(default)]
    pub otlp_headers: HashMap<String, String>,
    /// Enable console export for debugging
    #[serde(default)]
    pub console_export: bool,
    /// Service name for telemetry
    #[serde(default = "default_service_name")]
    pub service_name: String,
    /// Batch timeout in milliseconds
    #[serde(default = "default_batch_timeout")]
    pub batch_timeout_ms: u64,
    /// Maximum batch size
    #[serde(default = "default_batch_size")]
    pub max_batch_size: usize,
}

/// Tool execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    /// Enable automatic tool discovery
    #[serde(default = "default_auto_discover")]
    pub auto_discover: bool,
    /// Enable hot reload of tools during development
    #[serde(default)]
    pub hot_reload: bool,
    /// Maximum parallel tool executions
    #[serde(default = "default_max_parallel")]
    pub max_parallel: usize,
    /// Directories to search for tools
    #[serde(default = "default_tool_directories")]
    pub directories: Vec<PathBuf>,
    /// Tool execution timeout
    #[serde(with = "duration_seconds", default = "default_timeout")]
    pub timeout: Duration,
    /// Enable tool result caching
    #[serde(default)]
    pub enable_caching: bool,
    /// Maximum tool result size in bytes
    #[serde(default = "default_max_result_size")]
    pub max_result_size: usize,
}

/// Agent behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Maximum conversation context length
    #[serde(default = "default_max_context_length")]
    pub max_context_length: usize,
    /// Temperature for model responses
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Maximum tokens per response
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Maximum cycles in event loop
    #[serde(default = "default_max_cycles")]
    pub max_cycles: u32,
    /// Event loop timeout
    #[serde(with = "duration_seconds", default = "default_loop_timeout")]
    pub loop_timeout: Duration,
    /// Enable streaming responses
    #[serde(default = "default_enable_streaming")]
    pub enable_streaming: bool,
}

/// Feature flags for production deployment control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable context window management
    #[serde(default = "default_true")]
    pub context_window_management: bool,
    /// Enable automatic retry on failures
    #[serde(default = "default_true")]
    pub auto_retry: bool,
    /// Enable tool hot reload (development only)
    #[serde(default)]
    pub tool_hot_reload: bool,
    /// Enable health check endpoints
    #[serde(default = "default_true")]
    pub health_checks: bool,
    /// Enable metrics collection
    #[serde(default = "default_true")]
    pub metrics_collection: bool,
    /// Enable graceful shutdown handling
    #[serde(default = "default_true")]
    pub graceful_shutdown: bool,
}

/// Graceful shutdown configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownConfig {
    /// Maximum time to wait for graceful shutdown in seconds
    #[serde(with = "duration_seconds", default = "default_shutdown_timeout")]
    pub graceful_timeout: Duration,
    /// Time to wait for in-flight requests to complete in seconds
    #[serde(with = "duration_seconds", default = "default_request_timeout")]
    pub request_timeout: Duration,
    /// Time to wait for resource cleanup in seconds
    #[serde(with = "duration_seconds", default = "default_cleanup_timeout")]
    pub cleanup_timeout: Duration,
    /// Whether to force shutdown after timeout
    #[serde(default = "default_true")]
    pub force_after_timeout: bool,
    /// Enable telemetry flush during shutdown
    #[serde(default = "default_true")]
    pub flush_telemetry: bool,
}

impl Default for BedrockConfig {
    fn default() -> Self {
        Self {
            region: "us-west-2".to_string(),
            default_model: "us.anthropic.claude-3-5-haiku-20241022-v1:0".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            endpoint_url: None,
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            otlp_endpoint: None,
            otlp_headers: HashMap::new(),
            console_export: false,
            service_name: "stood-agent".to_string(),
            batch_timeout_ms: 5000,
            max_batch_size: 512,
        }
    }
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            auto_discover: true,
            hot_reload: false,
            max_parallel: num_cpus::get().max(4),
            directories: vec![PathBuf::from("./tools")],
            timeout: Duration::from_secs(30),
            enable_caching: false,
            max_result_size: 1024 * 1024, // 1MB
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_context_length: 100_000,
            temperature: 0.7,
            max_tokens: 4096,
            max_cycles: 10,
            loop_timeout: Duration::from_secs(300),
            enable_streaming: true,
        }
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            context_window_management: true,
            auto_retry: true,
            tool_hot_reload: false,
            health_checks: true,
            metrics_collection: true,
            graceful_shutdown: true,
        }
    }
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            graceful_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(15),
            cleanup_timeout: Duration::from_secs(10),
            force_after_timeout: true,
            flush_telemetry: true,
        }
    }
}

impl StoodConfig {
    /// Load configuration from a file (supports TOML, YAML, JSON)
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        let extension = path.extension().and_then(|s| s.to_str());

        match extension {
            Some("toml") => {
                toml::from_str(&content).map_err(|e| ConfigError::FileParse(e.to_string()))
            }
            Some("yaml") | Some("yml") => {
                serde_yaml::from_str(&content).map_err(|e| ConfigError::FileParse(e.to_string()))
            }
            Some("json") => {
                serde_json::from_str(&content).map_err(|e| ConfigError::FileParse(e.to_string()))
            }
            _ => Err(ConfigError::FileParse(
                "Unsupported file format. Use .toml, .yaml, .yml, or .json".to_string(),
            )),
        }
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Bedrock configuration
        if let Ok(region) = env::var("AWS_REGION") {
            config.bedrock.region = region;
        }
        if let Ok(model) = env::var("STOOD_DEFAULT_MODEL") {
            config.bedrock.default_model = model;
        }
        if let Ok(timeout) = env::var("STOOD_BEDROCK_TIMEOUT") {
            config.bedrock.timeout =
                Duration::from_secs(timeout.parse().map_err(|e| {
                    ConfigError::EnvVarParse(format!("STOOD_BEDROCK_TIMEOUT: {}", e))
                })?);
        }

        // Telemetry configuration
        if let Ok(endpoint) = env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
            config.telemetry.enabled = true;
            config.telemetry.otlp_endpoint = Some(endpoint);
        }
        if let Ok(headers_str) = env::var("OTEL_EXPORTER_OTLP_HEADERS") {
            config.telemetry.otlp_headers = parse_otlp_headers(&headers_str)?;
        }
        if let Ok(console_export) = env::var("STOOD_OTEL_CONSOLE_EXPORT") {
            config.telemetry.console_export = console_export == "true";
        }
        if let Ok(service_name) = env::var("OTEL_SERVICE_NAME") {
            config.telemetry.service_name = service_name;
        }

        // Tools configuration
        if let Ok(max_parallel) = env::var("STOOD_TOOLS_MAX_PARALLEL") {
            config.tools.max_parallel = max_parallel.parse().map_err(|e| {
                ConfigError::EnvVarParse(format!("STOOD_TOOLS_MAX_PARALLEL: {}", e))
            })?;
        }
        if let Ok(timeout) = env::var("STOOD_TOOLS_TIMEOUT") {
            config.tools.timeout =
                Duration::from_secs(timeout.parse().map_err(|e| {
                    ConfigError::EnvVarParse(format!("STOOD_TOOLS_TIMEOUT: {}", e))
                })?);
        }

        // Agent configuration
        if let Ok(temperature) = env::var("STOOD_TEMPERATURE") {
            config.agent.temperature = temperature
                .parse()
                .map_err(|e| ConfigError::EnvVarParse(format!("STOOD_TEMPERATURE: {}", e)))?;
        }
        if let Ok(max_tokens) = env::var("STOOD_MAX_TOKENS") {
            config.agent.max_tokens = max_tokens
                .parse()
                .map_err(|e| ConfigError::EnvVarParse(format!("STOOD_MAX_TOKENS: {}", e)))?;
        }

        // Feature flags
        config.features = FeatureFlags::from_env();

        // Shutdown configuration
        if let Ok(timeout) = env::var("STOOD_SHUTDOWN_TIMEOUT") {
            config.shutdown.graceful_timeout =
                Duration::from_secs(timeout.parse().map_err(|e| {
                    ConfigError::EnvVarParse(format!("STOOD_SHUTDOWN_TIMEOUT: {}", e))
                })?);
        }
        if let Ok(timeout) = env::var("STOOD_REQUEST_TIMEOUT") {
            config.shutdown.request_timeout =
                Duration::from_secs(timeout.parse().map_err(|e| {
                    ConfigError::EnvVarParse(format!("STOOD_REQUEST_TIMEOUT: {}", e))
                })?);
        }
        if let Ok(force) = env::var("STOOD_FORCE_SHUTDOWN") {
            config.shutdown.force_after_timeout = force == "true";
        }

        Ok(config)
    }

    /// Merge configuration with environment variable overrides
    pub fn merge_with_env(mut self) -> Result<Self, ConfigError> {
        let env_config = Self::from_env()?;

        // Merge environment overrides (environment takes precedence)
        if env::var("AWS_REGION").is_ok() {
            self.bedrock.region = env_config.bedrock.region;
        }
        if env::var("STOOD_DEFAULT_MODEL").is_ok() {
            self.bedrock.default_model = env_config.bedrock.default_model;
        }
        if env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
            self.telemetry.enabled = env_config.telemetry.enabled;
            self.telemetry.otlp_endpoint = env_config.telemetry.otlp_endpoint;
        }
        if env::var("OTEL_EXPORTER_OTLP_HEADERS").is_ok() {
            self.telemetry.otlp_headers = env_config.telemetry.otlp_headers;
        }

        // Merge agent configuration
        if env::var("STOOD_TEMPERATURE").is_ok() {
            self.agent.temperature = env_config.agent.temperature;
        }
        if env::var("STOOD_MAX_TOKENS").is_ok() {
            self.agent.max_tokens = env_config.agent.max_tokens;
        }

        // Merge feature flags
        self.features = env_config.features;

        Ok(self)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate Bedrock configuration
        if self.bedrock.region.is_empty() {
            return Err(ConfigError::Validation(
                "AWS region cannot be empty".to_string(),
            ));
        }
        if self.bedrock.default_model.is_empty() {
            return Err(ConfigError::Validation(
                "Default model cannot be empty".to_string(),
            ));
        }
        if self.bedrock.timeout.as_secs() == 0 {
            return Err(ConfigError::Validation(
                "Bedrock timeout must be greater than 0".to_string(),
            ));
        }

        // Validate telemetry configuration
        if self.telemetry.enabled && self.telemetry.otlp_endpoint.is_none() {
            return Err(ConfigError::Validation(
                "OTLP endpoint required when telemetry is enabled".to_string(),
            ));
        }
        if self.telemetry.service_name.is_empty() {
            return Err(ConfigError::Validation(
                "Service name cannot be empty".to_string(),
            ));
        }

        // Validate tools configuration
        if self.tools.max_parallel == 0 {
            return Err(ConfigError::Validation(
                "Tools max_parallel must be greater than 0".to_string(),
            ));
        }
        if self.tools.timeout.as_secs() == 0 {
            return Err(ConfigError::Validation(
                "Tools timeout must be greater than 0".to_string(),
            ));
        }

        // Validate agent configuration
        if self.agent.temperature < 0.0 || self.agent.temperature > 2.0 {
            return Err(ConfigError::Validation(
                "Temperature must be between 0.0 and 2.0".to_string(),
            ));
        }
        if self.agent.max_tokens == 0 {
            return Err(ConfigError::Validation(
                "Max tokens must be greater than 0".to_string(),
            ));
        }
        if self.agent.max_cycles == 0 {
            return Err(ConfigError::Validation(
                "Max cycles must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the configuration for Bedrock client initialization
    pub fn bedrock_config(&self) -> &BedrockConfig {
        &self.bedrock
    }

    /// Get the configuration for telemetry initialization
    pub fn telemetry_config(&self) -> &TelemetryConfig {
        &self.telemetry
    }

    /// Get the configuration for tools
    pub fn tools_config(&self) -> &ToolsConfig {
        &self.tools
    }

    /// Get the configuration for agent behavior
    pub fn agent_config(&self) -> &AgentConfig {
        &self.agent
    }

    /// Get the feature flags
    pub fn features(&self) -> &FeatureFlags {
        &self.features
    }
}

impl FeatureFlags {
    /// Load feature flags from environment variables
    pub fn from_env() -> Self {
        Self {
            context_window_management: env::var("STOOD_CONTEXT_MANAGEMENT")
                .map(|v| v == "true")
                .unwrap_or(true),
            auto_retry: env::var("STOOD_AUTO_RETRY")
                .map(|v| v == "true")
                .unwrap_or(true),
            tool_hot_reload: env::var("STOOD_TOOL_HOT_RELOAD")
                .map(|v| v == "true")
                .unwrap_or(false),
            health_checks: env::var("STOOD_HEALTH_CHECKS")
                .map(|v| v == "true")
                .unwrap_or(true),
            metrics_collection: env::var("STOOD_METRICS_COLLECTION")
                .map(|v| v == "true")
                .unwrap_or(true),
            graceful_shutdown: env::var("STOOD_GRACEFUL_SHUTDOWN")
                .map(|v| v == "true")
                .unwrap_or(true),
        }
    }
}

/// Parse OTLP headers from environment variable format
/// Expected format: "key1=value1,key2=value2"
fn parse_otlp_headers(headers_str: &str) -> Result<HashMap<String, String>, ConfigError> {
    let mut headers = HashMap::new();

    for pair in headers_str.split(',') {
        let parts: Vec<&str> = pair.trim().splitn(2, '=').collect();
        if parts.len() == 2 {
            headers.insert(parts[0].to_string(), parts[1].to_string());
        } else {
            return Err(ConfigError::EnvVarParse(format!(
                "Invalid OTLP header format: {}",
                pair
            )));
        }
    }

    Ok(headers)
}

/// Custom serialization for Duration as seconds
mod duration_seconds {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

// Default value functions for serde
fn default_region() -> String {
    "us-west-2".to_string()
}

fn default_model() -> String {
    "us.anthropic.claude-3-5-haiku-20241022-v1:0".to_string()
}

fn default_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_max_retries() -> u32 {
    3
}

fn default_service_name() -> String {
    "stood-agent".to_string()
}

fn default_batch_timeout() -> u64 {
    5000
}

fn default_batch_size() -> usize {
    512
}

fn default_auto_discover() -> bool {
    true
}

fn default_max_parallel() -> usize {
    num_cpus::get().max(4)
}

fn default_tool_directories() -> Vec<PathBuf> {
    vec![PathBuf::from("./tools")]
}

fn default_max_result_size() -> usize {
    1024 * 1024 // 1MB
}

fn default_max_context_length() -> usize {
    100_000
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_max_cycles() -> u32 {
    10
}

fn default_loop_timeout() -> Duration {
    Duration::from_secs(300)
}

fn default_enable_streaming() -> bool {
    true
}

fn default_true() -> bool {
    true
}

fn default_shutdown_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_request_timeout() -> Duration {
    Duration::from_secs(15)
}

fn default_cleanup_timeout() -> Duration {
    Duration::from_secs(10)
}

#[cfg(test)]
mod test_example;

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = StoodConfig::default();
        assert_eq!(config.bedrock.region, "us-west-2");
        assert_eq!(
            config.bedrock.default_model,
            "us.anthropic.claude-3-5-haiku-20241022-v1:0"
        );
        assert!(!config.telemetry.enabled);
        assert!(config.features.context_window_management);
    }

    #[test]
    fn test_config_validation() {
        let mut config = StoodConfig::default();
        assert!(config.validate().is_ok());

        // Test invalid configuration
        config.bedrock.region = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_toml_config_loading() {
        let toml_content = r#"
[bedrock]
region = "us-east-1"
default_model = "anthropic.claude-3-sonnet-20240229-v1:0"
timeout = 30
max_retries = 3

[telemetry]
enabled = true
service_name = "test-service"
console_export = true

[tools]
max_parallel = 8
timeout = 20

[agent]
temperature = 0.5
max_tokens = 2048
max_cycles = 5
loop_timeout = 300

[features]
context_window_management = false
"#;

        let temp_file = NamedTempFile::with_suffix(".toml").unwrap();
        std::fs::write(temp_file.path(), toml_content).unwrap();

        let config = StoodConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.bedrock.region, "us-east-1");
        assert_eq!(config.agent.temperature, 0.5);
        assert!(!config.features.context_window_management);
        assert_eq!(config.tools.max_parallel, 8);
        assert!(config.telemetry.enabled);
    }

    #[test]
    fn test_env_var_loading() {
        env::set_var("AWS_REGION", "eu-west-1");
        env::set_var("STOOD_TEMPERATURE", "0.3");
        env::set_var("STOOD_AUTO_RETRY", "false");

        let config = StoodConfig::from_env().unwrap();
        assert_eq!(config.bedrock.region, "eu-west-1");
        assert_eq!(config.agent.temperature, 0.3);
        assert!(!config.features.auto_retry);

        // Clean up
        env::remove_var("AWS_REGION");
        env::remove_var("STOOD_TEMPERATURE");
        env::remove_var("STOOD_AUTO_RETRY");
    }

    #[test]
    fn test_otlp_headers_parsing() {
        let headers_str = "authorization=Bearer token123,x-custom-header=value456";
        let headers = parse_otlp_headers(headers_str).unwrap();

        assert_eq!(
            headers.get("authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(
            headers.get("x-custom-header"),
            Some(&"value456".to_string())
        );
    }

    #[test]
    fn test_feature_flags_from_env() {
        env::set_var("STOOD_CONTEXT_MANAGEMENT", "false");
        env::set_var("STOOD_HEALTH_CHECKS", "true");

        let flags = FeatureFlags::from_env();
        assert!(!flags.context_window_management);
        assert!(flags.health_checks);

        // Clean up
        env::remove_var("STOOD_CONTEXT_MANAGEMENT");
        env::remove_var("STOOD_HEALTH_CHECKS");
    }
}
