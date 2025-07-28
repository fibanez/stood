//! Stress testing suite for the Stood agent library
//!
//! This module implements comprehensive stress tests to validate production readiness,
//! including memory leak detection, resource exhaustion testing, and chaos engineering.

use super::*;
use crate::stress_testing_framework::*;

/// Test memory leak detection under sustained load
#[tokio::test]
async fn test_memory_leak_detection() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping stress test - AWS credentials not available");
        return Ok(());
    }

    let config = StressTestConfig::memory_leak_detection();
    let executor = StressTestExecutor::new(config);

    let results = executor.execute().await?;

    // Validate basic execution
    assert!(results.total_operations > 0, "No operations executed");
    assert!(
        results.resource_metrics.memory_samples.len() > 0,
        "No memory samples collected"
    );

    // Log critical findings
    if results.resource_metrics.memory_leak_detected {
        println!(
            "⚠️  Memory leak detected: {:.2} MB/min",
            results.resource_metrics.memory_leak_rate_mb_per_min
        );
    } else {
        println!("✅ No memory leaks detected");
    }

    // Check for critical issues
    if !results.critical_issues.is_empty() {
        println!("❌ Critical issues found:");
        for issue in &results.critical_issues {
            println!("   • {}", issue);
        }
    }

    // Performance validation
    assert!(
        results.stability_metrics.stability_score > 0.5,
        "Stability score too low: {:.3}",
        results.stability_metrics.stability_score
    );

    Ok(())
}

/// Test resource exhaustion scenarios
#[tokio::test]
async fn test_resource_exhaustion() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping stress test - AWS credentials not available");
        return Ok(());
    }

    let config = StressTestConfig::resource_exhaustion();
    let executor = StressTestExecutor::new(config);

    let results = executor.execute().await?;

    // Validate execution
    assert!(results.total_operations > 0, "No operations executed");
    assert!(
        results.peak_concurrent_users > 50,
        "Insufficient load generated"
    );

    // Resource utilization checks
    println!("📊 Resource Exhaustion Test Results:");
    println!(
        "   Peak memory: {} MB",
        results.resource_metrics.peak_memory_mb
    );
    println!(
        "   Peak CPU: {:.1}%",
        results.resource_metrics.peak_cpu_usage
    );
    println!("   Peak users: {}", results.peak_concurrent_users);

    // System should handle high load gracefully
    assert!(
        results.resource_metrics.peak_memory_mb < 8000,
        "Memory usage too high: {} MB",
        results.resource_metrics.peak_memory_mb
    );

    // Allow higher error rates for resource exhaustion testing
    let error_rate = results.failed_operations as f64 / results.total_operations as f64;
    assert!(
        error_rate < 0.5,
        "Error rate too high: {:.2}%",
        error_rate * 100.0
    );

    Ok(())
}

/// Test system stability under chaos conditions
#[tokio::test]
async fn test_chaos_engineering() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping stress test - AWS credentials not available");
        return Ok(());
    }

    let config = StressTestConfig::chaos_engineering();
    let executor = StressTestExecutor::new(config);

    let results = executor.execute().await?;

    // Validate chaos testing execution
    assert!(results.total_operations > 0, "No operations executed");

    println!("🔥 Chaos Engineering Test Results:");
    println!("   Total operations: {}", results.total_operations);
    println!(
        "   Success rate: {:.1}%",
        results.successful_operations as f64 / results.total_operations as f64 * 100.0
    );
    println!(
        "   Stability score: {:.3}",
        results.stability_metrics.stability_score
    );
    println!("   Failure patterns: {}", results.failure_patterns.len());

    // System should maintain some level of functionality under chaos
    assert!(
        results.successful_operations > 0,
        "No successful operations under chaos"
    );

    // Check failure pattern analysis
    for pattern in &results.failure_patterns {
        println!(
            "   Pattern: {:?} - {} (freq: {})",
            pattern.severity, pattern.description, pattern.frequency
        );
    }

    // Chaos test should identify resilience characteristics
    if results.stress_test_passed {
        println!("✅ System demonstrated resilience under chaos conditions");
    } else {
        println!("⚠️  System showed stress under chaos conditions (expected for chaos testing)");
    }

    Ok(())
}

/// Test conversation cycling stress
#[tokio::test]
async fn test_conversation_cycling_stress() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping stress test - AWS credentials not available");
        return Ok(());
    }

    let config = StressTestConfig {
        test_duration: std::time::Duration::from_secs(300),
        max_concurrent_users: 25,
        ramp_pattern: RampPattern::Linear {
            step_size: 5,
            step_interval: std::time::Duration::from_secs(30),
        },
        monitoring_interval: std::time::Duration::from_secs(10),
        memory_thresholds: MemoryThresholds {
            warning_mb: 800,
            critical_mb: 1500,
            leak_threshold_mb_per_min: 5.0,
            max_memory_mb: 2000,
        },
        failure_injection: FailureInjectionConfig {
            network_latency: false,
            timeout_simulation: false,
            memory_pressure: false,
            failure_rate: 0.0,
        },
        scenario: StressTestScenario::ConversationCycling,
    };

    let executor = StressTestExecutor::new(config);
    let results = executor.execute().await?;

    // Validate conversation cycling
    assert!(results.total_operations > 0, "No operations executed");

    println!("💬 Conversation Cycling Stress Test:");
    println!("   Operations: {}", results.total_operations);
    println!(
        "   Memory stability: {}",
        !results.resource_metrics.memory_leak_detected
    );

    // Conversation cycling should be memory-efficient
    assert!(
        !results.resource_metrics.memory_leak_detected,
        "Memory leak detected during conversation cycling"
    );

    Ok(())
}

/// Test tool execution stress scenarios
#[tokio::test]
async fn test_tool_execution_stress() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping stress test - AWS credentials not available");
        return Ok(());
    }

    let config = StressTestConfig {
        test_duration: std::time::Duration::from_secs(180),
        max_concurrent_users: 15,
        ramp_pattern: RampPattern::Sustained,
        monitoring_interval: std::time::Duration::from_secs(5),
        memory_thresholds: MemoryThresholds {
            warning_mb: 1000,
            critical_mb: 2000,
            leak_threshold_mb_per_min: 8.0,
            max_memory_mb: 3000,
        },
        failure_injection: FailureInjectionConfig {
            network_latency: false,
            timeout_simulation: false,
            memory_pressure: false,
            failure_rate: 0.0,
        },
        scenario: StressTestScenario::ToolExecutionStress,
    };

    let executor = StressTestExecutor::new(config);
    let results = executor.execute().await?;

    // Validate tool execution stress
    assert!(results.total_operations > 0, "No operations executed");

    println!("🔧 Tool Execution Stress Test:");
    println!("   Operations: {}", results.total_operations);
    println!(
        "   Peak memory: {} MB",
        results.resource_metrics.peak_memory_mb
    );
    println!(
        "   Success rate: {:.1}%",
        results.successful_operations as f64 / results.total_operations as f64 * 100.0
    );

    // Tool execution should be reliable under stress
    let error_rate = results.failed_operations as f64 / results.total_operations as f64;
    assert!(
        error_rate < 0.1,
        "Too many tool execution failures: {:.2}%",
        error_rate * 100.0
    );

    Ok(())
}

/// Test mixed high-intensity workloads
#[tokio::test]
async fn test_mixed_high_intensity() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping stress test - AWS credentials not available");
        return Ok(());
    }

    let config = StressTestConfig {
        test_duration: std::time::Duration::from_secs(240),
        max_concurrent_users: 30,
        ramp_pattern: RampPattern::Exponential {
            multiplier: 1.3,
            step_interval: std::time::Duration::from_secs(45),
        },
        monitoring_interval: std::time::Duration::from_secs(8),
        memory_thresholds: MemoryThresholds {
            warning_mb: 1200,
            critical_mb: 2500,
            leak_threshold_mb_per_min: 12.0,
            max_memory_mb: 4000,
        },
        failure_injection: FailureInjectionConfig {
            network_latency: true,
            timeout_simulation: false,
            memory_pressure: false,
            failure_rate: 0.05,
        },
        scenario: StressTestScenario::MixedHighIntensity,
    };

    let executor = StressTestExecutor::new(config);
    let results = executor.execute().await?;

    // Validate mixed workload handling
    assert!(results.total_operations > 0, "No operations executed");

    println!("🌊 Mixed High-Intensity Test:");
    println!("   Total operations: {}", results.total_operations);
    println!(
        "   Peak concurrent users: {}",
        results.peak_concurrent_users
    );
    println!(
        "   Stability score: {:.3}",
        results.stability_metrics.stability_score
    );
    println!(
        "   Response time degradation: {}",
        results.stability_metrics.response_time_degradation
    );

    // Mixed workloads should maintain reasonable performance
    assert!(
        results.stability_metrics.stability_score > 0.6,
        "Stability score under mixed load too low: {:.3}",
        results.stability_metrics.stability_score
    );

    // Check for critical resource issues
    for issue in &results.critical_issues {
        println!("⚠️  Critical issue: {}", issue);
    }

    Ok(())
}

/// Integration test combining multiple stress scenarios
#[tokio::test]
async fn test_comprehensive_stress_suite() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping comprehensive stress test - AWS credentials not available");
        return Ok(());
    }

    println!("🚀 Running comprehensive stress test suite...");

    // Run a series of stress tests in sequence
    let scenarios = vec![
        (
            "Memory Leak Detection",
            StressTestConfig::memory_leak_detection(),
        ),
        (
            "Resource Exhaustion",
            StressTestConfig::resource_exhaustion(),
        ),
    ];

    let mut all_results = Vec::new();

    for (name, config) in scenarios {
        println!("\n📊 Running {}", name);
        let executor = StressTestExecutor::new(config);
        let results = executor.execute().await?;

        println!("✅ {} completed", name);
        println!("   Operations: {}", results.total_operations);
        println!(
            "   Success rate: {:.1}%",
            results.successful_operations as f64 / results.total_operations as f64 * 100.0
        );
        println!(
            "   Memory leak: {}",
            if results.resource_metrics.memory_leak_detected {
                "❌"
            } else {
                "✅"
            }
        );

        all_results.push((name, results));

        // Brief pause between tests
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }

    // Overall assessment
    println!("\n🏁 COMPREHENSIVE STRESS TEST SUMMARY");
    println!("════════════════════════════════════════");

    let mut total_operations = 0;
    let mut total_successful = 0;
    let mut memory_leaks_detected = 0;
    let mut critical_issues_count = 0;

    for (name, results) in &all_results {
        total_operations += results.total_operations;
        total_successful += results.successful_operations;
        if results.resource_metrics.memory_leak_detected {
            memory_leaks_detected += 1;
        }
        critical_issues_count += results.critical_issues.len();

        println!(
            "• {}: {} ops, {:.1}% success",
            name,
            results.total_operations,
            results.successful_operations as f64 / results.total_operations as f64 * 100.0
        );
    }

    let overall_success_rate = total_successful as f64 / total_operations as f64;

    println!("\nOverall Statistics:");
    println!("  Total operations: {}", total_operations);
    println!(
        "  Overall success rate: {:.1}%",
        overall_success_rate * 100.0
    );
    println!("  Memory leaks detected: {}", memory_leaks_detected);
    println!("  Total critical issues: {}", critical_issues_count);

    // Comprehensive test validation
    assert!(total_operations > 100, "Insufficient test coverage");
    assert!(
        overall_success_rate > 0.7,
        "Overall success rate too low: {:.1}%",
        overall_success_rate * 100.0
    );

    if memory_leaks_detected == 0 && critical_issues_count == 0 {
        println!("🎉 ALL STRESS TESTS PASSED - PRODUCTION READY");
    } else {
        println!("⚠️  Stress tests identified issues requiring attention");
    }

    Ok(())
}
