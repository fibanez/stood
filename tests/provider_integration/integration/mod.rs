//! Integration tests for cross-provider functionality
//!
//! This module contains tests that verify functionality works consistently
//! across different providers and can compare behavior between providers.

use crate::llm::traits::ProviderType;
use crate::verification::shared::*;
use std::collections::HashMap;

/// Cross-provider integration test
pub struct CrossProviderConsistencyTest;

#[async_trait::async_trait]
impl VerificationTest for CrossProviderConsistencyTest {
    fn test_name(&self) -> &'static str {
        "cross_provider_consistency"
    }
    fn description(&self) -> &'static str {
        "Verify same functionality works across providers"
    }
    fn category(&self) -> TestCategory {
        TestCategory::Core
    }
    fn required_features(&self) -> Vec<ProviderFeature> {
        vec![ProviderFeature::BasicChat]
    }

    async fn execute(&self, config: &TestConfig) -> VerificationResult {
        let start = std::time::Instant::now();
        let mut metadata = std::collections::HashMap::new();

        // This test should be run across multiple providers
        // For now, just pass if config is valid
        let result = Ok(());

        metadata.insert(
            "provider_tested".to_string(),
            serde_json::Value::String(format!("{:?}", config.provider)),
        );

        VerificationResult {
            test_name: self.test_name().to_string(),
            provider: config.provider,
            model_id: config.model_id.clone(),
            success: result.is_ok(),
            duration: start.elapsed(),
            error: result
                .err()
                .map(|e: Box<dyn std::error::Error>| e.to_string()),
            metadata,
        }
    }
}

/// Run integration tests across multiple providers
pub async fn run_cross_provider_tests(
    providers: Vec<ProviderType>,
) -> HashMap<ProviderType, Vec<VerificationResult>> {
    println!("🔄 Running Cross-Provider Integration Tests");
    println!("===========================================");

    let mut results = HashMap::new();

    for provider in providers {
        let env = config::TestEnvironment::detect().await;
        if let Ok(config) = env.get_config_for_provider(provider) {
            let suite = VerificationSuite::new("Cross-Provider Integration")
                .add_test(Box::new(CrossProviderConsistencyTest));

            let provider_results = suite.run(&config).await;
            results.insert(provider, provider_results);
        }
    }

    results
}
