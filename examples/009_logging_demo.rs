//! Demonstration of the comprehensive logging and tracing system
//!
//! This example shows how the logging system captures detailed performance data
//! and traces all operations with exact timings.

use std::time::Duration;
use stood::telemetry::{init_logging, LoggingConfig, PerformanceTracer};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize comprehensive logging
    let logging_config = LoggingConfig {
        log_dir: std::env::current_dir()?.join("logs"),
        max_file_size: 10 * 1024 * 1024, // 10MB for demo
        max_files: 2,
        file_log_level: "DEBUG".to_string(),
        console_log_level: "INFO".to_string(),
        console_enabled: true,
        json_format: true,
        enable_performance_tracing: true,
        enable_cycle_detection: true,
    };

    let _logging_guard = init_logging(logging_config)?;

    // Initialize performance tracer
    let tracer = PerformanceTracer::new();

    println!("🚀 Logging and Tracing Demo Started");
    println!("📁 Logs will be written to: logs/stood.log");
    println!("📊 All operations will be traced with exact timings");
    println!();

    // Demonstrate comprehensive tracing
    demo_basic_operations(&tracer).await;
    demo_nested_operations(&tracer).await;
    demo_performance_detection(&tracer).await;
    demo_cycle_detection(&tracer).await;

    println!("✅ Demo completed! Check logs/stood.log for detailed trace data");
    println!("💡 The log file contains JSON-formatted entries with exact timings");

    Ok(())
}

async fn demo_basic_operations(tracer: &PerformanceTracer) {
    println!("🔬 Demo 1: Basic Operation Tracing");

    let _guard = tracer.start_operation("demo_basic_operations");
    _guard.add_context("demo_type", "basic");

    {
        let _task_guard = tracer.start_operation("task_1_quick");
        _task_guard.add_context("task_id", "1");
        sleep(Duration::from_millis(50)).await;
        _task_guard.checkpoint("task_1_complete");
    }

    {
        let _task_guard = tracer.start_operation("task_2_medium");
        _task_guard.add_context("task_id", "2");
        sleep(Duration::from_millis(150)).await;
        _task_guard.checkpoint("task_2_complete");
    }

    {
        let _task_guard = tracer.start_operation("task_3_slow");
        _task_guard.add_context("task_id", "3");
        sleep(Duration::from_millis(300)).await;
        _task_guard.checkpoint("task_3_complete");
    }

    println!("   ✅ Basic operations traced");
    println!();
}

async fn demo_nested_operations(tracer: &PerformanceTracer) {
    println!("🔗 Demo 2: Nested Operation Tracing");

    let _guard = tracer.start_operation("demo_nested_operations");
    _guard.add_context("nesting_level", "0");

    {
        let _level1_guard = tracer.start_operation("level_1_operation");
        _level1_guard.add_context("level", "1");
        sleep(Duration::from_millis(30)).await;

        {
            let _level2_guard = tracer.start_operation("level_2_operation");
            _level2_guard.add_context("level", "2");
            sleep(Duration::from_millis(60)).await;

            {
                let _level3_guard = tracer.start_operation("level_3_operation");
                _level3_guard.add_context("level", "3");
                sleep(Duration::from_millis(90)).await;
                _level3_guard.checkpoint("deepest_operation");
            }

            _level2_guard.checkpoint("level_2_complete");
        }

        _level1_guard.checkpoint("level_1_complete");
    }

    println!("   ✅ Nested operations traced with stack depth tracking");
    println!();
}

async fn demo_performance_detection(tracer: &PerformanceTracer) {
    println!("⚡ Demo 3: Performance Issue Detection");

    let _guard = tracer.start_operation("demo_performance_detection");

    // Simulate waiting for external resource
    {
        let _wait_guard = tracer.start_operation("external_api_call");
        _wait_guard.add_context("api", "slow_service");
        let wait_duration = Duration::from_millis(600); // Will trigger slow operation warning
        sleep(wait_duration).await;
        _wait_guard.record_wait("external_api", wait_duration);
    }

    // Simulate blocking operation
    {
        let _block_guard = tracer.start_operation("file_processing");
        _block_guard.add_context("file_size", "large");
        let block_duration = Duration::from_millis(1200); // Will trigger blocking operation warning
        sleep(block_duration).await;
        _block_guard.record_blocking("file_io", block_duration);
    }

    println!("   ⚠️  Performance issues detected and logged");
    println!();
}

async fn demo_cycle_detection(tracer: &PerformanceTracer) {
    println!("🔄 Demo 4: Event Loop Cycle Detection");

    let _guard = tracer.start_operation("demo_cycle_detection");

    // Simulate event loop cycles
    for i in 1..=10 {
        let cycle_duration = if i <= 5 {
            Duration::from_millis(50) // Normal cycles
        } else {
            Duration::from_millis(20) // Rapid cycles (will trigger loop detection)
        };

        tracer.record_cycle(&format!("demo_cycle_{}", i), cycle_duration);
        sleep(cycle_duration).await;
    }

    // Simulate rapid type changes
    for i in 1..=15 {
        let cycle_type = match i % 3 {
            0 => "type_a",
            1 => "type_b",
            _ => "type_c",
        };
        tracer.record_cycle(cycle_type, Duration::from_millis(10));
        sleep(Duration::from_millis(10)).await;
    }

    println!("   🔍 Event loop patterns analyzed and logged");
    println!();
}
