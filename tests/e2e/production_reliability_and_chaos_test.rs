//! Reliability testing suite for the Stood agent library
//!
//! This module implements comprehensive reliability tests including failure injection,
//! chaos engineering, recovery validation, and resilience testing for production readiness.

use super::*;
use crate::reliability_testing_framework::*;

/// Test basic failure injection capabilities
#[tokio::test]
async fn test_basic_failure_injection() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping reliability test - AWS credentials not available");
        return Ok(());
    }

    let config = ReliabilityTestConfig::basic_failure_injection();
    let executor = ReliabilityTestExecutor::new(config);

    let results = executor.execute().await?;

    // Validate basic execution
    assert!(results.total_operations > 0, "No operations executed");
    assert!(results.resilience_score > 0.0, "Invalid resilience score");

    // Check failure injection effectiveness
    assert!(
        results.failure_injection_results.injected_failures > 0,
        "No failures injected"
    );

    println!("📊 Failure Injection Test Results:");
    println!("   Resilience score: {:.3}", results.resilience_score);
    println!(
        "   Availability: {:.2}%",
        results.slo_compliance.availability_achieved * 100.0
    );
    println!(
        "   Failures injected: {}",
        results.failure_injection_results.injected_failures
    );
    println!(
        "   Detection rate: {:.1}%",
        results.failure_injection_results.failure_detection_rate * 100.0
    );

    // Basic reliability validation
    assert!(
        results.resilience_score >= 0.5,
        "Resilience score too low: {:.3}",
        results.resilience_score
    );
    assert!(
        results.slo_compliance.availability_achieved >= 0.80,
        "Availability too low: {:.2}%",
        results.slo_compliance.availability_achieved * 100.0
    );

    Ok(())
}

/// Test chaos engineering scenarios
#[tokio::test]
async fn test_chaos_engineering() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping reliability test - AWS credentials not available");
        return Ok(());
    }

    let config = ReliabilityTestConfig::chaos_engineering();
    let executor = ReliabilityTestExecutor::new(config);

    let results = executor.execute().await?;

    // Validate chaos testing execution
    assert!(results.total_operations > 0, "No operations executed");

    println!("🌪️  Chaos Engineering Test Results:");
    println!("   Total operations: {}", results.total_operations);
    println!(
        "   Chaos events triggered: {}",
        results.chaos_engineering_results.chaos_events_triggered
    );
    println!(
        "   System survived chaos: {}",
        results.chaos_engineering_results.system_survived_chaos
    );
    println!(
        "   Performance degradation: {:.1}%",
        results.chaos_engineering_results.performance_degradation * 100.0
    );
    println!(
        "   Chaos resilience score: {:.3}",
        results.chaos_engineering_results.chaos_resilience_score
    );

    // System should survive chaos engineering
    assert!(
        results.chaos_engineering_results.system_survived_chaos,
        "System did not survive chaos"
    );
    assert!(
        results.successful_operations > 0,
        "No successful operations during chaos"
    );

    // Performance degradation should be within acceptable limits for chaos testing
    assert!(
        results.chaos_engineering_results.performance_degradation < 0.8,
        "Excessive performance degradation: {:.1}%",
        results.chaos_engineering_results.performance_degradation * 100.0
    );

    Ok(())
}

/// Test production readiness validation
#[tokio::test]
async fn test_production_readiness() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping reliability test - AWS credentials not available");
        return Ok(());
    }

    let config = ReliabilityTestConfig::production_readiness();
    let executor = ReliabilityTestExecutor::new(config);

    let results = executor.execute().await?;

    // Validate production readiness
    assert!(results.total_operations > 0, "No operations executed");

    println!("🏭 Production Readiness Test Results:");
    println!("   Total operations: {}", results.total_operations);
    println!(
        "   Overall availability: {:.2}%",
        results.slo_compliance.availability_achieved * 100.0
    );
    println!("   Resilience score: {:.3}", results.resilience_score);
    println!(
        "   SLO compliance: {:.1}%",
        results.slo_compliance.overall_slo_compliance * 100.0
    );
    println!(
        "   Recovery success rate: {:.1}%",
        results.recovery_metrics.recovery_success_rate * 100.0
    );

    // Production readiness criteria
    assert!(
        results.resilience_score >= 0.7,
        "Resilience score insufficient for production: {:.3}",
        results.resilience_score
    );
    assert!(
        results.slo_compliance.availability_achieved >= 0.90,
        "Availability insufficient for production: {:.2}%",
        results.slo_compliance.availability_achieved * 100.0
    );

    // Check critical issues
    if !results.critical_reliability_issues.is_empty() {
        println!("⚠️  Critical reliability issues found:");
        for issue in &results.critical_reliability_issues {
            println!("   • {}", issue);
        }
    }

    // Recovery validation
    if results.recovery_metrics.recovery_attempts > 0 {
        assert!(
            results.recovery_metrics.recovery_success_rate >= 0.8,
            "Recovery success rate too low: {:.1}%",
            results.recovery_metrics.recovery_success_rate * 100.0
        );
    }

    Ok(())
}

/// Test recovery mechanisms under failure conditions
#[tokio::test]
async fn test_recovery_mechanisms() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping reliability test - AWS credentials not available");
        return Ok(());
    }

    let config = ReliabilityTestConfig {
        test_duration: std::time::Duration::from_secs(180),
        concurrent_users: 15,
        failure_injection: FailureInjectionStrategy::SystematicFailures {
            pattern: FailurePattern::Burst {
                burst_size: 5,
                burst_interval: std::time::Duration::from_secs(30),
            },
        },
        recovery_validation: RecoveryValidationConfig {
            max_recovery_time: std::time::Duration::from_secs(30),
            recovery_slo: ServiceLevelObjectives {
                availability: 0.90,
                max_error_rate: 0.10,
                response_time_p95: std::time::Duration::from_secs(12),
                response_time_p99: std::time::Duration::from_secs(25),
            },
            validate_automatic_recovery: true,
            simulate_manual_recovery: false,
        },
        chaos_config: ChaosEngineeringConfig {
            random_termination: false,
            config_corruption: false,
            dependency_failures: false,
            clock_skew: false,
            chaos_intensity: 0.0,
        },
        resilience_thresholds: ResilienceThresholds {
            min_availability: 0.85,
            max_error_rate_during_failure: 0.20,
            max_recovery_time: std::time::Duration::from_secs(45),
            min_resilience_score: 0.7,
        },
    };

    let executor = ReliabilityTestExecutor::new(config);
    let results = executor.execute().await?;

    // Validate recovery testing
    assert!(results.total_operations > 0, "No operations executed");

    println!("🔄 Recovery Mechanisms Test Results:");
    println!("   Operations: {}", results.total_operations);
    println!(
        "   Recovery attempts: {}",
        results.recovery_metrics.recovery_attempts
    );
    println!(
        "   Recovery success rate: {:.1}%",
        results.recovery_metrics.recovery_success_rate * 100.0
    );
    println!(
        "   Average recovery time: {:?}",
        results.recovery_metrics.average_recovery_time
    );
    println!(
        "   Service degradation periods: {}",
        results.recovery_metrics.service_degradation_periods.len()
    );

    // Recovery should be effective
    if results.recovery_metrics.recovery_attempts > 0 {
        assert!(
            results.recovery_metrics.recovery_success_rate >= 0.7,
            "Recovery success rate too low: {:.1}%",
            results.recovery_metrics.recovery_success_rate * 100.0
        );
        assert!(
            results.recovery_metrics.average_recovery_time <= std::time::Duration::from_secs(60),
            "Recovery time too slow: {:?}",
            results.recovery_metrics.average_recovery_time
        );
    }

    Ok(())
}

/// Test system resilience under various failure patterns
#[tokio::test]
async fn test_failure_pattern_resilience() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping reliability test - AWS credentials not available");
        return Ok(());
    }

    // Test different failure patterns
    let failure_patterns = vec![
        (
            "Random Failures",
            FailureInjectionStrategy::RandomFailures { failure_rate: 0.15 },
        ),
        (
            "Periodic Failures",
            FailureInjectionStrategy::SystematicFailures {
                pattern: FailurePattern::Periodic {
                    interval: std::time::Duration::from_secs(45),
                },
            },
        ),
        (
            "Cascade Failures",
            FailureInjectionStrategy::CascadeFailures {
                cascade_probability: 0.2,
            },
        ),
    ];

    let mut pattern_results = Vec::new();

    for (pattern_name, failure_strategy) in failure_patterns {
        println!("\n🧪 Testing failure pattern: {}", pattern_name);

        let config = ReliabilityTestConfig {
            test_duration: std::time::Duration::from_secs(120),
            concurrent_users: 10,
            failure_injection: failure_strategy,
            recovery_validation: RecoveryValidationConfig {
                max_recovery_time: std::time::Duration::from_secs(30),
                recovery_slo: ServiceLevelObjectives {
                    availability: 0.85,
                    max_error_rate: 0.15,
                    response_time_p95: std::time::Duration::from_secs(15),
                    response_time_p99: std::time::Duration::from_secs(30),
                },
                validate_automatic_recovery: true,
                simulate_manual_recovery: false,
            },
            chaos_config: ChaosEngineeringConfig {
                random_termination: false,
                config_corruption: false,
                dependency_failures: false,
                clock_skew: false,
                chaos_intensity: 0.0,
            },
            resilience_thresholds: ResilienceThresholds {
                min_availability: 0.80,
                max_error_rate_during_failure: 0.25,
                max_recovery_time: std::time::Duration::from_secs(60),
                min_resilience_score: 0.6,
            },
        };

        let executor = ReliabilityTestExecutor::new(config);
        let results = executor.execute().await?;

        println!("   Operations: {}", results.total_operations);
        println!(
            "   Availability: {:.1}%",
            results.slo_compliance.availability_achieved * 100.0
        );
        println!("   Resilience: {:.3}", results.resilience_score);

        // Validate each pattern
        assert!(
            results.total_operations > 0,
            "No operations executed for {}",
            pattern_name
        );
        assert!(
            results.resilience_score >= 0.5,
            "{} resilience too low: {:.3}",
            pattern_name,
            results.resilience_score
        );

        pattern_results.push((pattern_name, results));

        // Brief pause between pattern tests
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }

    // Overall pattern analysis
    println!("\n📊 FAILURE PATTERN RESILIENCE SUMMARY");
    println!("════════════════════════════════════════");

    let mut total_operations = 0;
    let mut total_resilience = 0.0;

    for (pattern_name, results) in &pattern_results {
        total_operations += results.total_operations;
        total_resilience += results.resilience_score;

        println!(
            "• {}: {:.3} resilience, {:.1}% availability",
            pattern_name,
            results.resilience_score,
            results.slo_compliance.availability_achieved * 100.0
        );
    }

    let avg_resilience = total_resilience / pattern_results.len() as f64;

    println!("\nOverall Statistics:");
    println!("  Total operations across patterns: {}", total_operations);
    println!("  Average resilience score: {:.3}", avg_resilience);

    // System should be resilient across different failure patterns
    assert!(
        avg_resilience >= 0.6,
        "Average resilience across patterns too low: {:.3}",
        avg_resilience
    );
    assert!(
        total_operations >= 100,
        "Insufficient test coverage across patterns"
    );

    println!("✅ System demonstrated resilience across various failure patterns");

    Ok(())
}

/// Test latency injection and response time resilience
#[tokio::test]
async fn test_latency_resilience() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping reliability test - AWS credentials not available");
        return Ok(());
    }

    let config = ReliabilityTestConfig {
        test_duration: std::time::Duration::from_secs(150),
        concurrent_users: 12,
        failure_injection: FailureInjectionStrategy::LatencyInjection {
            min_delay: std::time::Duration::from_millis(500),
            max_delay: std::time::Duration::from_secs(3),
        },
        recovery_validation: RecoveryValidationConfig {
            max_recovery_time: std::time::Duration::from_secs(20),
            recovery_slo: ServiceLevelObjectives {
                availability: 0.90,
                max_error_rate: 0.10,
                response_time_p95: std::time::Duration::from_secs(20), // More lenient for latency testing
                response_time_p99: std::time::Duration::from_secs(35),
            },
            validate_automatic_recovery: true,
            simulate_manual_recovery: false,
        },
        chaos_config: ChaosEngineeringConfig {
            random_termination: false,
            config_corruption: false,
            dependency_failures: false,
            clock_skew: false,
            chaos_intensity: 0.0,
        },
        resilience_thresholds: ResilienceThresholds {
            min_availability: 0.85,
            max_error_rate_during_failure: 0.15,
            max_recovery_time: std::time::Duration::from_secs(30),
            min_resilience_score: 0.7,
        },
    };

    let executor = ReliabilityTestExecutor::new(config);
    let results = executor.execute().await?;

    // Validate latency resilience
    assert!(results.total_operations > 0, "No operations executed");

    println!("🐌 Latency Resilience Test Results:");
    println!("   Operations: {}", results.total_operations);
    println!(
        "   Availability: {:.1}%",
        results.slo_compliance.availability_achieved * 100.0
    );
    println!(
        "   P95 response time: {:?}",
        results.slo_compliance.response_time_p95_achieved
    );
    println!(
        "   P99 response time: {:?}",
        results.slo_compliance.response_time_p99_achieved
    );
    println!("   Resilience score: {:.3}", results.resilience_score);

    // System should handle latency injection gracefully
    assert!(
        results.slo_compliance.availability_achieved >= 0.80,
        "Availability under latency injection too low: {:.1}%",
        results.slo_compliance.availability_achieved * 100.0
    );

    // Response times should be reasonable despite latency injection
    assert!(
        results.slo_compliance.response_time_p95_achieved <= std::time::Duration::from_secs(30),
        "P95 response time too high: {:?}",
        results.slo_compliance.response_time_p95_achieved
    );

    Ok(())
}

/// Comprehensive reliability test suite
#[tokio::test]
async fn test_comprehensive_reliability_suite() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping comprehensive reliability test - AWS credentials not available");
        return Ok(());
    }

    println!("🚀 Running comprehensive reliability test suite...");

    // Run a series of reliability tests
    let test_scenarios = vec![
        (
            "Basic Failure Injection",
            ReliabilityTestConfig::basic_failure_injection(),
        ),
        (
            "Chaos Engineering",
            ReliabilityTestConfig::chaos_engineering(),
        ),
        (
            "Production Readiness",
            ReliabilityTestConfig::production_readiness(),
        ),
    ];

    let mut all_results = Vec::new();

    for (name, config) in test_scenarios {
        println!("\n📊 Running {}", name);
        let executor = ReliabilityTestExecutor::new(config);
        let results = executor.execute().await?;

        println!("✅ {} completed", name);
        println!("   Operations: {}", results.total_operations);
        println!(
            "   Availability: {:.1}%",
            results.slo_compliance.availability_achieved * 100.0
        );
        println!("   Resilience: {:.3}", results.resilience_score);
        println!(
            "   SLO compliance: {:.1}%",
            results.slo_compliance.overall_slo_compliance * 100.0
        );

        all_results.push((name, results));

        // Brief pause between tests
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }

    // Overall assessment
    println!("\n🏁 COMPREHENSIVE RELIABILITY SUMMARY");
    println!("════════════════════════════════════════");

    let mut total_operations = 0;
    let mut total_availability = 0.0;
    let mut total_resilience = 0.0;
    let mut passed_tests = 0;

    for (name, results) in &all_results {
        total_operations += results.total_operations;
        total_availability += results.slo_compliance.availability_achieved;
        total_resilience += results.resilience_score;

        if results.reliability_test_passed {
            passed_tests += 1;
        }

        println!(
            "• {}: {} (resilience: {:.3}, availability: {:.1}%)",
            name,
            if results.reliability_test_passed {
                "✅ PASSED"
            } else {
                "❌ FAILED"
            },
            results.resilience_score,
            results.slo_compliance.availability_achieved * 100.0
        );
    }

    let avg_availability = total_availability / all_results.len() as f64;
    let avg_resilience = total_resilience / all_results.len() as f64;
    let pass_rate = passed_tests as f64 / all_results.len() as f64;

    println!("\nOverall Statistics:");
    println!("  Total operations: {}", total_operations);
    println!("  Average availability: {:.1}%", avg_availability * 100.0);
    println!("  Average resilience: {:.3}", avg_resilience);
    println!("  Test pass rate: {:.1}%", pass_rate * 100.0);

    // Comprehensive reliability validation
    assert!(total_operations > 200, "Insufficient test coverage");
    assert!(
        avg_availability >= 0.85,
        "Average availability too low: {:.1}%",
        avg_availability * 100.0
    );
    assert!(
        avg_resilience >= 0.65,
        "Average resilience too low: {:.3}",
        avg_resilience
    );
    assert!(
        pass_rate >= 0.7,
        "Too many reliability tests failed: {:.1}%",
        pass_rate * 100.0
    );

    if pass_rate >= 0.8 && avg_resilience >= 0.7 && avg_availability >= 0.90 {
        println!("🎉 RELIABILITY SUITE PASSED - SYSTEM IS PRODUCTION READY");
    } else {
        println!("⚠️  Reliability tests show areas for improvement");
    }

    Ok(())
}
