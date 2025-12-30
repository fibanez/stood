//! Shared test framework for LLM Client verification
//!
//! This module provides a provider-agnostic test framework that can verify
//! all Stood features across different LLM providers and models.

pub mod assertions;
pub mod config;
pub mod fixtures;
pub mod test_cases;
pub mod token_counting_tests;

use crate::llm::traits::ProviderType;
use serde_json::Value;
use std::time::Duration;

/// Test result for a single verification
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub test_name: String,
    pub provider: ProviderType,
    pub model_id: String,
    pub success: bool,
    pub duration: Duration,
    pub error: Option<String>,
    pub metadata: std::collections::HashMap<String, Value>,
}

/// Provider-agnostic test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub provider: ProviderType,
    pub model_id: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub enable_telemetry: bool,
    pub enable_streaming: bool,
    pub max_parallel_tools: usize,
}

impl TestConfig {
    /// Create test config for LM Studio Gemma 3 27B
    pub fn lm_studio_gemma3() -> Self {
        Self {
            provider: ProviderType::LmStudio,
            model_id: "google/gemma-3-27b".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            enable_telemetry: true,
            enable_streaming: true,
            max_parallel_tools: 4,
        }
    }

    /// Create test config for AWS Bedrock Claude 3.5 Haiku
    pub fn bedrock_claude35_haiku() -> Self {
        Self {
            provider: ProviderType::Bedrock,
            model_id: "us.anthropic.claude-3-5-haiku-20241022-v1:0".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            enable_telemetry: true,
            enable_streaming: true,
            max_parallel_tools: 4,
        }
    }

    /// Create test config for Anthropic Direct Claude 3.5 Sonnet
    pub fn anthropic_claude35_sonnet() -> Self {
        Self {
            provider: ProviderType::Anthropic,
            model_id: "claude-3-5-sonnet-20241022".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            enable_telemetry: true,
            enable_streaming: false, // TODO: Implement Anthropic streaming
            max_parallel_tools: 4,
        }
    }
}

/// Trait for provider-agnostic test execution
#[async_trait::async_trait]
pub trait VerificationTest {
    /// Test name for reporting
    fn test_name(&self) -> &'static str;

    /// Test description
    fn description(&self) -> &'static str;

    /// Test category (core, tools, streaming, etc.)
    fn category(&self) -> TestCategory;

    /// Whether this test requires specific provider features
    fn required_features(&self) -> Vec<ProviderFeature>;

    /// Execute the test with the given configuration
    async fn execute(&self, config: &TestConfig) -> VerificationResult;

    /// Setup any required test state
    async fn setup(&self, _config: &TestConfig) -> Result<(), Box<dyn std::error::Error>> {
        // Default: no setup required
        Ok(())
    }

    /// Cleanup after test execution
    async fn cleanup(&self, _config: &TestConfig) -> Result<(), Box<dyn std::error::Error>> {
        // Default: no cleanup required
        Ok(())
    }
}

/// Test categories for organization
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestCategory {
    Core,          // Basic provider functionality
    Tools,         // Tool calling and execution
    Streaming,     // Real-time streaming features
    Agentic,       // Event loop and multi-step reasoning
    Telemetry,     // Observability and monitoring
    ErrorHandling, // Error resilience and recovery
    Advanced,      // MCP and advanced features
    Performance,   // Benchmarking and optimization
}

/// Provider features that tests may require
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderFeature {
    BasicChat,
    ToolCalling,
    Streaming,
    ThinkingMode,
    Vision,
    ContextCaching,
    ParallelExecution,
    CustomTools,
    MCPIntegration,
}

/// Test suite containing multiple verification tests
pub struct VerificationSuite {
    pub name: String,
    pub tests: Vec<Box<dyn VerificationTest + Send + Sync>>,
}

impl VerificationSuite {
    /// Create a new test suite
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tests: Vec::new(),
        }
    }

    /// Add a test to the suite
    pub fn add_test(mut self, test: Box<dyn VerificationTest + Send + Sync>) -> Self {
        self.tests.push(test);
        self
    }

    /// Run all tests in the suite
    pub async fn run(&self, config: &TestConfig) -> Vec<VerificationResult> {
        let mut results = Vec::new();

        println!("🚀 Running verification suite: {}", self.name);
        println!(
            "📋 Provider: {:?}, Model: {}",
            config.provider, config.model_id
        );

        for test in &self.tests {
            // Check if provider supports required features
            if !self.provider_supports_features(config.provider, &test.required_features()) {
                println!("⏭️  Skipping {} - unsupported features", test.test_name());
                continue;
            }

            println!("🧪 Running test: {}", test.test_name());

            // Setup
            if let Err(e) = test.setup(config).await {
                println!("❌ Setup failed for {}: {}", test.test_name(), e);
                continue;
            }

            // Execute
            let result = test.execute(config).await;

            // Cleanup
            if let Err(e) = test.cleanup(config).await {
                println!("⚠️  Cleanup failed for {}: {}", test.test_name(), e);
            }

            // Report result
            if result.success {
                println!("✅ {} - {:?}", test.test_name(), result.duration);
            } else {
                println!(
                    "❌ {} - {}",
                    test.test_name(),
                    result.error.as_deref().unwrap_or("Unknown error")
                );
            }

            results.push(result);
        }

        self.print_summary(&results);
        results
    }

    /// Check if provider supports the required features
    fn provider_supports_features(
        &self,
        provider: ProviderType,
        features: &[ProviderFeature],
    ) -> bool {
        for feature in features {
            if !self.provider_has_feature(provider, feature) {
                return false;
            }
        }
        true
    }

    /// Check if a provider supports a specific feature
    fn provider_has_feature(&self, provider: ProviderType, feature: &ProviderFeature) -> bool {
        match (provider, feature) {
            // All providers support basic chat
            (_, ProviderFeature::BasicChat) => true,

            // Tool calling support varies
            (ProviderType::Bedrock, ProviderFeature::ToolCalling) => true,
            (ProviderType::LmStudio, ProviderFeature::ToolCalling) => true, // Depends on model
            (ProviderType::Anthropic, ProviderFeature::ToolCalling) => true,

            // Streaming support
            (ProviderType::Bedrock, ProviderFeature::Streaming) => true,
            (ProviderType::LmStudio, ProviderFeature::Streaming) => true,
            (ProviderType::Anthropic, ProviderFeature::Streaming) => false, // TODO

            // Advanced features
            (ProviderType::Anthropic, ProviderFeature::ThinkingMode) => false, // TODO
            (ProviderType::Bedrock, ProviderFeature::Vision) => true,
            (ProviderType::Anthropic, ProviderFeature::Vision) => false, // TODO

            // Default: assume not supported
            _ => false,
        }
    }

    /// Print test suite summary
    fn print_summary(&self, results: &[VerificationResult]) {
        let total = results.len();
        let passed = results.iter().filter(|r| r.success).count();
        let failed = total - passed;

        println!("\n📊 Test Suite Summary: {}", self.name);
        println!("   Total:  {}", total);
        println!("   Passed: {} ✅", passed);
        println!("   Failed: {} ❌", failed);

        if failed > 0 {
            println!("\n❌ Failed Tests:");
            for result in results.iter().filter(|r| !r.success) {
                println!(
                    "   - {}: {}",
                    result.test_name,
                    result.error.as_deref().unwrap_or("Unknown")
                );
            }
        }

        let success_rate = (passed as f64 / total as f64) * 100.0;
        println!("   Success Rate: {:.1}%\n", success_rate);
    }
}

/// Helper function to run tests across multiple provider configurations
pub async fn run_cross_provider_tests(
    suite: &VerificationSuite,
    configs: &[TestConfig],
) -> std::collections::HashMap<ProviderType, Vec<VerificationResult>> {
    let mut all_results = std::collections::HashMap::new();

    for config in configs {
        let results = suite.run(config).await;
        all_results.insert(config.provider, results);
    }

    all_results
}

/// Generate a cross-provider comparison report
pub fn generate_comparison_report(
    results: &std::collections::HashMap<ProviderType, Vec<VerificationResult>>,
) -> String {
    let mut report = String::new();
    report.push_str("# Cross-Provider Verification Report\n\n");

    // Success rates by provider
    report.push_str("## Success Rates by Provider\n\n");
    for (provider, provider_results) in results {
        let total = provider_results.len();
        let passed = provider_results.iter().filter(|r| r.success).count();
        let rate = (passed as f64 / total as f64) * 100.0;
        report.push_str(&format!(
            "- {:?}: {}/{} ({:.1}%)\n",
            provider, passed, total, rate
        ));
    }

    // Performance comparison
    report.push_str("\n## Average Response Times\n\n");
    for (provider, provider_results) in results {
        let avg_duration = provider_results
            .iter()
            .filter(|r| r.success)
            .map(|r| r.duration.as_millis() as f64)
            .sum::<f64>()
            / provider_results.iter().filter(|r| r.success).count() as f64;
        report.push_str(&format!("- {:?}: {:.0}ms\n", provider, avg_duration));
    }

    report
}
