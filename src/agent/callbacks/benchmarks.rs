//! Performance benchmarks for the callback system
//!
//! This module provides benchmarking utilities to measure callback system performance
//! and identify optimization opportunities.

use super::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Simple benchmark result structure
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub operation: String,
    pub iterations: usize,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub ops_per_second: f64,
}

impl BenchmarkResult {
    pub fn new(operation: String, iterations: usize, total_duration: Duration) -> Self {
        let avg_duration = total_duration / iterations as u32;
        let ops_per_second = iterations as f64 / total_duration.as_secs_f64();

        Self {
            operation,
            iterations,
            total_duration,
            avg_duration,
            ops_per_second,
        }
    }
}

/// Benchmark callback handler that tracks performance metrics
#[derive(Debug)]
pub struct BenchmarkCallbackHandler {
    event_count: Arc<Mutex<usize>>,
    total_processing_time: Arc<Mutex<Duration>>,
}

impl BenchmarkCallbackHandler {
    pub fn new() -> Self {
        Self {
            event_count: Arc::new(Mutex::new(0)),
            total_processing_time: Arc::new(Mutex::new(Duration::ZERO)),
        }
    }

    pub fn get_stats(&self) -> (usize, Duration) {
        let count = *self.event_count.lock().unwrap();
        let total_time = *self.total_processing_time.lock().unwrap();
        (count, total_time)
    }

    pub fn reset(&self) {
        *self.event_count.lock().unwrap() = 0;
        *self.total_processing_time.lock().unwrap() = Duration::ZERO;
    }
}

#[async_trait::async_trait]
impl CallbackHandler for BenchmarkCallbackHandler {
    async fn handle_event(&self, _event: CallbackEvent) -> Result<(), CallbackError> {
        let start = Instant::now();

        // Simulate minimal processing work
        // In real scenarios, callbacks might do logging, metric collection, etc.
        let _processing_work = format!("Processing event");

        let processing_time = start.elapsed();

        // Update metrics
        *self.event_count.lock().unwrap() += 1;
        *self.total_processing_time.lock().unwrap() += processing_time;

        Ok(())
    }
}

/// Benchmark utilities for measuring callback performance
pub struct CallbackBenchmark;

impl CallbackBenchmark {
    /// Benchmark event creation and dispatch performance
    pub async fn benchmark_event_creation(iterations: usize) -> BenchmarkResult {
        let start = Instant::now();

        for _i in 0..iterations {
            let _event = CallbackEvent::ContentDelta {
                delta: "test content".to_string(),
                complete: false,
                reasoning: false,
            };
        }

        let total_duration = start.elapsed();
        BenchmarkResult::new("Event Creation".to_string(), iterations, total_duration)
    }

    /// Benchmark callback handler dispatch performance
    pub async fn benchmark_handler_dispatch(iterations: usize) -> BenchmarkResult {
        let handler = BenchmarkCallbackHandler::new();
        let start = Instant::now();

        for _i in 0..iterations {
            let event = CallbackEvent::ContentDelta {
                delta: "test content".to_string(),
                complete: false,
                reasoning: false,
            };

            let _ = handler.handle_event(event).await;
        }

        let total_duration = start.elapsed();
        BenchmarkResult::new("Handler Dispatch".to_string(), iterations, total_duration)
    }

    /// Benchmark composite handler performance
    pub async fn benchmark_composite_handler(
        iterations: usize,
        handler_count: usize,
    ) -> BenchmarkResult {
        // Create multiple handlers
        let mut handlers: Vec<Arc<dyn CallbackHandler>> = Vec::new();
        for _i in 0..handler_count {
            handlers.push(Arc::new(BenchmarkCallbackHandler::new()));
        }

        let composite = CompositeCallbackHandler::with_handlers(handlers);
        let start = Instant::now();

        for _i in 0..iterations {
            let event = CallbackEvent::ContentDelta {
                delta: "test content".to_string(),
                complete: false,
                reasoning: false,
            };

            let _ = composite.handle_event(event).await;
        }

        let total_duration = start.elapsed();
        BenchmarkResult::new(
            format!("Composite Handler ({} handlers)", handler_count),
            iterations,
            total_duration,
        )
    }

    /// Run a comprehensive benchmark suite
    pub async fn run_benchmark_suite() -> Vec<BenchmarkResult> {
        println!("🚀 Running callback system performance benchmarks...");

        let mut results = Vec::new();

        // Test event creation performance
        results.push(Self::benchmark_event_creation(10_000).await);

        // Test single handler dispatch performance
        results.push(Self::benchmark_handler_dispatch(10_000).await);

        // Test composite handler performance with different handler counts
        results.push(Self::benchmark_composite_handler(1_000, 2).await);
        results.push(Self::benchmark_composite_handler(1_000, 5).await);
        results.push(Self::benchmark_composite_handler(1_000, 10).await);

        results
    }

    /// Print benchmark results in a formatted table
    pub fn print_results(results: &[BenchmarkResult]) {
        println!("\n📊 Callback System Performance Benchmark Results");
        println!("═══════════════════════════════════════════════════════════════════");
        println!(
            "{:<30} {:>10} {:>15} {:>15} {:>15}",
            "Operation", "Iterations", "Total (ms)", "Avg (μs)", "Ops/sec"
        );
        println!("───────────────────────────────────────────────────────────────────");

        for result in results {
            println!(
                "{:<30} {:>10} {:>15.2} {:>15.2} {:>15.0}",
                result.operation,
                result.iterations,
                result.total_duration.as_millis(),
                result.avg_duration.as_micros(),
                result.ops_per_second
            );
        }

        println!("═══════════════════════════════════════════════════════════════════");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_benchmark_callback_handler() {
        let handler = BenchmarkCallbackHandler::new();

        // Process some events
        for _i in 0..5 {
            let event = CallbackEvent::ContentDelta {
                delta: "test".to_string(),
                complete: false,
                reasoning: false,
            };
            handler.handle_event(event).await.unwrap();
        }

        let (count, _total_time) = handler.get_stats();
        assert_eq!(count, 5);
    }

    #[tokio::test]
    async fn test_event_creation_benchmark() {
        let result = CallbackBenchmark::benchmark_event_creation(100).await;
        assert_eq!(result.iterations, 100);
        assert!(result.ops_per_second > 0.0);
    }

    #[tokio::test]
    async fn test_handler_dispatch_benchmark() {
        let result = CallbackBenchmark::benchmark_handler_dispatch(100).await;
        assert_eq!(result.iterations, 100);
        assert!(result.ops_per_second > 0.0);
    }
}
