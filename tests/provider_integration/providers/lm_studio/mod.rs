//! LM Studio provider verification tests
//!
//! This module contains comprehensive verification tests for the LM Studio provider,
//! focusing on Gemma 3 12B model but designed to work with any LM Studio model.

pub mod core;
// TODO: Implement these modules
// pub mod tools;
// pub mod streaming;
// pub mod agentic;
// pub mod performance;

use crate::llm::traits::ProviderType;
use crate::verification::shared::config::*;
use crate::verification::shared::*;

/// LM Studio-specific test configuration
pub struct LMStudioTestConfig;

impl LMStudioTestConfig {
    /// Create default LM Studio test configuration
    pub fn default() -> TestConfig {
        TestConfig::lm_studio_gemma3()
    }

    /// Create LM Studio test configuration with custom model
    pub fn with_model(model_id: impl Into<String>) -> TestConfig {
        TestConfigBuilder::new()
            .provider(ProviderType::LmStudio)
            .model_id(model_id)
            .from_env()
            .build()
            .expect("Failed to create LM Studio test config")
    }

    /// Create fast test configuration for quick feedback
    pub fn fast() -> TestConfig {
        TestConfigBuilder::new()
            .provider(ProviderType::LmStudio)
            .timeout(std::time::Duration::from_secs(15))
            .max_retries(1)
            .max_parallel_tools(2)
            .from_env()
            .build()
            .expect("Failed to create fast LM Studio test config")
    }

    /// Create thorough test configuration for comprehensive testing
    pub fn thorough() -> TestConfig {
        TestConfigBuilder::new()
            .provider(ProviderType::LmStudio)
            .timeout(std::time::Duration::from_secs(60))
            .max_retries(3)
            .max_parallel_tools(4)
            .from_env()
            .build()
            .expect("Failed to create thorough LM Studio test config")
    }
}

/// Run all LM Studio verification tests
pub async fn run_all_tests() -> Vec<VerificationResult> {
    println!("🚀 Starting LM Studio Provider Verification");
    println!("============================================");

    // Configure provider registry first
    println!("⚙️  Configuring provider registry...");
    if let Err(e) = crate::llm::registry::ProviderRegistry::configure().await {
        println!("❌ Failed to configure provider registry: {}", e);
        return Vec::new();
    }
    println!("✅ Provider registry configured");

    // Check if LM Studio is available
    if !config::ProviderChecker::check_lm_studio().await {
        println!("❌ LM Studio is not available - skipping tests");
        println!("   Make sure LM Studio is running with API enabled at http://localhost:1234");
        return Vec::new();
    }

    println!("✅ LM Studio detected and available");

    let config = LMStudioTestConfig::default();
    println!("📋 Using model: {}", config.model_id);
    println!("⚙️  Configuration: {:?}", config);

    let mut all_results = Vec::new();

    // Run core functionality tests (Milestone 1)
    println!("\n🎯 MILESTONE 1: Core Provider Functionality");
    let core_suite = test_cases::create_core_test_suite();
    let core_results = core_suite.run(&config).await;
    all_results.extend(core_results);

    // Run tool system tests (Milestone 2)
    println!("\n🛠️  MILESTONE 2: Tool System Integration");
    let tools_suite = test_cases::create_tools_test_suite();
    let tools_results = tools_suite.run(&config).await;
    all_results.extend(tools_results);

    // Run streaming tests (Milestone 3) - only if provider supports streaming
    if config.enable_streaming {
        println!("\n📡 MILESTONE 3: Streaming and Real-time Features");
        let streaming_suite = test_cases::create_streaming_test_suite();
        let streaming_results = streaming_suite.run(&config).await;
        all_results.extend(streaming_results);
    } else {
        println!("\n⏭️  MILESTONE 3: Streaming tests skipped (not enabled)");
    }

    // Print final summary
    print_final_summary(&all_results);

    all_results
}

/// Run specific milestone tests
pub async fn run_milestone_tests(milestone: u8) -> Vec<VerificationResult> {
    let config = LMStudioTestConfig::default();

    match milestone {
        1 => {
            println!("🎯 Running MILESTONE 1: Core Provider Functionality");
            let suite = test_cases::create_core_test_suite();
            suite.run(&config).await
        }
        2 => {
            println!("🛠️  Running MILESTONE 2: Tool System Integration");
            let suite = test_cases::create_tools_test_suite();
            suite.run(&config).await
        }
        3 => {
            println!("📡 Running MILESTONE 3: Streaming and Real-time Features");
            let suite = test_cases::create_streaming_test_suite();
            suite.run(&config).await
        }
        _ => {
            println!(
                "❌ Invalid milestone number: {}. Valid options are 1, 2, 3",
                milestone
            );
            Vec::new()
        }
    }
}

/// Run performance benchmarks
pub async fn run_performance_tests() -> Vec<VerificationResult> {
    println!("🏃‍♂️ Running LM Studio Performance Tests");
    println!("========================================");

    let config = LMStudioTestConfig::thorough();

    // TODO: Implement performance-specific tests
    // For now, run comprehensive test suite with performance focus
    let suite = test_cases::create_comprehensive_test_suite();
    suite.run(&config).await
}

/// Print final summary of all test results
fn print_final_summary(results: &[VerificationResult]) {
    let total = results.len();
    let passed = results.iter().filter(|r| r.success).count();
    let failed = total - passed;

    println!("\n📊 LM Studio Verification Summary");
    println!("=================================");
    println!("Total Tests: {}", total);
    println!("Passed: {} ✅", passed);
    println!("Failed: {} ❌", failed);

    if failed > 0 {
        println!("\n❌ Failed Tests:");
        for result in results.iter().filter(|r| !r.success) {
            println!(
                "   - {}: {}",
                result.test_name,
                result.error.as_deref().unwrap_or("Unknown error")
            );
        }
    }

    let success_rate = (passed as f64 / total as f64) * 100.0;
    println!("\nSuccess Rate: {:.1}%", success_rate);

    // Performance summary
    let avg_duration = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.duration.as_millis() as f64)
        .sum::<f64>()
        / passed.max(1) as f64;

    println!("Average Response Time: {:.0}ms", avg_duration);

    // Feature coverage summary
    let core_tests = results
        .iter()
        .filter(|r| r.test_name.contains("chat") || r.test_name.contains("conversation"))
        .count();
    let tool_tests = results
        .iter()
        .filter(|r| r.test_name.contains("tool"))
        .count();
    let streaming_tests = results
        .iter()
        .filter(|r| r.test_name.contains("streaming"))
        .count();

    println!("\nFeature Coverage:");
    println!("  Core Chat: {} tests", core_tests);
    println!("  Tools: {} tests", tool_tests);
    println!("  Streaming: {} tests", streaming_tests);

    if success_rate >= 90.0 {
        println!("\n🎉 LM Studio provider verification: EXCELLENT");
    } else if success_rate >= 75.0 {
        println!("\n✅ LM Studio provider verification: GOOD");
    } else if success_rate >= 50.0 {
        println!("\n⚠️  LM Studio provider verification: NEEDS IMPROVEMENT");
    } else {
        println!("\n❌ LM Studio provider verification: POOR - NEEDS ATTENTION");
    }
}
