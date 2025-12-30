//! LLM Client Verification Test Suite
//!
//! This module provides comprehensive verification testing for all LLM providers
//! and models supported by the Stood library.

pub mod integration;
pub mod providers;
pub mod shared;

// Re-export key types for convenience
pub use shared::*;

use crate::llm::traits::ProviderType;
use std::collections::HashMap;

/// Main verification runner for all providers
pub struct VerificationRunner;

impl VerificationRunner {
    /// Run verification tests for all available providers
    pub async fn run_all() -> HashMap<ProviderType, Vec<VerificationResult>> {
        println!("🚀 Starting Comprehensive LLM Client Verification");
        println!("==================================================");

        // Detect available providers
        let env = shared::config::TestEnvironment::detect().await;
        env.print_summary();

        if env.available_providers.is_empty() {
            println!("❌ No providers available for testing");
            println!("   Please ensure at least one provider is configured:");
            println!("   - LM Studio: Start LM Studio with API enabled");
            println!("   - AWS Bedrock: Set AWS credentials");
            println!("   - Anthropic: Set ANTHROPIC_API_KEY");
            println!("   - OpenAI: Set OPENAI_API_KEY");
            return HashMap::new();
        }

        let mut results = HashMap::new();

        // Run tests for each available provider
        for provider in &env.available_providers {
            println!("\n📋 Testing Provider: {:?}", provider);
            println!("{}", "=".repeat(40));

            let provider_results = match provider {
                ProviderType::LmStudio => providers::lm_studio::run_all_tests().await,
                ProviderType::Bedrock => {
                    // TODO: Implement bedrock verification
                    println!("⏭️  Bedrock verification not yet implemented");
                    Vec::new()
                }
                ProviderType::Anthropic => {
                    // TODO: Implement anthropic verification
                    println!("⏭️  Anthropic verification not yet implemented");
                    Vec::new()
                }
                _ => {
                    println!("⏭️  Verification not implemented for {:?}", provider);
                    Vec::new()
                }
            };

            if !provider_results.is_empty() {
                results.insert(*provider, provider_results);
            }
        }

        // Generate cross-provider comparison report
        if results.len() > 1 {
            println!("\n📊 Cross-Provider Comparison");
            println!("============================");
            let comparison = shared::generate_comparison_report(&results);
            println!("{}", comparison);
        }

        Self::print_overall_summary(&results);
        results
    }

    /// Run verification tests for a specific provider
    pub async fn run_provider(provider: ProviderType) -> Vec<VerificationResult> {
        println!("🎯 Running Verification for Provider: {:?}", provider);
        println!("{}", "=".repeat(50));

        // Check if provider is available
        let env = shared::config::TestEnvironment::detect().await;
        if !env.has_provider(provider) {
            println!("❌ Provider {:?} is not available", provider);
            return Vec::new();
        }

        match provider {
            ProviderType::LmStudio => providers::lm_studio::run_all_tests().await,
            ProviderType::Bedrock => {
                println!("⏭️  Bedrock verification not yet implemented");
                Vec::new()
            }
            ProviderType::Anthropic => {
                println!("⏭️  Anthropic verification not yet implemented");
                Vec::new()
            }
            _ => {
                println!("⏭️  Verification not implemented for {:?}", provider);
                Vec::new()
            }
        }
    }

    /// Run specific milestone tests across all providers
    pub async fn run_milestone(milestone: u8) -> HashMap<ProviderType, Vec<VerificationResult>> {
        println!("🎯 Running MILESTONE {} across all providers", milestone);
        println!(
            "{}={}",
            "=".repeat(milestone_name(milestone).len() + 20),
            "=".repeat(20)
        );

        let env = shared::config::TestEnvironment::detect().await;
        let mut results = HashMap::new();

        for provider in &env.available_providers {
            println!("\n📋 MILESTONE {} - Provider: {:?}", milestone, provider);

            let provider_results = match provider {
                ProviderType::LmStudio => {
                    providers::lm_studio::run_milestone_tests(milestone).await
                }
                _ => {
                    println!("⏭️  Milestone tests not implemented for {:?}", provider);
                    Vec::new()
                }
            };

            if !provider_results.is_empty() {
                results.insert(*provider, provider_results);
            }
        }

        results
    }

    /// Run performance benchmarks for all providers
    pub async fn run_performance_tests() -> HashMap<ProviderType, Vec<VerificationResult>> {
        println!("🏃‍♂️ Running Performance Tests for All Providers");
        println!("================================================");

        let env = shared::config::TestEnvironment::detect().await;
        let mut results = HashMap::new();

        for provider in &env.available_providers {
            println!("\n⚡ Performance Testing - Provider: {:?}", provider);

            let provider_results = match provider {
                ProviderType::LmStudio => providers::lm_studio::run_performance_tests().await,
                _ => {
                    println!("⏭️  Performance tests not implemented for {:?}", provider);
                    Vec::new()
                }
            };

            if !provider_results.is_empty() {
                results.insert(*provider, provider_results);
            }
        }

        Self::print_performance_summary(&results);
        results
    }

    /// Print overall summary across all providers
    fn print_overall_summary(results: &HashMap<ProviderType, Vec<VerificationResult>>) {
        println!("\n🎯 OVERALL VERIFICATION SUMMARY");
        println!("===============================");

        let mut total_tests = 0;
        let mut total_passed = 0;
        let mut total_failed = 0;

        for (provider, provider_results) in results {
            let passed = provider_results.iter().filter(|r| r.success).count();
            let failed = provider_results.len() - passed;

            total_tests += provider_results.len();
            total_passed += passed;
            total_failed += failed;

            let success_rate = (passed as f64 / provider_results.len() as f64) * 100.0;
            println!(
                "  {:?}: {}/{} tests passed ({:.1}%)",
                provider,
                passed,
                provider_results.len(),
                success_rate
            );
        }

        if total_tests > 0 {
            let overall_success_rate = (total_passed as f64 / total_tests as f64) * 100.0;
            println!("\n📊 OVERALL RESULTS:");
            println!("   Total Tests: {}", total_tests);
            println!("   Passed: {} ✅", total_passed);
            println!("   Failed: {} ❌", total_failed);
            println!("   Success Rate: {:.1}%", overall_success_rate);

            if overall_success_rate >= 90.0 {
                println!("\n🎉 LLM Client Verification: EXCELLENT - Ready for production!");
            } else if overall_success_rate >= 75.0 {
                println!("\n✅ LLM Client Verification: GOOD - Minor issues to address");
            } else if overall_success_rate >= 50.0 {
                println!("\n⚠️  LLM Client Verification: NEEDS IMPROVEMENT - Several issues found");
            } else {
                println!("\n❌ LLM Client Verification: POOR - Major issues need attention");
            }
        }
    }

    /// Print performance summary
    fn print_performance_summary(results: &HashMap<ProviderType, Vec<VerificationResult>>) {
        println!("\n⚡ PERFORMANCE SUMMARY");
        println!("=====================");

        for (provider, provider_results) in results {
            let successful_results: Vec<_> =
                provider_results.iter().filter(|r| r.success).collect();

            if successful_results.is_empty() {
                println!(
                    "  {:?}: No successful tests for performance analysis",
                    provider
                );
                continue;
            }

            let avg_duration = successful_results
                .iter()
                .map(|r| r.duration.as_millis() as f64)
                .sum::<f64>()
                / successful_results.len() as f64;

            let min_duration = successful_results
                .iter()
                .map(|r| r.duration.as_millis())
                .min()
                .unwrap_or(0);

            let max_duration = successful_results
                .iter()
                .map(|r| r.duration.as_millis())
                .max()
                .unwrap_or(0);

            println!("  {:?}:", provider);
            println!("    Average: {:.0}ms", avg_duration);
            println!("    Min: {}ms", min_duration);
            println!("    Max: {}ms", max_duration);
            println!("    Tests: {}", successful_results.len());
        }
    }
}

/// Get milestone name for display
fn milestone_name(milestone: u8) -> &'static str {
    match milestone {
        1 => "Core Provider Functionality",
        2 => "Tool System Integration",
        3 => "Streaming and Real-time Features",
        4 => "Agentic Event Loop",
        5 => "Telemetry and Observability",
        6 => "Error Handling and Resilience",
        7 => "MCP and Advanced Integration",
        _ => "Unknown Milestone",
    }
}

/// Quick verification runner for CI/CD
pub async fn quick_verification() -> bool {
    println!("⚡ Quick Verification for CI/CD");
    println!("===============================");

    let env = shared::config::TestEnvironment::detect().await;
    if env.available_providers.is_empty() {
        println!("❌ No providers available - verification failed");
        return false;
    }

    // Run core tests only for available providers
    let mut overall_success = true;

    for provider in env.available_providers.iter().take(1) {
        // Test only first available provider for speed
        let config = env.get_config_for_provider(*provider);
        if let Ok(test_config) = config {
            let suite = shared::test_cases::create_core_test_suite();
            let results = suite.run(&test_config).await;

            let success_rate =
                results.iter().filter(|r| r.success).count() as f64 / results.len() as f64;
            if success_rate < 0.8 {
                // 80% minimum for quick verification
                overall_success = false;
            }
        }
    }

    if overall_success {
        println!("✅ Quick verification PASSED");
    } else {
        println!("❌ Quick verification FAILED");
    }

    overall_success
}
