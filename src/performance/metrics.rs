//! Comprehensive performance metrics for production monitoring
//!
//! This module enables you to collect, analyze, and monitor performance metrics
//! across all aspects of your AI agent deployment. You'll get detailed insights
//! into request latency, resource usage, optimization effectiveness, and error patterns.
//!
//! # Metrics Categories
//!
//! ## Request Metrics
//! - **Throughput**: Total, successful, and failed request counts
//! - **Latency**: Request processing times with percentile analysis
//! - **Error Rates**: Categorized error tracking with trend analysis
//!
//! ## Resource Metrics
//! - **Connection Pool**: Hit rates, acquisition times, active connections
//! - **Memory Usage**: Optimization results, freed bytes, threshold monitoring
//! - **Batch Processing**: Batch sizes, processing times, efficiency metrics
//!
//! ## Performance Indicators
//! - **Health Scores**: Overall system health based on multiple metrics
//! - **Optimization Impact**: Before/after performance improvements
//! - **Trend Analysis**: Performance changes over time
//!
//! # Usage Patterns
//!
//! ```rust
//! use stood::performance::PerformanceMetrics;
//! use std::time::Duration;
//!
//! let mut metrics = PerformanceMetrics::new();
//!
//! // Record request metrics
//! metrics.record_request_success(Duration::from_millis(150));
//! metrics.record_request_failure("timeout".to_string());
//!
//! // Record resource metrics
//! metrics.record_memory_optimization(1024 * 1024); // 1MB freed
//! metrics.record_connection_acquisition(Duration::from_millis(2));
//!
//! // Analyze performance
//! println!("Average latency: {:?}", metrics.average_latency());
//! println!("Error rate: {:.2}%", metrics.error_rate() * 100.0);
//! println!("Pool hit rate: {:.2}%", metrics.pool_hit_rate() * 100.0);
//! ```
//!
//! # Monitoring Integration
//!
//! Metrics are designed for integration with monitoring systems:
//!
//! - **Prometheus**: Export metrics in Prometheus format
//! - **CloudWatch**: Push metrics to AWS CloudWatch
//! - **Grafana**: Build dashboards from collected metrics
//! - **Custom Systems**: Access raw metrics data for custom analysis

use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    time::{Duration, Instant},
};
use tracing::{debug, info};

/// Comprehensive performance metrics collector for production monitoring
///
/// This collector tracks all aspects of agent performance including request metrics,
/// resource usage, optimization results, and error patterns. It provides both
/// real-time access to current metrics and historical analysis capabilities.
///
/// # Thread Safety
///
/// All metric operations are thread-safe and designed for high-concurrency access.
/// Atomic operations are used for counters while time-sensitive data uses
/// appropriate synchronization primitives.
///
/// # Memory Management
///
/// The collector maintains a configurable number of samples to prevent unbounded
/// memory growth. Older samples are automatically discarded when limits are reached.
#[derive(Debug)]
pub struct PerformanceMetrics {
    // Request metrics
    total_requests: AtomicUsize,
    successful_requests: AtomicUsize,
    failed_requests: AtomicUsize,

    // Timing metrics
    latency_samples: VecDeque<Duration>,
    connection_acquisition_times: VecDeque<Duration>,
    processing_times: VecDeque<Duration>,

    // Resource metrics
    memory_optimizations: AtomicUsize,
    memory_freed_bytes: AtomicU64,
    active_connections: AtomicUsize,
    pool_hits: AtomicUsize,
    pool_misses: AtomicUsize,

    // Batch metrics
    total_batches: AtomicUsize,
    total_batch_items: AtomicUsize,
    batch_processing_times: VecDeque<Duration>,

    // Error tracking
    error_counts: HashMap<String, AtomicUsize>,
    timeout_counts: AtomicUsize,

    // Timestamps
    start_time: Instant,
    last_reset: Instant,

    // Configuration
    max_samples: usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PerformanceMetrics {
    fn clone(&self) -> Self {
        Self {
            total_requests: AtomicUsize::new(self.total_requests.load(Ordering::Relaxed)),
            successful_requests: AtomicUsize::new(self.successful_requests.load(Ordering::Relaxed)),
            failed_requests: AtomicUsize::new(self.failed_requests.load(Ordering::Relaxed)),
            latency_samples: self.latency_samples.clone(),
            connection_acquisition_times: self.connection_acquisition_times.clone(),
            processing_times: self.processing_times.clone(),
            memory_optimizations: AtomicUsize::new(
                self.memory_optimizations.load(Ordering::Relaxed),
            ),
            memory_freed_bytes: AtomicU64::new(self.memory_freed_bytes.load(Ordering::Relaxed)),
            active_connections: AtomicUsize::new(self.active_connections.load(Ordering::Relaxed)),
            pool_hits: AtomicUsize::new(self.pool_hits.load(Ordering::Relaxed)),
            pool_misses: AtomicUsize::new(self.pool_misses.load(Ordering::Relaxed)),
            total_batches: AtomicUsize::new(self.total_batches.load(Ordering::Relaxed)),
            total_batch_items: AtomicUsize::new(self.total_batch_items.load(Ordering::Relaxed)),
            batch_processing_times: self.batch_processing_times.clone(),
            error_counts: self
                .error_counts
                .iter()
                .map(|(k, v)| (k.clone(), AtomicUsize::new(v.load(Ordering::Relaxed))))
                .collect(),
            timeout_counts: AtomicUsize::new(self.timeout_counts.load(Ordering::Relaxed)),
            start_time: self.start_time,
            last_reset: self.last_reset,
            max_samples: self.max_samples,
        }
    }
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            total_requests: AtomicUsize::new(0),
            successful_requests: AtomicUsize::new(0),
            failed_requests: AtomicUsize::new(0),
            latency_samples: VecDeque::new(),
            connection_acquisition_times: VecDeque::new(),
            processing_times: VecDeque::new(),
            memory_optimizations: AtomicUsize::new(0),
            memory_freed_bytes: AtomicU64::new(0),
            active_connections: AtomicUsize::new(0),
            pool_hits: AtomicUsize::new(0),
            pool_misses: AtomicUsize::new(0),
            total_batches: AtomicUsize::new(0),
            total_batch_items: AtomicUsize::new(0),
            batch_processing_times: VecDeque::new(),
            error_counts: HashMap::new(),
            timeout_counts: AtomicUsize::new(0),
            start_time: now,
            last_reset: now,
            max_samples: 1000,
        }
    }

    /// Record a request completion
    pub fn record_request(&mut self, success: bool, latency: Duration) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }

        self.add_latency_sample(latency);
    }

    /// Record connection acquisition time
    pub fn record_connection_acquisition(&mut self, duration: Duration) {
        self.add_connection_acquisition_time(duration);
    }

    /// Record processing time
    pub fn record_processing_time(&mut self, duration: Duration) {
        self.add_processing_time(duration);
    }

    /// Record memory optimization
    pub fn record_memory_optimization(&mut self, bytes_freed: usize) {
        self.memory_optimizations.fetch_add(1, Ordering::Relaxed);
        self.memory_freed_bytes
            .fetch_add(bytes_freed as u64, Ordering::Relaxed);
    }

    /// Record batch processing
    pub fn record_batch(&mut self, item_count: usize, processing_time: Duration) {
        self.total_batches.fetch_add(1, Ordering::Relaxed);
        self.total_batch_items
            .fetch_add(item_count, Ordering::Relaxed);
        self.add_batch_processing_time(processing_time);
    }

    /// Record an error
    pub fn record_error(&mut self, error_type: &str) {
        let counter = self
            .error_counts
            .entry(error_type.to_string())
            .or_insert_with(|| AtomicUsize::new(0));
        counter.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a timeout
    pub fn record_timeout(&mut self) {
        self.timeout_counts.fetch_add(1, Ordering::Relaxed);
    }

    /// Record pool hit
    pub fn record_pool_hit(&mut self) {
        self.pool_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record pool miss
    pub fn record_pool_miss(&mut self) {
        self.pool_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Update active connection count
    pub fn set_active_connections(&mut self, count: usize) {
        self.active_connections.store(count, Ordering::Relaxed);
    }

    // Private helper methods for maintaining sample collections
    fn add_latency_sample(&mut self, duration: Duration) {
        self.latency_samples.push_back(duration);
        if self.latency_samples.len() > self.max_samples {
            self.latency_samples.pop_front();
        }
    }

    fn add_connection_acquisition_time(&mut self, duration: Duration) {
        self.connection_acquisition_times.push_back(duration);
        if self.connection_acquisition_times.len() > self.max_samples {
            self.connection_acquisition_times.pop_front();
        }
    }

    fn add_processing_time(&mut self, duration: Duration) {
        self.processing_times.push_back(duration);
        if self.processing_times.len() > self.max_samples {
            self.processing_times.pop_front();
        }
    }

    fn add_batch_processing_time(&mut self, duration: Duration) {
        self.batch_processing_times.push_back(duration);
        if self.batch_processing_times.len() > self.max_samples {
            self.batch_processing_times.pop_front();
        }
    }

    // Accessor methods
    pub fn total_requests(&self) -> usize {
        self.total_requests.load(Ordering::Relaxed)
    }

    pub fn successful_requests(&self) -> usize {
        self.successful_requests.load(Ordering::Relaxed)
    }

    pub fn failed_requests(&self) -> usize {
        self.failed_requests.load(Ordering::Relaxed)
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            1.0
        } else {
            self.successful_requests() as f64 / total as f64
        }
    }

    pub fn error_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            0.0
        } else {
            self.failed_requests() as f64 / total as f64
        }
    }

    pub fn average_latency(&self) -> Duration {
        if self.latency_samples.is_empty() {
            Duration::from_millis(0)
        } else {
            let total: Duration = self.latency_samples.iter().sum();
            total / self.latency_samples.len() as u32
        }
    }

    pub fn p95_latency(&self) -> Duration {
        self.percentile_latency(95.0)
    }

    pub fn p99_latency(&self) -> Duration {
        self.percentile_latency(99.0)
    }

    fn percentile_latency(&self, percentile: f64) -> Duration {
        if self.latency_samples.is_empty() {
            return Duration::from_millis(0);
        }

        let mut sorted: Vec<_> = self.latency_samples.iter().cloned().collect();
        sorted.sort();

        let index = ((percentile / 100.0) * sorted.len() as f64) as usize;
        let index = index.min(sorted.len() - 1);

        sorted[index]
    }

    pub fn average_connection_acquisition_time(&self) -> Duration {
        if self.connection_acquisition_times.is_empty() {
            Duration::from_millis(0)
        } else {
            let total: Duration = self.connection_acquisition_times.iter().sum();
            total / self.connection_acquisition_times.len() as u32
        }
    }

    pub fn average_processing_time(&self) -> Duration {
        if self.processing_times.is_empty() {
            Duration::from_millis(0)
        } else {
            let total: Duration = self.processing_times.iter().sum();
            total / self.processing_times.len() as u32
        }
    }

    pub fn memory_optimizations_count(&self) -> usize {
        self.memory_optimizations.load(Ordering::Relaxed)
    }

    pub fn total_memory_freed(&self) -> u64 {
        self.memory_freed_bytes.load(Ordering::Relaxed)
    }

    pub fn active_connections(&self) -> usize {
        self.active_connections.load(Ordering::Relaxed)
    }

    pub fn pool_hit_rate(&self) -> f64 {
        let hits = self.pool_hits.load(Ordering::Relaxed);
        let misses = self.pool_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    pub fn total_batches(&self) -> usize {
        self.total_batches.load(Ordering::Relaxed)
    }

    pub fn average_batch_size(&self) -> f64 {
        let batches = self.total_batches();
        if batches == 0 {
            0.0
        } else {
            self.total_batch_items.load(Ordering::Relaxed) as f64 / batches as f64
        }
    }

    pub fn average_batch_processing_time(&self) -> Duration {
        if self.batch_processing_times.is_empty() {
            Duration::from_millis(0)
        } else {
            let total: Duration = self.batch_processing_times.iter().sum();
            total / self.batch_processing_times.len() as u32
        }
    }

    pub fn timeout_count(&self) -> usize {
        self.timeout_counts.load(Ordering::Relaxed)
    }

    pub fn error_counts(&self) -> HashMap<String, usize> {
        self.error_counts
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect()
    }

    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn time_since_reset(&self) -> Duration {
        self.last_reset.elapsed()
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.successful_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
        self.latency_samples.clear();
        self.connection_acquisition_times.clear();
        self.processing_times.clear();
        self.memory_optimizations.store(0, Ordering::Relaxed);
        self.memory_freed_bytes.store(0, Ordering::Relaxed);
        self.active_connections.store(0, Ordering::Relaxed);
        self.pool_hits.store(0, Ordering::Relaxed);
        self.pool_misses.store(0, Ordering::Relaxed);
        self.total_batches.store(0, Ordering::Relaxed);
        self.total_batch_items.store(0, Ordering::Relaxed);
        self.batch_processing_times.clear();
        self.error_counts.clear();
        self.timeout_counts.store(0, Ordering::Relaxed);
        self.last_reset = Instant::now();

        info!("Performance metrics reset");
    }

    /// Generate a summary report
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            total_requests: self.total_requests(),
            success_rate: self.success_rate(),
            error_rate: self.error_rate(),
            average_latency_ms: self.average_latency().as_millis() as u64,
            p95_latency_ms: self.p95_latency().as_millis() as u64,
            p99_latency_ms: self.p99_latency().as_millis() as u64,
            average_connection_acquisition_ms: self
                .average_connection_acquisition_time()
                .as_millis() as u64,
            average_processing_time_ms: self.average_processing_time().as_millis() as u64,
            memory_optimizations: self.memory_optimizations_count(),
            memory_freed_mb: self.total_memory_freed() / (1024 * 1024),
            active_connections: self.active_connections(),
            pool_hit_rate: self.pool_hit_rate(),
            total_batches: self.total_batches(),
            average_batch_size: self.average_batch_size(),
            average_batch_processing_ms: self.average_batch_processing_time().as_millis() as u64,
            timeout_count: self.timeout_count(),
            uptime_seconds: self.uptime().as_secs(),
            error_counts: self.error_counts(),
        }
    }
}

/// Serializable metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub total_requests: usize,
    pub success_rate: f64,
    pub error_rate: f64,
    pub average_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub p99_latency_ms: u64,
    pub average_connection_acquisition_ms: u64,
    pub average_processing_time_ms: u64,
    pub memory_optimizations: usize,
    pub memory_freed_mb: u64,
    pub active_connections: usize,
    pub pool_hit_rate: f64,
    pub total_batches: usize,
    pub average_batch_size: f64,
    pub average_batch_processing_ms: u64,
    pub timeout_count: usize,
    pub uptime_seconds: u64,
    pub error_counts: HashMap<String, usize>,
}

impl MetricsSummary {
    /// Check if performance is within acceptable thresholds
    pub fn is_healthy(&self) -> bool {
        self.success_rate >= 0.95
            && self.error_rate <= 0.05
            && self.average_latency_ms <= 1000
            && self.pool_hit_rate >= 0.8
    }

    /// Get a health score (0.0 to 1.0)
    pub fn health_score(&self) -> f64 {
        let success_score = self.success_rate;
        let latency_score = (1000.0 - self.average_latency_ms as f64).max(0.0) / 1000.0;
        let pool_score = self.pool_hit_rate;

        (success_score + latency_score + pool_score) / 3.0
    }
}

/// Metrics reporter for periodic reporting
pub struct MetricsReporter {
    metrics: PerformanceMetrics,
    report_interval: Duration,
}

impl MetricsReporter {
    pub fn new(metrics: PerformanceMetrics, report_interval: Duration) -> Self {
        Self {
            metrics,
            report_interval,
        }
    }

    /// Start periodic reporting
    pub async fn start_reporting(&mut self) {
        let mut interval = tokio::time::interval(self.report_interval);

        loop {
            interval.tick().await;
            self.report_metrics();
        }
    }

    fn report_metrics(&self) {
        let summary = self.metrics.summary();

        info!(
            "Performance Report - Requests: {}, Success Rate: {:.2}%, Avg Latency: {}ms, Pool Hit Rate: {:.2}%",
            summary.total_requests,
            summary.success_rate * 100.0,
            summary.average_latency_ms,
            summary.pool_hit_rate * 100.0
        );

        if !summary.is_healthy() {
            debug!(
                "Performance health check failed: score = {:.2}",
                summary.health_score()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = PerformanceMetrics::new();
        assert_eq!(metrics.total_requests(), 0);
        assert_eq!(metrics.success_rate(), 1.0);
    }

    #[test]
    fn test_record_request() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_request(true, Duration::from_millis(100));
        assert_eq!(metrics.total_requests(), 1);
        assert_eq!(metrics.successful_requests(), 1);
        assert_eq!(metrics.success_rate(), 1.0);
    }

    #[test]
    fn test_latency_calculations() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_request(true, Duration::from_millis(100));
        metrics.record_request(true, Duration::from_millis(200));
        metrics.record_request(true, Duration::from_millis(300));

        assert_eq!(metrics.average_latency(), Duration::from_millis(200));
    }

    #[test]
    fn test_memory_optimization_tracking() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_memory_optimization(1024);
        assert_eq!(metrics.memory_optimizations_count(), 1);
        assert_eq!(metrics.total_memory_freed(), 1024);
    }

    #[test]
    fn test_pool_metrics() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_pool_hit();
        metrics.record_pool_hit();
        metrics.record_pool_miss();

        assert_eq!(metrics.pool_hit_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_metrics_summary() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_request(true, Duration::from_millis(100));
        metrics.record_memory_optimization(1024);
        // Add pool metrics to make it healthy
        metrics.record_pool_hit();
        metrics.record_pool_hit();
        metrics.record_pool_hit();
        metrics.record_pool_hit(); // 4 hits, 1 miss = 80% hit rate
        metrics.record_pool_miss();

        let summary = metrics.summary();
        assert_eq!(summary.total_requests, 1);
        assert_eq!(summary.success_rate, 1.0);
        assert!(summary.pool_hit_rate >= 0.8);
        assert!(summary.is_healthy());
    }

    #[test]
    fn test_health_score() {
        let summary = MetricsSummary {
            total_requests: 100,
            success_rate: 0.95,
            error_rate: 0.05,
            average_latency_ms: 200,
            p95_latency_ms: 500,
            p99_latency_ms: 800,
            average_connection_acquisition_ms: 50,
            average_processing_time_ms: 150,
            memory_optimizations: 5,
            memory_freed_mb: 10,
            active_connections: 3,
            pool_hit_rate: 0.85,
            total_batches: 20,
            average_batch_size: 5.0,
            average_batch_processing_ms: 100,
            timeout_count: 2,
            uptime_seconds: 3600,
            error_counts: HashMap::new(),
        };

        assert!(summary.is_healthy());
        assert!(summary.health_score() > 0.8);
    }
}
