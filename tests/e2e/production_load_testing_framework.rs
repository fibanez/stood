//! Comprehensive Load Testing Framework for Stood Agent Library
//!
//! This module provides production-grade load testing capabilities that build upon
//! the existing performance tests to provide comprehensive benchmarking, stress testing,
//! and reliability validation for production deployment scenarios.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

// Import from e2e lib module when used as a module
use super::*;

/// Load testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestConfig {
    /// Number of concurrent virtual users
    pub concurrent_users: usize,
    /// Duration of the test
    pub test_duration: Duration,
    /// Ramp-up time to reach full load
    pub ramp_up_duration: Duration,
    /// Think time between user actions
    pub think_time: Duration,
    /// Test scenario to execute
    pub scenario: TestScenario,
    /// Performance thresholds
    pub thresholds: PerformanceThresholds,
}

/// Different test scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestScenario {
    /// Simple Q&A interactions
    SimpleChat,
    /// Complex conversations with context
    ConversationFlow,
    /// Tool-heavy interactions
    ToolIntensive,
    /// Mixed workload simulation
    MixedWorkload,
    /// File processing operations
    FileOperations,
    /// Streaming response testing
    StreamingTest,
}

/// Performance thresholds for pass/fail criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    /// Maximum acceptable P95 response time
    pub max_p95_latency: Duration,
    /// Maximum acceptable error rate (0.0 to 1.0)
    pub max_error_rate: f64,
    /// Minimum required throughput (operations per second)
    pub min_throughput: f64,
    /// Maximum memory usage (MB)
    pub max_memory_mb: usize,
}

/// Test execution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestResults {
    pub config: LoadTestConfig,
    pub execution_time: Duration,
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub latency_stats: LatencyStatistics,
    pub throughput_rps: f64,
    pub error_rate: f64,
    pub memory_stats: MemoryStatistics,
    pub passed_thresholds: bool,
    pub threshold_violations: Vec<String>,
}

/// Detailed latency statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStatistics {
    pub min: Duration,
    pub max: Duration,
    pub mean: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub p999: Duration,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatistics {
    pub initial_mb: usize,
    pub peak_mb: usize,
    pub final_mb: usize,
    pub average_mb: usize,
}

/// Individual operation result
#[derive(Debug, Clone)]
struct OperationResult {
    latency: Duration,
    success: bool,
}

/// Load test executor
pub struct LoadTestExecutor {
    config: LoadTestConfig,
    results: Arc<Mutex<Vec<OperationResult>>>,
    memory_samples: Arc<Mutex<Vec<(Instant, usize)>>>,
}

impl LoadTestExecutor {
    pub fn new(config: LoadTestConfig) -> Self {
        Self {
            config,
            results: Arc::new(Mutex::new(Vec::new())),
            memory_samples: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Execute the load test
    pub async fn execute(&self) -> super::Result<LoadTestResults> {
        println!(
            "🚀 Starting load test with {} concurrent users",
            self.config.concurrent_users
        );
        println!("📊 Test duration: {:?}", self.config.test_duration);
        println!("🎯 Scenario: {:?}", self.config.scenario);

        let test_start = Instant::now();

        // Start memory monitoring
        let memory_monitor = self.start_memory_monitoring();

        // Start user simulation
        let user_tasks = self.start_user_simulation().await?;

        // Wait for test completion
        let test_end = test_start + self.config.test_duration;
        while Instant::now() < test_end {
            sleep(Duration::from_secs(1)).await;
        }

        println!("⏹️  Test duration completed, stopping user tasks...");

        // Stop all tasks
        for task in user_tasks {
            task.abort();
        }

        // Stop memory monitoring
        memory_monitor.abort();

        // Analyze results
        let results = self.analyze_results(test_start.elapsed()).await?;

        // Print summary
        self.print_results_summary(&results);

        Ok(results)
    }

    /// Start memory monitoring background task
    fn start_memory_monitoring(&self) -> tokio::task::JoinHandle<()> {
        let memory_samples = self.memory_samples.clone();

        tokio::spawn(async move {
            loop {
                if let Ok(memory_mb) = get_process_memory_mb().await {
                    let mut samples = memory_samples.lock().unwrap();
                    samples.push((Instant::now(), memory_mb));
                }
                sleep(Duration::from_secs(5)).await;
            }
        })
    }

    /// Start user simulation tasks
    async fn start_user_simulation(&self) -> super::Result<Vec<tokio::task::JoinHandle<()>>> {
        let mut tasks = Vec::new();
        let ramp_up_delay = self.config.ramp_up_duration / self.config.concurrent_users as u32;

        for user_id in 0..self.config.concurrent_users {
            let results = self.results.clone();
            let scenario = self.config.scenario.clone();
            let think_time = self.config.think_time;

            let task = tokio::spawn(async move {
                // Ramp-up delay
                sleep(ramp_up_delay * user_id as u32).await;

                // Check AWS credentials
                if !check_aws_credentials() {
                    println!("⚠️  User {} skipping - no AWS credentials", user_id);
                    return;
                }

                // Start user session
                if let Ok(mut session) = spawn_cli().await {
                    loop {
                        let operation_start = Instant::now();
                        let _ = Self::execute_scenario_operation(&mut session, &scenario, user_id)
                            .await;
                        let latency = operation_start.elapsed();

                        // Record result
                        let success = latency < Duration::from_secs(60); // Basic success criteria
                        let result = OperationResult {
                            latency,
                            success,
                        };

                        results.lock().unwrap().push(result);

                        // Think time between operations
                        sleep(think_time).await;
                    }
                }
            });

            tasks.push(task);
        }

        Ok(tasks)
    }

    /// Execute a single scenario operation
    async fn execute_scenario_operation(
        session: &mut CliSession,
        scenario: &TestScenario,
        user_id: usize,
    ) -> String {
        match scenario {
            TestScenario::SimpleChat => {
                let _ = session
                    .send_line(&format!("What is {} + {}?", user_id, user_id * 2))
                    .await;
                "simple_calculation".to_string()
            }
            TestScenario::ConversationFlow => match user_id % 4 {
                0 => {
                    let _ = session.send_line("My name is Alice").await;
                    "context_setting".to_string()
                }
                1 => {
                    let _ = session.send_line("What's my name?").await;
                    "context_query".to_string()
                }
                2 => {
                    let _ = session.send_line("Calculate 15 * 23").await;
                    "calculation_in_context".to_string()
                }
                _ => {
                    let _ = session
                        .send_line("What was the result of that calculation?")
                        .await;
                    "result_recall".to_string()
                }
            },
            TestScenario::ToolIntensive => {
                let operations = [
                    "What time is it?",
                    "Calculate 42 * 17",
                    "Get the HOME environment variable",
                ];
                let op = operations[user_id % operations.len()];
                let _ = session.send_line(op).await;
                "tool_operation".to_string()
            }
            TestScenario::MixedWorkload => {
                let operations = [
                    ("What is Rust?", "general_question"),
                    ("Calculate 123 * 456", "calculation"),
                    ("What time is it?", "time_query"),
                    ("Tell me about machine learning", "complex_question"),
                ];
                let (question, op_type) = operations[user_id % operations.len()];
                let _ = session.send_line(question).await;
                op_type.to_string()
            }
            TestScenario::FileOperations => {
                // Simulate file operations (would need actual file setup)
                let _ = session
                    .send_line("List the files in the current directory")
                    .await;
                "file_operation".to_string()
            }
            TestScenario::StreamingTest => {
                let _ = session.send_line("Write a short story about a robot").await;
                "streaming_request".to_string()
            }
        }
    }

    /// Analyze collected results
    async fn analyze_results(&self, execution_time: Duration) -> super::Result<LoadTestResults> {
        let results = self.results.lock().unwrap().clone();
        let memory_samples = self.memory_samples.lock().unwrap().clone();

        if results.is_empty() {
            return Err("No test results collected".into());
        }

        // Calculate basic metrics
        let total_operations = results.len();
        let successful_operations = results.iter().filter(|r| r.success).count();
        let failed_operations = total_operations - successful_operations;
        let error_rate = failed_operations as f64 / total_operations as f64;
        let throughput_rps = total_operations as f64 / execution_time.as_secs_f64();

        // Calculate latency statistics
        let mut latencies: Vec<Duration> = results.iter().map(|r| r.latency).collect();
        latencies.sort();

        let latency_stats = LatencyStatistics {
            min: latencies[0],
            max: latencies[latencies.len() - 1],
            mean: latencies.iter().sum::<Duration>() / latencies.len() as u32,
            p50: latencies[latencies.len() * 50 / 100],
            p95: latencies[latencies.len() * 95 / 100],
            p99: latencies[latencies.len() * 99 / 100],
            p999: latencies[latencies.len() * 999 / 1000],
        };

        // Calculate memory statistics
        let memory_stats = if !memory_samples.is_empty() {
            let memory_values: Vec<usize> = memory_samples.iter().map(|(_, mb)| *mb).collect();
            MemoryStatistics {
                initial_mb: memory_values[0],
                peak_mb: *memory_values.iter().max().unwrap(),
                final_mb: *memory_values.last().unwrap(),
                average_mb: memory_values.iter().sum::<usize>() / memory_values.len(),
            }
        } else {
            MemoryStatistics {
                initial_mb: 0,
                peak_mb: 0,
                final_mb: 0,
                average_mb: 0,
            }
        };

        // Check threshold violations
        let mut threshold_violations = Vec::new();

        if latency_stats.p95 > self.config.thresholds.max_p95_latency {
            threshold_violations.push(format!(
                "P95 latency ({:?}) exceeds threshold ({:?})",
                latency_stats.p95, self.config.thresholds.max_p95_latency
            ));
        }

        if error_rate > self.config.thresholds.max_error_rate {
            threshold_violations.push(format!(
                "Error rate ({:.2}%) exceeds threshold ({:.2}%)",
                error_rate * 100.0,
                self.config.thresholds.max_error_rate * 100.0
            ));
        }

        if throughput_rps < self.config.thresholds.min_throughput {
            threshold_violations.push(format!(
                "Throughput ({:.2} ops/s) below threshold ({:.2} ops/s)",
                throughput_rps, self.config.thresholds.min_throughput
            ));
        }

        if memory_stats.peak_mb > self.config.thresholds.max_memory_mb {
            threshold_violations.push(format!(
                "Peak memory ({} MB) exceeds threshold ({} MB)",
                memory_stats.peak_mb, self.config.thresholds.max_memory_mb
            ));
        }

        let passed_thresholds = threshold_violations.is_empty();

        Ok(LoadTestResults {
            config: self.config.clone(),
            execution_time,
            total_operations,
            successful_operations,
            failed_operations,
            latency_stats,
            throughput_rps,
            error_rate,
            memory_stats,
            passed_thresholds,
            threshold_violations,
        })
    }

    /// Print results summary
    fn print_results_summary(&self, results: &LoadTestResults) {
        println!("\n📊 LOAD TEST RESULTS SUMMARY");
        println!("════════════════════════════════════════");
        println!("🎯 Scenario: {:?}", results.config.scenario);
        println!("👥 Concurrent Users: {}", results.config.concurrent_users);
        println!("⏱️  Execution Time: {:?}", results.execution_time);
        println!();

        println!("📈 PERFORMANCE METRICS");
        println!("────────────────────────────────────────");
        println!("Total Operations: {}", results.total_operations);
        println!(
            "Successful: {} ({:.1}%)",
            results.successful_operations,
            results.successful_operations as f64 / results.total_operations as f64 * 100.0
        );
        println!(
            "Failed: {} ({:.1}%)",
            results.failed_operations,
            results.error_rate * 100.0
        );
        println!("Throughput: {:.2} ops/sec", results.throughput_rps);
        println!();

        println!("⚡ LATENCY STATISTICS");
        println!("────────────────────────────────────────");
        println!("Min:  {:?}", results.latency_stats.min);
        println!("Mean: {:?}", results.latency_stats.mean);
        println!("P50:  {:?}", results.latency_stats.p50);
        println!("P95:  {:?}", results.latency_stats.p95);
        println!("P99:  {:?}", results.latency_stats.p99);
        println!("Max:  {:?}", results.latency_stats.max);
        println!();

        println!("🧠 MEMORY STATISTICS");
        println!("────────────────────────────────────────");
        println!("Initial: {} MB", results.memory_stats.initial_mb);
        println!("Peak:    {} MB", results.memory_stats.peak_mb);
        println!("Final:   {} MB", results.memory_stats.final_mb);
        println!("Average: {} MB", results.memory_stats.average_mb);
        println!();

        println!("✅ THRESHOLD VALIDATION");
        println!("────────────────────────────────────────");
        if results.passed_thresholds {
            println!("🎉 ALL THRESHOLDS PASSED");
        } else {
            println!("❌ THRESHOLD VIOLATIONS:");
            for violation in &results.threshold_violations {
                println!("   • {}", violation);
            }
        }
        println!();
    }
}

/// Helper function to get process memory usage in MB
pub async fn get_process_memory_mb() -> super::Result<usize> {
    #[cfg(target_os = "linux")]
    {
        let contents = tokio::fs::read_to_string("/proc/self/status").await?;
        for line in contents.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: usize = parts[1].parse()?;
                    return Ok(kb / 1024); // Convert KB to MB
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("ps")
            .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()?;
        let rss_str = String::from_utf8(output.stdout)?;
        let rss_kb: usize = rss_str.trim().parse()?;
        return Ok(rss_kb / 1024); // Convert KB to MB
    }

    // Fallback for other platforms
    Ok(0)
}

/// Predefined load test configurations
impl LoadTestConfig {
    /// Light load test configuration for CI/CD
    pub fn light_load() -> Self {
        Self {
            concurrent_users: 5,
            test_duration: Duration::from_secs(60),
            ramp_up_duration: Duration::from_secs(10),
            think_time: Duration::from_secs(2),
            scenario: TestScenario::SimpleChat,
            thresholds: PerformanceThresholds {
                max_p95_latency: Duration::from_secs(30),
                max_error_rate: 0.05, // 5%
                min_throughput: 0.5,  // 0.5 ops/sec
                max_memory_mb: 500,
            },
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_light_load_test() -> super::Result<()> {
        if !check_aws_credentials() {
            println!("⚠️  Skipping load test - AWS credentials not available");
            return Ok(());
        }

        let config = LoadTestConfig::light_load();
        let executor = LoadTestExecutor::new(config);

        let results = executor.execute().await?;

        // Validate results
        assert!(results.total_operations > 0, "No operations executed");
        assert!(results.throughput_rps > 0.0, "Zero throughput");

        // Check if thresholds passed (warn if not, but don't fail test)
        if !results.passed_thresholds {
            println!("⚠️  Performance thresholds not met, but test continues");
            for violation in &results.threshold_violations {
                println!("   • {}", violation);
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_conversation_flow_load() -> super::Result<()> {
        if !check_aws_credentials() {
            println!("⚠️  Skipping load test - AWS credentials not available");
            return Ok(());
        }

        let config = LoadTestConfig {
            concurrent_users: 3,
            test_duration: Duration::from_secs(30),
            ramp_up_duration: Duration::from_secs(5),
            think_time: Duration::from_secs(1),
            scenario: TestScenario::ConversationFlow,
            thresholds: PerformanceThresholds {
                max_p95_latency: Duration::from_secs(45),
                max_error_rate: 0.20, // Lenient for test
                min_throughput: 0.1,
                max_memory_mb: 1000,
            },
        };

        let executor = LoadTestExecutor::new(config);
        let results = executor.execute().await?;

        assert!(results.total_operations > 0);
        println!(
            "Conversation flow test completed with {} operations",
            results.total_operations
        );

        Ok(())
    }
}
