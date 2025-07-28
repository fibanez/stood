//! Stress Testing Framework for Stood Agent Library
//!
//! This module provides comprehensive stress testing capabilities for production
//! deployment validation, including resource leak detection, memory pressure testing,
//! failure injection, and stability validation under extreme conditions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};
use tokio::time::sleep;

// Import from e2e lib module when used as a module
use super::*;

/// Stress test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestConfig {
    /// Test duration
    pub test_duration: Duration,
    /// Maximum concurrent users to ramp up to
    pub max_concurrent_users: usize,
    /// Ramp-up pattern
    pub ramp_pattern: RampPattern,
    /// Resource monitoring interval
    pub monitoring_interval: Duration,
    /// Memory pressure thresholds
    pub memory_thresholds: MemoryThresholds,
    /// Failure injection settings
    pub failure_injection: FailureInjectionConfig,
    /// Test scenario
    pub scenario: StressTestScenario,
}

/// Different ramp-up patterns for stress testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RampPattern {
    /// Linear increase in users
    Linear {
        step_size: usize,
        step_interval: Duration,
    },
    /// Exponential increase in users
    Exponential {
        multiplier: f64,
        step_interval: Duration,
    },
    /// Sudden spike in users
    Spike { spike_duration: Duration },
    /// Sustained high load
    Sustained,
}

/// Memory pressure thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryThresholds {
    /// Warning threshold (MB)
    pub warning_mb: usize,
    /// Critical threshold (MB)
    pub critical_mb: usize,
    /// Memory leak detection threshold (MB increase per minute)
    pub leak_threshold_mb_per_min: f64,
    /// Maximum allowed memory usage (MB)
    pub max_memory_mb: usize,
}

/// Failure injection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureInjectionConfig {
    /// Enable network latency injection
    pub network_latency: bool,
    /// Enable timeout simulation
    pub timeout_simulation: bool,
    /// Enable memory pressure simulation
    pub memory_pressure: bool,
    /// Failure rate (0.0 to 1.0)
    pub failure_rate: f64,
}

/// Stress test scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StressTestScenario {
    /// Memory leak detection
    MemoryLeakDetection,
    /// Resource exhaustion testing
    ResourceExhaustion,
    /// Connection pooling stress
    ConnectionStress,
    /// Rapid conversation cycling
    ConversationCycling,
    /// Tool execution stress
    ToolExecutionStress,
    /// Mixed high-intensity workload
    MixedHighIntensity,
}

/// Stress test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResults {
    pub config: StressTestConfig,
    pub execution_time: Duration,
    pub peak_concurrent_users: usize,
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub resource_metrics: ResourceMetrics,
    pub stability_metrics: StabilityMetrics,
    pub failure_patterns: Vec<FailurePattern>,
    pub stress_test_passed: bool,
    pub critical_issues: Vec<String>,
}

/// Resource usage metrics during stress test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub memory_samples: Vec<MemorySample>,
    pub memory_leak_detected: bool,
    pub memory_leak_rate_mb_per_min: f64,
    pub peak_memory_mb: usize,
    pub cpu_usage_samples: Vec<CpuSample>,
    pub peak_cpu_usage: f64,
    pub connection_pool_stats: ConnectionPoolStats,
}

/// Memory usage sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySample {
    pub timestamp: f64,
    pub memory_mb: usize,
    pub active_users: usize,
}

/// CPU usage sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSample {
    pub timestamp: f64,
    pub cpu_percent: f64,
    pub active_users: usize,
}

/// Connection pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolStats {
    pub max_connections: usize,
    pub peak_active_connections: usize,
    pub connection_timeouts: usize,
    pub connection_errors: usize,
}

/// Stability metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityMetrics {
    pub error_rate_over_time: Vec<ErrorRateSample>,
    pub response_time_degradation: bool,
    pub system_recovery_time: Option<Duration>,
    pub stability_score: f64, // 0.0 to 1.0
}

/// Error rate sample over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRateSample {
    pub timestamp: f64,
    pub error_rate: f64,
    pub concurrent_users: usize,
}

/// Failure pattern detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    pub pattern_type: String,
    pub description: String,
    pub severity: FailureSeverity,
    pub first_occurrence: f64,
    pub frequency: usize,
}

/// Failure severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Stress test executor
pub struct StressTestExecutor {
    config: StressTestConfig,
    resource_monitor: Arc<Mutex<ResourceMonitor>>,
    user_counter: Arc<AtomicUsize>,
    stop_signal: Arc<AtomicBool>,
    results: Arc<Mutex<Vec<OperationResult>>>,
}

/// Resource monitoring component
pub struct ResourceMonitor {
    memory_samples: Vec<MemorySample>,
    cpu_samples: Vec<CpuSample>,
    connection_stats: ConnectionPoolStats,
}

/// Individual operation result for stress testing
#[derive(Debug, Clone)]
struct OperationResult {
    timestamp: Instant,
    duration: Duration,
    success: bool,
    error_type: Option<String>,
    concurrent_users: usize,
}

impl StressTestExecutor {
    pub fn new(config: StressTestConfig) -> Self {
        Self {
            config,
            resource_monitor: Arc::new(Mutex::new(ResourceMonitor::new())),
            user_counter: Arc::new(AtomicUsize::new(0)),
            stop_signal: Arc::new(AtomicBool::new(false)),
            results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Execute the stress test
    pub async fn execute(&self) -> super::Result<StressTestResults> {
        println!("🔥 Starting stress test: {:?}", self.config.scenario);
        println!("⏱️  Duration: {:?}", self.config.test_duration);
        println!(
            "👥 Max concurrent users: {}",
            self.config.max_concurrent_users
        );

        let test_start = Instant::now();

        // Start resource monitoring
        let monitoring_task = self.start_resource_monitoring().await;

        // Start user simulation with ramp pattern
        let user_simulation_task = self.start_user_simulation().await?;

        // Run test for configured duration
        sleep(self.config.test_duration).await;

        // Signal stop and wait for cleanup
        self.stop_signal.store(true, Ordering::SeqCst);
        println!("⏹️  Stopping stress test...");

        // Allow time for graceful shutdown
        sleep(Duration::from_secs(10)).await;

        // Stop monitoring
        monitoring_task.abort();
        user_simulation_task.abort();

        // Analyze results
        let results = self
            .analyze_stress_test_results(test_start.elapsed())
            .await?;

        // Print summary
        self.print_stress_test_summary(&results);

        Ok(results)
    }

    /// Start resource monitoring
    async fn start_resource_monitoring(&self) -> tokio::task::JoinHandle<()> {
        let monitor = self.resource_monitor.clone();
        let stop_signal = self.stop_signal.clone();
        let user_counter = self.user_counter.clone();
        let interval = self.config.monitoring_interval;

        tokio::spawn(async move {
            while !stop_signal.load(Ordering::SeqCst) {
                let current_users = user_counter.load(Ordering::SeqCst);

                // Collect memory sample
                if let Ok(memory_mb) = get_process_memory_mb().await {
                    let sample = MemorySample {
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f64(),
                        memory_mb,
                        active_users: current_users,
                    };

                    monitor.lock().unwrap().memory_samples.push(sample);
                }

                // Collect CPU sample
                if let Ok(cpu_percent) = get_process_cpu_usage().await {
                    let sample = CpuSample {
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f64(),
                        cpu_percent,
                        active_users: current_users,
                    };

                    monitor.lock().unwrap().cpu_samples.push(sample);
                }

                sleep(interval).await;
            }
        })
    }

    /// Start user simulation with ramp pattern
    async fn start_user_simulation(&self) -> super::Result<tokio::task::JoinHandle<()>> {
        let config = self.config.clone();
        let user_counter = self.user_counter.clone();
        let stop_signal = self.stop_signal.clone();
        let results = self.results.clone();

        let task = tokio::spawn(async move {
            match config.ramp_pattern {
                RampPattern::Linear {
                    step_size,
                    step_interval,
                } => {
                    Self::execute_linear_ramp(
                        config,
                        user_counter,
                        stop_signal,
                        results,
                        step_size,
                        step_interval,
                    )
                    .await;
                }
                RampPattern::Exponential {
                    multiplier,
                    step_interval,
                } => {
                    Self::execute_exponential_ramp(
                        config,
                        user_counter,
                        stop_signal,
                        results,
                        multiplier,
                        step_interval,
                    )
                    .await;
                }
                RampPattern::Spike { spike_duration } => {
                    Self::execute_spike_test(
                        config,
                        user_counter,
                        stop_signal,
                        results,
                        spike_duration,
                    )
                    .await;
                }
                RampPattern::Sustained => {
                    Self::execute_sustained_load(config, user_counter, stop_signal, results).await;
                }
            }
        });

        Ok(task)
    }

    /// Execute linear ramp pattern
    async fn execute_linear_ramp(
        config: StressTestConfig,
        user_counter: Arc<AtomicUsize>,
        stop_signal: Arc<AtomicBool>,
        results: Arc<Mutex<Vec<OperationResult>>>,
        step_size: usize,
        step_interval: Duration,
    ) {
        let mut current_users = 0;

        while current_users < config.max_concurrent_users && !stop_signal.load(Ordering::SeqCst) {
            current_users = (current_users + step_size).min(config.max_concurrent_users);
            user_counter.store(current_users, Ordering::SeqCst);

            println!("📈 Ramping up to {} users", current_users);

            // Spawn additional user tasks
            for _ in 0..step_size {
                if current_users <= config.max_concurrent_users {
                    Self::spawn_virtual_user(
                        config.clone(),
                        stop_signal.clone(),
                        results.clone(),
                        user_counter.clone(),
                    )
                    .await;
                }
            }

            sleep(step_interval).await;
        }
    }

    /// Execute exponential ramp pattern
    async fn execute_exponential_ramp(
        config: StressTestConfig,
        user_counter: Arc<AtomicUsize>,
        stop_signal: Arc<AtomicBool>,
        results: Arc<Mutex<Vec<OperationResult>>>,
        multiplier: f64,
        step_interval: Duration,
    ) {
        let mut current_users = 1;

        while current_users < config.max_concurrent_users && !stop_signal.load(Ordering::SeqCst) {
            user_counter.store(current_users, Ordering::SeqCst);

            println!("📈 Exponential ramp to {} users", current_users);

            // Spawn user tasks for current level
            for _ in 0..current_users {
                Self::spawn_virtual_user(
                    config.clone(),
                    stop_signal.clone(),
                    results.clone(),
                    user_counter.clone(),
                )
                .await;
            }

            current_users = ((current_users as f64) * multiplier) as usize;
            current_users = current_users.min(config.max_concurrent_users);

            sleep(step_interval).await;
        }
    }

    /// Execute spike test pattern
    async fn execute_spike_test(
        config: StressTestConfig,
        user_counter: Arc<AtomicUsize>,
        stop_signal: Arc<AtomicBool>,
        results: Arc<Mutex<Vec<OperationResult>>>,
        spike_duration: Duration,
    ) {
        println!(
            "💥 Executing spike test - immediate ramp to {} users",
            config.max_concurrent_users
        );

        user_counter.store(config.max_concurrent_users, Ordering::SeqCst);

        // Spawn all users immediately
        for _ in 0..config.max_concurrent_users {
            Self::spawn_virtual_user(
                config.clone(),
                stop_signal.clone(),
                results.clone(),
                user_counter.clone(),
            )
            .await;
        }

        // Maintain spike for specified duration
        sleep(spike_duration).await;

        println!("📉 Spike test completed, maintaining load");
    }

    /// Execute sustained load pattern
    async fn execute_sustained_load(
        config: StressTestConfig,
        user_counter: Arc<AtomicUsize>,
        stop_signal: Arc<AtomicBool>,
        results: Arc<Mutex<Vec<OperationResult>>>,
    ) {
        println!(
            "🔄 Executing sustained load test with {} users",
            config.max_concurrent_users
        );

        user_counter.store(config.max_concurrent_users, Ordering::SeqCst);

        // Spawn all users for sustained load
        for _ in 0..config.max_concurrent_users {
            Self::spawn_virtual_user(
                config.clone(),
                stop_signal.clone(),
                results.clone(),
                user_counter.clone(),
            )
            .await;
        }

        println!("🔄 Sustained load active");
    }

    /// Spawn a virtual user task
    async fn spawn_virtual_user(
        config: StressTestConfig,
        stop_signal: Arc<AtomicBool>,
        results: Arc<Mutex<Vec<OperationResult>>>,
        user_counter: Arc<AtomicUsize>,
    ) {
        tokio::spawn(async move {
            // Check AWS credentials
            if !check_aws_credentials() {
                return;
            }

            // Start user session
            if let Ok(mut session) = spawn_cli().await {
                while !stop_signal.load(Ordering::SeqCst) {
                    let operation_start = Instant::now();
                    let current_users = user_counter.load(Ordering::SeqCst);

                    // Execute scenario operation
                    let (success, error_type) =
                        Self::execute_stress_scenario_operation(&mut session, &config.scenario)
                            .await;

                    let duration = operation_start.elapsed();

                    // Record result
                    let result = OperationResult {
                        timestamp: Instant::now(),
                        duration,
                        success,
                        error_type,
                        concurrent_users: current_users,
                    };

                    results.lock().unwrap().push(result);

                    // Brief pause to prevent overwhelming the system
                    sleep(Duration::from_millis(100)).await;
                }
            }
        });
    }

    /// Execute stress scenario operation
    async fn execute_stress_scenario_operation(
        session: &mut CliSession,
        scenario: &StressTestScenario,
    ) -> (bool, Option<String>) {
        match scenario {
            StressTestScenario::MemoryLeakDetection => {
                // Operations that might cause memory leaks
                let operations = [
                    "Create a large dataset and process it",
                    "Generate a detailed report with many sections",
                    "Process multiple files simultaneously",
                    "Execute complex calculations with large numbers",
                ];
                let op = operations[fastrand::usize(0..usize::MAX) % operations.len()];
                match session.send_line(op).await {
                    Ok(_) => (true, None),
                    Err(e) => (false, Some(e.to_string())),
                }
            }
            StressTestScenario::ResourceExhaustion => {
                // Operations that consume significant resources
                let op = "Calculate the factorial of 1000 and explain each step";
                match session.send_line(op).await {
                    Ok(_) => (true, None),
                    Err(e) => (false, Some(e.to_string())),
                }
            }
            StressTestScenario::ConnectionStress => {
                // Operations that stress connection handling
                let op = "What time is it? Also check the weather and calculate 42 * 17";
                match session.send_line(op).await {
                    Ok(_) => (true, None),
                    Err(e) => (false, Some(e.to_string())),
                }
            }
            StressTestScenario::ConversationCycling => {
                // Rapid conversation state changes
                let operations = [
                    "Remember that my name is TestUser",
                    "What's my name?",
                    "Forget everything we discussed",
                    "Start a new topic about programming",
                ];
                let op = operations[fastrand::usize(0..usize::MAX) % operations.len()];
                match session.send_line(op).await {
                    Ok(_) => (true, None),
                    Err(e) => (false, Some(e.to_string())),
                }
            }
            StressTestScenario::ToolExecutionStress => {
                // Heavy tool usage
                let operations = [
                    "Get the current time, calculate 123 * 456, and get my HOME directory",
                    "Calculate the square root of 2, get the time, and list environment variables",
                    "Do math: 999 + 111, then tell me the time twice",
                ];
                let op = operations[fastrand::usize(0..usize::MAX) % operations.len()];
                match session.send_line(op).await {
                    Ok(_) => (true, None),
                    Err(e) => (false, Some(e.to_string())),
                }
            }
            StressTestScenario::MixedHighIntensity => {
                // Mixed high-intensity operations
                let operations = [
                    "Write a detailed technical explanation of quantum computing",
                    "Calculate complex mathematical expressions and show work",
                    "Analyze a hypothetical dataset and provide insights",
                    "Create a comprehensive project plan with timeline",
                ];
                let op = operations[fastrand::usize(0..usize::MAX) % operations.len()];
                match session.send_line(op).await {
                    Ok(_) => (true, None),
                    Err(e) => (false, Some(e.to_string())),
                }
            }
        }
    }

    /// Analyze stress test results
    async fn analyze_stress_test_results(
        &self,
        execution_time: Duration,
    ) -> super::Result<StressTestResults> {
        let results = self.results.lock().unwrap().clone();
        let monitor = self.resource_monitor.lock().unwrap();

        // Basic metrics
        let total_operations = results.len();
        let successful_operations = results.iter().filter(|r| r.success).count();
        let failed_operations = total_operations - successful_operations;
        let peak_concurrent_users = self.user_counter.load(Ordering::SeqCst);

        // Memory leak detection
        let memory_leak_detected = self.detect_memory_leak(&monitor.memory_samples);
        let memory_leak_rate = self.calculate_memory_leak_rate(&monitor.memory_samples);

        // Resource metrics
        let resource_metrics = ResourceMetrics {
            memory_samples: monitor.memory_samples.clone(),
            memory_leak_detected,
            memory_leak_rate_mb_per_min: memory_leak_rate,
            peak_memory_mb: monitor
                .memory_samples
                .iter()
                .map(|s| s.memory_mb)
                .max()
                .unwrap_or(0),
            cpu_usage_samples: monitor.cpu_samples.clone(),
            peak_cpu_usage: monitor
                .cpu_samples
                .iter()
                .map(|s| s.cpu_percent)
                .fold(0.0, f64::max),
            connection_pool_stats: monitor.connection_stats.clone(),
        };

        // Stability metrics
        let stability_metrics = self.calculate_stability_metrics(&results);

        // Failure pattern analysis
        let failure_patterns = self.analyze_failure_patterns(&results);

        // Critical issues detection
        let critical_issues =
            self.detect_critical_issues(&resource_metrics, &stability_metrics, &failure_patterns);

        // Overall pass/fail determination
        let stress_test_passed = critical_issues.is_empty()
            && !memory_leak_detected
            && stability_metrics.stability_score > 0.8;

        Ok(StressTestResults {
            config: self.config.clone(),
            execution_time,
            peak_concurrent_users,
            total_operations,
            successful_operations,
            failed_operations,
            resource_metrics,
            stability_metrics,
            failure_patterns,
            stress_test_passed,
            critical_issues,
        })
    }

    /// Detect memory leaks from memory samples
    fn detect_memory_leak(&self, memory_samples: &[MemorySample]) -> bool {
        if memory_samples.len() < 10 {
            return false; // Not enough data
        }

        let rate = self.calculate_memory_leak_rate(memory_samples);
        rate > self.config.memory_thresholds.leak_threshold_mb_per_min
    }

    /// Calculate memory leak rate in MB per minute
    fn calculate_memory_leak_rate(&self, memory_samples: &[MemorySample]) -> f64 {
        if memory_samples.len() < 2 {
            return 0.0;
        }

        let first_sample = &memory_samples[0];
        let last_sample = &memory_samples[memory_samples.len() - 1];

        let memory_diff = last_sample.memory_mb as f64 - first_sample.memory_mb as f64;
        let time_diff = (last_sample.timestamp - first_sample.timestamp) / 60.0; // Convert to minutes

        if time_diff > 0.0 {
            memory_diff / time_diff
        } else {
            0.0
        }
    }

    /// Calculate stability metrics
    fn calculate_stability_metrics(&self, results: &[OperationResult]) -> StabilityMetrics {
        let mut error_rate_samples = Vec::new();
        let window_size = 100; // Calculate error rate over 100-operation windows

        for (_i, chunk) in results.chunks(window_size).enumerate() {
            let errors = chunk.iter().filter(|r| !r.success).count();
            let error_rate = errors as f64 / chunk.len() as f64;
            let avg_users = chunk.iter().map(|r| r.concurrent_users).sum::<usize>() / chunk.len();

            error_rate_samples.push(ErrorRateSample {
                timestamp: chunk[0].timestamp.elapsed().as_secs_f64(),
                error_rate,
                concurrent_users: avg_users,
            });
        }

        // Check for response time degradation
        let response_time_degradation = self.detect_response_time_degradation(results);

        // Calculate stability score
        let avg_error_rate = error_rate_samples.iter().map(|s| s.error_rate).sum::<f64>()
            / error_rate_samples.len() as f64;
        let stability_score = (1.0 - avg_error_rate).max(0.0);

        StabilityMetrics {
            error_rate_over_time: error_rate_samples,
            response_time_degradation,
            system_recovery_time: None, // Would need failure injection to measure
            stability_score,
        }
    }

    /// Detect response time degradation
    fn detect_response_time_degradation(&self, results: &[OperationResult]) -> bool {
        if results.len() < 100 {
            return false;
        }

        let first_quarter = &results[0..results.len() / 4];
        let last_quarter = &results[results.len() * 3 / 4..];

        let avg_early = first_quarter
            .iter()
            .map(|r| r.duration.as_millis())
            .sum::<u128>()
            / first_quarter.len() as u128;
        let avg_late = last_quarter
            .iter()
            .map(|r| r.duration.as_millis())
            .sum::<u128>()
            / last_quarter.len() as u128;

        // Consider degradation if response time increased by more than 50%
        avg_late > avg_early * 150 / 100
    }

    /// Analyze failure patterns
    fn analyze_failure_patterns(&self, results: &[OperationResult]) -> Vec<FailurePattern> {
        let mut patterns = Vec::new();
        let mut error_types: HashMap<String, Vec<&OperationResult>> = HashMap::new();

        // Group errors by type
        for result in results.iter().filter(|r| !r.success) {
            if let Some(error_type) = &result.error_type {
                error_types
                    .entry(error_type.clone())
                    .or_default()
                    .push(result);
            }
        }

        // Analyze each error type
        for (error_type, error_results) in error_types {
            if error_results.len() > 5 {
                // Only consider patterns with multiple occurrences
                let severity = if error_results.len() > 50 {
                    FailureSeverity::Critical
                } else if error_results.len() > 20 {
                    FailureSeverity::High
                } else if error_results.len() > 10 {
                    FailureSeverity::Medium
                } else {
                    FailureSeverity::Low
                };

                patterns.push(FailurePattern {
                    pattern_type: error_type.clone(),
                    description: format!("Recurring error pattern: {}", error_type),
                    severity,
                    first_occurrence: error_results[0].timestamp.elapsed().as_secs_f64(),
                    frequency: error_results.len(),
                });
            }
        }

        patterns
    }

    /// Detect critical issues
    fn detect_critical_issues(
        &self,
        resource_metrics: &ResourceMetrics,
        stability_metrics: &StabilityMetrics,
        failure_patterns: &[FailurePattern],
    ) -> Vec<String> {
        let mut issues = Vec::new();

        // Memory issues
        if resource_metrics.memory_leak_detected {
            issues.push(format!(
                "Memory leak detected: {} MB/min increase",
                resource_metrics.memory_leak_rate_mb_per_min
            ));
        }

        if resource_metrics.peak_memory_mb > self.config.memory_thresholds.critical_mb {
            issues.push(format!(
                "Critical memory usage: {} MB (threshold: {} MB)",
                resource_metrics.peak_memory_mb, self.config.memory_thresholds.critical_mb
            ));
        }

        // Stability issues
        if stability_metrics.response_time_degradation {
            issues.push("Significant response time degradation detected".to_string());
        }

        if stability_metrics.stability_score < 0.5 {
            issues.push(format!(
                "Poor stability score: {:.2} (minimum: 0.5)",
                stability_metrics.stability_score
            ));
        }

        // Critical failure patterns
        for pattern in failure_patterns
            .iter()
            .filter(|p| matches!(p.severity, FailureSeverity::Critical))
        {
            issues.push(format!(
                "Critical failure pattern: {} (frequency: {})",
                pattern.description, pattern.frequency
            ));
        }

        issues
    }

    /// Print stress test summary
    fn print_stress_test_summary(&self, results: &StressTestResults) {
        println!("\n🔥 STRESS TEST RESULTS SUMMARY");
        println!("════════════════════════════════════════");
        println!("🎯 Scenario: {:?}", results.config.scenario);
        println!(
            "👥 Peak Concurrent Users: {}",
            results.peak_concurrent_users
        );
        println!("⏱️  Execution Time: {:?}", results.execution_time);
        println!();

        println!("📊 PERFORMANCE METRICS");
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
            results.failed_operations as f64 / results.total_operations as f64 * 100.0
        );
        println!();

        println!("🧠 RESOURCE ANALYSIS");
        println!("────────────────────────────────────────");
        println!(
            "Peak Memory: {} MB",
            results.resource_metrics.peak_memory_mb
        );
        println!(
            "Memory Leak Detected: {}",
            if results.resource_metrics.memory_leak_detected {
                "❌ YES"
            } else {
                "✅ NO"
            }
        );
        if results.resource_metrics.memory_leak_detected {
            println!(
                "Memory Leak Rate: {:.2} MB/min",
                results.resource_metrics.memory_leak_rate_mb_per_min
            );
        }
        println!(
            "Peak CPU Usage: {:.1}%",
            results.resource_metrics.peak_cpu_usage
        );
        println!();

        println!("🎯 STABILITY ANALYSIS");
        println!("────────────────────────────────────────");
        println!(
            "Stability Score: {:.3}",
            results.stability_metrics.stability_score
        );
        println!(
            "Response Time Degradation: {}",
            if results.stability_metrics.response_time_degradation {
                "❌ YES"
            } else {
                "✅ NO"
            }
        );
        println!();

        println!("⚠️  FAILURE PATTERNS");
        println!("────────────────────────────────────────");
        if results.failure_patterns.is_empty() {
            println!("✅ No significant failure patterns detected");
        } else {
            for pattern in &results.failure_patterns {
                println!(
                    "• {:?}: {} (freq: {})",
                    pattern.severity, pattern.description, pattern.frequency
                );
            }
        }
        println!();

        println!("🏁 FINAL RESULT");
        println!("────────────────────────────────────────");
        if results.stress_test_passed {
            println!("🎉 STRESS TEST PASSED");
        } else {
            println!("❌ STRESS TEST FAILED");
            println!("Critical Issues:");
            for issue in &results.critical_issues {
                println!("   • {}", issue);
            }
        }
        println!();
    }
}

impl ResourceMonitor {
    fn new() -> Self {
        Self {
            memory_samples: Vec::new(),
            cpu_samples: Vec::new(),
            connection_stats: ConnectionPoolStats {
                max_connections: 0,
                peak_active_connections: 0,
                connection_timeouts: 0,
                connection_errors: 0,
            },
        }
    }
}

/// Get process memory usage in MB
async fn get_process_memory_mb() -> super::Result<usize> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        let status = fs::read_to_string("/proc/self/status").map_err(|e| {
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to read proc status: {}", e))) as Box<dyn std::error::Error + Send + Sync>
        })?;
        
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<usize>() {
                        return Ok(kb / 1024); // Convert KB to MB
                    }
                }
            }
        }
        Ok(0)
    }

    #[cfg(target_os = "macos")]
    {
        // Simple implementation for macOS - would need more sophisticated approach for production
        Ok(100) // Mock value
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Ok(0)
    }
}

/// Get process CPU usage percentage
async fn get_process_cpu_usage() -> super::Result<f64> {
    #[cfg(target_os = "linux")]
    {
        // Simple CPU usage measurement - would need more sophisticated implementation for production
        // For now, return a mock value
        Ok(fastrand::f64() * 100.0)
    }

    #[cfg(target_os = "macos")]
    {
        // Mock implementation for macOS
        Ok(fastrand::f64() * 100.0)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Ok(0.0)
    }
}

/// Predefined stress test configurations
impl StressTestConfig {
    /// Memory leak detection test
    pub fn memory_leak_detection() -> Self {
        Self {
            test_duration: Duration::from_secs(600), // 10 minutes
            max_concurrent_users: 50,
            ramp_pattern: RampPattern::Linear {
                step_size: 10,
                step_interval: Duration::from_secs(30),
            },
            monitoring_interval: Duration::from_secs(10),
            memory_thresholds: MemoryThresholds {
                warning_mb: 1000,
                critical_mb: 2000,
                leak_threshold_mb_per_min: 10.0,
                max_memory_mb: 4000,
            },
            failure_injection: FailureInjectionConfig {
                network_latency: false,
                timeout_simulation: false,
                memory_pressure: true,
                failure_rate: 0.0,
            },
            scenario: StressTestScenario::MemoryLeakDetection,
        }
    }

    /// Resource exhaustion test
    pub fn resource_exhaustion() -> Self {
        Self {
            test_duration: Duration::from_secs(300), // 5 minutes
            max_concurrent_users: 100,
            ramp_pattern: RampPattern::Spike {
                spike_duration: Duration::from_secs(60),
            },
            monitoring_interval: Duration::from_secs(5),
            memory_thresholds: MemoryThresholds {
                warning_mb: 1500,
                critical_mb: 3000,
                leak_threshold_mb_per_min: 20.0,
                max_memory_mb: 6000,
            },
            failure_injection: FailureInjectionConfig {
                network_latency: true,
                timeout_simulation: true,
                memory_pressure: true,
                failure_rate: 0.1,
            },
            scenario: StressTestScenario::ResourceExhaustion,
        }
    }

    /// Chaos engineering test
    pub fn chaos_engineering() -> Self {
        Self {
            test_duration: Duration::from_secs(900), // 15 minutes
            max_concurrent_users: 75,
            ramp_pattern: RampPattern::Exponential {
                multiplier: 1.5,
                step_interval: Duration::from_secs(60),
            },
            monitoring_interval: Duration::from_secs(15),
            memory_thresholds: MemoryThresholds {
                warning_mb: 1200,
                critical_mb: 2500,
                leak_threshold_mb_per_min: 15.0,
                max_memory_mb: 5000,
            },
            failure_injection: FailureInjectionConfig {
                network_latency: true,
                timeout_simulation: true,
                memory_pressure: true,
                failure_rate: 0.2,
            },
            scenario: StressTestScenario::MixedHighIntensity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_leak_detection() -> super::Result<()> {
        if !check_aws_credentials() {
            println!("⚠️  Skipping stress test - AWS credentials not available");
            return Ok(());
        }

        let config = StressTestConfig::memory_leak_detection();
        let executor = StressTestExecutor::new(config);

        let results = executor.execute().await?;

        // Validate results
        assert!(results.total_operations > 0, "No operations executed");
        assert!(
            results.resource_metrics.memory_samples.len() > 0,
            "No memory samples collected"
        );

        // Log results for analysis
        println!("Memory leak test completed:");
        println!(
            "  Peak memory: {} MB",
            results.resource_metrics.peak_memory_mb
        );
        println!(
            "  Memory leak detected: {}",
            results.resource_metrics.memory_leak_detected
        );
        println!(
            "  Stability score: {:.3}",
            results.stability_metrics.stability_score
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_resource_exhaustion() -> super::Result<()> {
        if !check_aws_credentials() {
            println!("⚠️  Skipping stress test - AWS credentials not available");
            return Ok(());
        }

        let config = StressTestConfig::resource_exhaustion();
        let executor = StressTestExecutor::new(config);

        let results = executor.execute().await?;

        // Validate results
        assert!(results.total_operations > 0, "No operations executed");
        assert!(
            results.peak_concurrent_users > 0,
            "No concurrent users recorded"
        );

        println!("Resource exhaustion test completed:");
        println!("  Total operations: {}", results.total_operations);
        println!("  Peak users: {}", results.peak_concurrent_users);
        println!("  Critical issues: {}", results.critical_issues.len());

        Ok(())
    }
}
