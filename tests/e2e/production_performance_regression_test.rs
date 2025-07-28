//! Performance regression testing suite for the Stood agent library
//!
//! This module implements automated performance regression tests to ensure
//! consistent performance across code changes and detect performance degradations
//! before they reach production.

use super::*;
use crate::performance_regression_framework::*;

/// Test basic performance regression detection
#[tokio::test]
async fn test_basic_performance_regression() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping performance regression test - AWS credentials not available");
        return Ok(());
    }

    let config = PerformanceRegressionConfig::basic_regression_testing();
    let mut executor = PerformanceRegressionExecutor::new(config);

    let results = executor.execute().await?;

    // Validate basic regression testing
    assert!(
        results.benchmark_results.len() > 0,
        "No performance benchmarks executed"
    );
    assert!(
        results.performance_summary.overall_performance_score >= 0.0,
        "Invalid performance score"
    );

    println!("📈 Basic Performance Regression Test Results:");
    println!(
        "   Benchmarks executed: {}",
        results.benchmark_results.len()
    );
    println!(
        "   Overall performance score: {:.3}",
        results.performance_summary.overall_performance_score
    );
    println!(
        "   Performance grade: {:?}",
        results.performance_summary.performance_grade
    );
    println!(
        "   Regression detected: {}",
        if results.regression_detected {
            "❌"
        } else {
            "✅"
        }
    );

    // Basic performance validation
    assert!(
        results.performance_summary.overall_performance_score >= 0.5,
        "Performance score too low: {:.3}",
        results.performance_summary.overall_performance_score
    );

    // Check benchmark results
    let passed_benchmarks = results
        .benchmark_results
        .iter()
        .filter(|r| r.threshold_met)
        .count();
    assert!(
        passed_benchmarks > 0,
        "No benchmarks passed performance thresholds"
    );

    println!(
        "   Benchmarks passed: {}/{}",
        passed_benchmarks,
        results.benchmark_results.len()
    );

    Ok(())
}

/// Test response time regression detection
#[tokio::test]
async fn test_response_time_regression() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping performance regression test - AWS credentials not available");
        return Ok(());
    }

    let config = PerformanceRegressionConfig {
        test_duration: std::time::Duration::from_secs(60),
        iterations: 8,
        benchmarks: vec![
            PerformanceBenchmark::ResponseTime {
                scenario: "quick_calculation".to_string(),
                target_latency: std::time::Duration::from_secs(8),
            },
            PerformanceBenchmark::ResponseTime {
                scenario: "conversation_response".to_string(),
                target_latency: std::time::Duration::from_secs(12),
            },
        ],
        regression_detection: RegressionDetectionConfig {
            response_time_threshold: 0.20,
            throughput_threshold: 0.15,
            memory_threshold: 0.25,
            error_rate_threshold: 0.10,
            confidence_level: 0.85,
            min_samples: 5,
        },
        baseline_management: BaselineManagementConfig {
            baseline_dir: "test_baselines".to_string(),
            auto_update_baseline: false,
            retention_days: 7,
            version_tracking: false,
        },
        performance_thresholds: PerformanceThresholds {
            max_p95_response_time: std::time::Duration::from_secs(15),
            min_throughput_rps: 0.5,
            max_memory_usage_mb: 800,
            max_error_rate: 0.15,
        },
    };

    let mut executor = PerformanceRegressionExecutor::new(config);
    let results = executor.execute().await?;

    // Validate response time regression testing
    assert!(
        results.benchmark_results.len() == 2,
        "Expected 2 response time benchmarks"
    );

    println!("⚡ Response Time Regression Test Results:");
    for (i, result) in results.benchmark_results.iter().enumerate() {
        println!("   Benchmark {}: {:?}", i + 1, result.benchmark);
        println!(
            "     P95 Response Time: {:?}",
            result.metrics.response_times.p95
        );
        println!(
            "     Mean Response Time: {:?}",
            result.metrics.response_times.mean
        );
        println!(
            "     Threshold Met: {}",
            if result.threshold_met { "✅" } else { "❌" }
        );
        println!(
            "     Regression Detected: {}",
            if result.regression_detected {
                "❌"
            } else {
                "✅"
            }
        );
    }

    // Response time validation
    for result in &results.benchmark_results {
        assert!(
            result.metrics.response_times.p95 <= std::time::Duration::from_secs(30),
            "P95 response time too high: {:?}",
            result.metrics.response_times.p95
        );
        assert!(
            result.iterations_completed > 0,
            "No iterations completed for benchmark"
        );
    }

    Ok(())
}

/// Test memory usage regression detection
#[tokio::test]
async fn test_memory_usage_regression() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping performance regression test - AWS credentials not available");
        return Ok(());
    }

    let config = PerformanceRegressionConfig {
        test_duration: std::time::Duration::from_secs(90),
        iterations: 6,
        benchmarks: vec![
            PerformanceBenchmark::MemoryUsage {
                scenario: "basic_memory_test".to_string(),
                max_memory_mb: 600,
            },
            PerformanceBenchmark::MemoryUsage {
                scenario: "large_context_memory".to_string(),
                max_memory_mb: 1000,
            },
        ],
        regression_detection: RegressionDetectionConfig {
            response_time_threshold: 0.15,
            throughput_threshold: 0.15,
            memory_threshold: 0.20,
            error_rate_threshold: 0.10,
            confidence_level: 0.90,
            min_samples: 3,
        },
        baseline_management: BaselineManagementConfig {
            baseline_dir: "memory_baselines".to_string(),
            auto_update_baseline: false,
            retention_days: 14,
            version_tracking: false,
        },
        performance_thresholds: PerformanceThresholds {
            max_p95_response_time: std::time::Duration::from_secs(20),
            min_throughput_rps: 0.3,
            max_memory_usage_mb: 1200,
            max_error_rate: 0.12,
        },
    };

    let mut executor = PerformanceRegressionExecutor::new(config);
    let results = executor.execute().await?;

    // Validate memory regression testing
    assert!(
        results.benchmark_results.len() == 2,
        "Expected 2 memory usage benchmarks"
    );

    println!("🧠 Memory Usage Regression Test Results:");
    for (i, result) in results.benchmark_results.iter().enumerate() {
        println!("   Benchmark {}: {:?}", i + 1, result.benchmark);
        println!(
            "     Peak Memory: {} MB",
            result.metrics.memory_metrics.peak_memory_mb
        );
        println!(
            "     Average Memory: {} MB",
            result.metrics.memory_metrics.average_memory_mb
        );
        println!(
            "     Memory Growth Rate: {:.2}%",
            result.metrics.memory_metrics.memory_growth_rate * 100.0
        );
        println!(
            "     Threshold Met: {}",
            if result.threshold_met { "✅" } else { "❌" }
        );
    }

    // Memory validation
    for result in &results.benchmark_results {
        assert!(
            result.metrics.memory_metrics.peak_memory_mb < 2000,
            "Peak memory usage too high: {} MB",
            result.metrics.memory_metrics.peak_memory_mb
        );
    }

    Ok(())
}

/// Test throughput regression detection
#[tokio::test]
async fn test_throughput_regression() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping performance regression test - AWS credentials not available");
        return Ok(());
    }

    let config = PerformanceRegressionConfig {
        test_duration: std::time::Duration::from_secs(120),
        iterations: 15,
        benchmarks: vec![
            PerformanceBenchmark::Throughput {
                scenario: "rapid_requests".to_string(),
                target_rps: 1.0,
            },
            PerformanceBenchmark::Throughput {
                scenario: "sustained_load".to_string(),
                target_rps: 0.8,
            },
        ],
        regression_detection: RegressionDetectionConfig {
            response_time_threshold: 0.15,
            throughput_threshold: 0.12,
            memory_threshold: 0.20,
            error_rate_threshold: 0.08,
            confidence_level: 0.92,
            min_samples: 8,
        },
        baseline_management: BaselineManagementConfig {
            baseline_dir: "throughput_baselines".to_string(),
            auto_update_baseline: false,
            retention_days: 21,
            version_tracking: false,
        },
        performance_thresholds: PerformanceThresholds {
            max_p95_response_time: std::time::Duration::from_secs(18),
            min_throughput_rps: 0.6,
            max_memory_usage_mb: 700,
            max_error_rate: 0.10,
        },
    };

    let mut executor = PerformanceRegressionExecutor::new(config);
    let results = executor.execute().await?;

    // Validate throughput regression testing
    assert!(
        results.benchmark_results.len() == 2,
        "Expected 2 throughput benchmarks"
    );

    println!("🚀 Throughput Regression Test Results:");
    for (i, result) in results.benchmark_results.iter().enumerate() {
        println!("   Benchmark {}: {:?}", i + 1, result.benchmark);
        println!(
            "     Requests per Second: {:.2}",
            result.metrics.throughput_metrics.requests_per_second
        );
        println!(
            "     Average RPS: {:.2}",
            result.metrics.throughput_metrics.average_rps
        );
        println!(
            "     Peak RPS: {:.2}",
            result.metrics.throughput_metrics.peak_rps
        );
        println!(
            "     Consistency: {:.1}%",
            result.metrics.throughput_metrics.throughput_consistency * 100.0
        );
        println!(
            "     Threshold Met: {}",
            if result.threshold_met { "✅" } else { "❌" }
        );
    }

    // Throughput validation
    for result in &results.benchmark_results {
        assert!(
            result.metrics.throughput_metrics.requests_per_second >= 0.0,
            "Invalid throughput measurement"
        );
    }

    Ok(())
}

/// Test error rate regression detection
#[tokio::test]
async fn test_error_rate_regression() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping performance regression test - AWS credentials not available");
        return Ok(());
    }

    let config = PerformanceRegressionConfig {
        test_duration: std::time::Duration::from_secs(100),
        iterations: 12,
        benchmarks: vec![
            PerformanceBenchmark::ErrorRate {
                scenario: "error_resilience".to_string(),
                max_error_rate: 0.05,
            },
            PerformanceBenchmark::ErrorRate {
                scenario: "stress_error_rate".to_string(),
                max_error_rate: 0.10,
            },
        ],
        regression_detection: RegressionDetectionConfig {
            response_time_threshold: 0.18,
            throughput_threshold: 0.15,
            memory_threshold: 0.22,
            error_rate_threshold: 0.03,
            confidence_level: 0.88,
            min_samples: 6,
        },
        baseline_management: BaselineManagementConfig {
            baseline_dir: "error_rate_baselines".to_string(),
            auto_update_baseline: false,
            retention_days: 10,
            version_tracking: false,
        },
        performance_thresholds: PerformanceThresholds {
            max_p95_response_time: std::time::Duration::from_secs(25),
            min_throughput_rps: 0.4,
            max_memory_usage_mb: 900,
            max_error_rate: 0.12,
        },
    };

    let mut executor = PerformanceRegressionExecutor::new(config);
    let results = executor.execute().await?;

    // Validate error rate regression testing
    assert!(
        results.benchmark_results.len() == 2,
        "Expected 2 error rate benchmarks"
    );

    println!("🚨 Error Rate Regression Test Results:");
    for (i, result) in results.benchmark_results.iter().enumerate() {
        println!("   Benchmark {}: {:?}", i + 1, result.benchmark);
        println!(
            "     Total Requests: {}",
            result.metrics.error_metrics.total_requests
        );
        println!(
            "     Successful: {}",
            result.metrics.error_metrics.successful_requests
        );
        println!(
            "     Failed: {}",
            result.metrics.error_metrics.failed_requests
        );
        println!(
            "     Error Rate: {:.2}%",
            result.metrics.error_metrics.error_rate * 100.0
        );
        println!(
            "     Threshold Met: {}",
            if result.threshold_met { "✅" } else { "❌" }
        );
    }

    // Error rate validation
    for result in &results.benchmark_results {
        assert!(
            result.metrics.error_metrics.error_rate <= 0.20,
            "Error rate too high: {:.1}%",
            result.metrics.error_metrics.error_rate * 100.0
        );
        assert!(
            result.metrics.error_metrics.total_requests > 0,
            "No requests recorded"
        );
    }

    Ok(())
}

/// Test concurrent load regression detection
#[tokio::test]
async fn test_concurrent_load_regression() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping performance regression test - AWS credentials not available");
        return Ok(());
    }

    let config = PerformanceRegressionConfig {
        test_duration: std::time::Duration::from_secs(150),
        iterations: 10,
        benchmarks: vec![
            PerformanceBenchmark::ConcurrentLoad {
                users: 3,
                target_latency: std::time::Duration::from_secs(15),
            },
            PerformanceBenchmark::ConcurrentLoad {
                users: 5,
                target_latency: std::time::Duration::from_secs(20),
            },
        ],
        regression_detection: RegressionDetectionConfig {
            response_time_threshold: 0.25,
            throughput_threshold: 0.20,
            memory_threshold: 0.30,
            error_rate_threshold: 0.15,
            confidence_level: 0.85,
            min_samples: 5,
        },
        baseline_management: BaselineManagementConfig {
            baseline_dir: "concurrent_baselines".to_string(),
            auto_update_baseline: false,
            retention_days: 7,
            version_tracking: false,
        },
        performance_thresholds: PerformanceThresholds {
            max_p95_response_time: std::time::Duration::from_secs(30),
            min_throughput_rps: 0.2,
            max_memory_usage_mb: 1500,
            max_error_rate: 0.20,
        },
    };

    let mut executor = PerformanceRegressionExecutor::new(config);
    let results = executor.execute().await?;

    // Validate concurrent load regression testing
    assert!(
        results.benchmark_results.len() == 2,
        "Expected 2 concurrent load benchmarks"
    );

    println!("👥 Concurrent Load Regression Test Results:");
    for (i, result) in results.benchmark_results.iter().enumerate() {
        println!("   Benchmark {}: {:?}", i + 1, result.benchmark);
        println!(
            "     P95 Response Time: {:?}",
            result.metrics.response_times.p95
        );
        println!(
            "     Throughput: {:.2} rps",
            result.metrics.throughput_metrics.requests_per_second
        );
        println!(
            "     Peak Memory: {} MB",
            result.metrics.memory_metrics.peak_memory_mb
        );
        println!(
            "     Error Rate: {:.2}%",
            result.metrics.error_metrics.error_rate * 100.0
        );
        println!(
            "     Threshold Met: {}",
            if result.threshold_met { "✅" } else { "❌" }
        );
    }

    // Concurrent load validation
    for result in &results.benchmark_results {
        assert!(
            result.metrics.response_times.p95 <= std::time::Duration::from_secs(45),
            "P95 response time under load too high: {:?}",
            result.metrics.response_times.p95
        );
    }

    Ok(())
}

/// Test comprehensive performance regression suite
#[tokio::test]
async fn test_comprehensive_performance_regression() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping comprehensive performance regression test - AWS credentials not available");
        return Ok(());
    }

    let config = PerformanceRegressionConfig::comprehensive_regression_testing();
    let mut executor = PerformanceRegressionExecutor::new(config);

    let results = executor.execute().await?;

    // Validate comprehensive regression testing
    assert!(
        results.benchmark_results.len() > 0,
        "No performance benchmarks executed"
    );

    println!("\n🏁 COMPREHENSIVE PERFORMANCE REGRESSION SUMMARY");
    println!("════════════════════════════════════════");
    println!("📊 Benchmarks: {}", results.benchmark_results.len());
    println!("⏱️  Execution Time: {:?}", results.execution_time);
    println!(
        "🎯 Overall Performance Score: {:.3}",
        results.performance_summary.overall_performance_score
    );
    println!(
        "🏆 Performance Grade: {:?}",
        results.performance_summary.performance_grade
    );
    println!(
        "📈 Regression Detected: {}",
        if results.regression_detected {
            "❌"
        } else {
            "✅"
        }
    );
    println!();

    println!("📋 BENCHMARK DETAILS");
    println!("────────────────────────────────────────");
    let passed_benchmarks = results
        .benchmark_results
        .iter()
        .filter(|r| r.threshold_met)
        .count();
    let regressed_benchmarks = results
        .benchmark_results
        .iter()
        .filter(|r| r.regression_detected)
        .count();

    for (i, result) in results.benchmark_results.iter().enumerate() {
        println!("{}. {:?}", i + 1, result.benchmark);
        println!(
            "   ✅ Threshold: {} | 📊 Regression: {}",
            if result.threshold_met { "PASS" } else { "FAIL" },
            if result.regression_detected {
                "DETECTED"
            } else {
                "NONE"
            }
        );
    }

    println!("\n📊 PERFORMANCE ANALYSIS");
    println!("────────────────────────────────────────");
    println!(
        "Benchmarks Passed: {}/{} ({:.1}%)",
        passed_benchmarks,
        results.benchmark_results.len(),
        passed_benchmarks as f64 / results.benchmark_results.len() as f64 * 100.0
    );
    println!(
        "Regressions Detected: {}/{}",
        regressed_benchmarks,
        results.benchmark_results.len()
    );
    println!(
        "Performance Trend: {:?}",
        results.regression_analysis.trend_analysis.performance_trend
    );
    println!(
        "Statistical Significance: {:.1}%",
        results.regression_analysis.statistical_significance * 100.0
    );

    println!("\n💡 INSIGHTS");
    println!("────────────────────────────────────────");
    for insight in &results.performance_summary.performance_insights {
        println!("• {}", insight);
    }

    println!("\n📝 RECOMMENDATIONS");
    println!("────────────────────────────────────────");
    for recommendation in &results.performance_summary.optimization_recommendations {
        println!("• {}", recommendation);
    }

    // Comprehensive validation
    let pass_rate = passed_benchmarks as f64 / results.benchmark_results.len() as f64;
    assert!(
        pass_rate >= 0.7,
        "Too many benchmarks failed: {:.1}%",
        pass_rate * 100.0
    );
    assert!(
        results.performance_summary.overall_performance_score >= 0.6,
        "Overall performance score too low: {:.3}",
        results.performance_summary.overall_performance_score
    );

    // Check for critical regressions
    if !results.critical_regressions.is_empty() {
        println!("\n❌ CRITICAL REGRESSIONS DETECTED:");
        for regression in &results.critical_regressions {
            println!("   • {}", regression);
        }
        panic!("Critical performance regressions detected");
    }

    println!("\n🎉 COMPREHENSIVE PERFORMANCE REGRESSION TESTING COMPLETED");
    if results.regression_detected {
        println!("⚠️  Performance regressions detected - review recommended");
    } else {
        println!("✅ No significant performance regressions detected");
    }

    Ok(())
}

/// Performance regression benchmark validation
#[tokio::test]
async fn test_performance_benchmark_validation() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping performance benchmark validation - AWS credentials not available");
        return Ok(());
    }

    // Test individual benchmark types
    let benchmark_configs = vec![
        (
            "Response Time",
            PerformanceBenchmark::ResponseTime {
                scenario: "validation_test".to_string(),
                target_latency: std::time::Duration::from_secs(10),
            },
        ),
        (
            "Memory Usage",
            PerformanceBenchmark::MemoryUsage {
                scenario: "memory_validation".to_string(),
                max_memory_mb: 400,
            },
        ),
        (
            "Error Rate",
            PerformanceBenchmark::ErrorRate {
                scenario: "error_validation".to_string(),
                max_error_rate: 0.08,
            },
        ),
    ];

    for (name, benchmark) in benchmark_configs {
        println!("🧪 Validating {} benchmark...", name);

        let config = PerformanceRegressionConfig {
            test_duration: std::time::Duration::from_secs(30),
            iterations: 3,
            benchmarks: vec![benchmark],
            regression_detection: RegressionDetectionConfig {
                response_time_threshold: 0.30,
                throughput_threshold: 0.30,
                memory_threshold: 0.40,
                error_rate_threshold: 0.20,
                confidence_level: 0.80,
                min_samples: 2,
            },
            baseline_management: BaselineManagementConfig {
                baseline_dir: "validation_baselines".to_string(),
                auto_update_baseline: false,
                retention_days: 1,
                version_tracking: false,
            },
            performance_thresholds: PerformanceThresholds {
                max_p95_response_time: std::time::Duration::from_secs(20),
                min_throughput_rps: 0.1,
                max_memory_usage_mb: 1000,
                max_error_rate: 0.25,
            },
        };

        let mut executor = PerformanceRegressionExecutor::new(config);
        let results = executor.execute().await?;

        assert!(
            results.benchmark_results.len() == 1,
            "Expected 1 benchmark result for {}",
            name
        );
        let result = &results.benchmark_results[0];

        println!(
            "   ✅ {} benchmark completed: {} iterations",
            name, result.iterations_completed
        );
        assert!(
            result.iterations_completed > 0,
            "No iterations completed for {} benchmark",
            name
        );
    }

    println!("🎉 All benchmark validations completed successfully");

    Ok(())
}
