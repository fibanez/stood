//! Telemetry Performance Benchmark
//!
//! Measures the overhead of telemetry instrumentation vs disabled telemetry.
//! This benchmark creates spans and records attributes without making actual
//! network calls to CloudWatch.
//!
//! Run with: cargo run --example 023_performance_benchmark

use std::sync::Arc;
use std::time::{Duration, Instant};
use stood::telemetry::exporter::NoOpExporter;
use stood::telemetry::tracer::StoodTracer;
use stood::telemetry::TelemetryConfig;

const ITERATIONS: usize = 10_000;
const WARMUP_ITERATIONS: usize = 1_000;

fn main() {
    println!("==============================================");
    println!("  Telemetry Performance Benchmark");
    println!("==============================================\n");

    // Warmup
    println!("Warming up ({} iterations)...", WARMUP_ITERATIONS);
    let config = TelemetryConfig::cloudwatch("us-east-1");
    let tracer = StoodTracer::new(config, Arc::new(NoOpExporter));
    for _ in 0..WARMUP_ITERATIONS {
        let mut span = tracer.start_chat_span("claude-3-haiku");
        span.set_attribute("test.iteration", "warmup");
        span.record_tokens(100, 50);
        span.set_success();
        span.finish();
    }

    println!("Running benchmarks ({} iterations each)...\n", ITERATIONS);

    // Benchmark 1: Disabled telemetry (baseline)
    let baseline_duration = benchmark_disabled();
    println!(
        "1. Disabled telemetry (baseline):     {:>8.2} ns/op",
        baseline_duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Benchmark 2: Enabled telemetry with NoOp exporter
    let enabled_duration = benchmark_enabled_noop();
    println!(
        "2. Enabled telemetry (NoOp export):   {:>8.2} ns/op",
        enabled_duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Benchmark 3: Full span lifecycle
    let full_span_duration = benchmark_full_span_lifecycle();
    println!(
        "3. Full span lifecycle:               {:>8.2} ns/op",
        full_span_duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Benchmark 4: Agent invocation span
    let agent_span_duration = benchmark_agent_span();
    println!(
        "4. Agent invocation span:             {:>8.2} ns/op",
        agent_span_duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Benchmark 5: Tool execution span
    let tool_span_duration = benchmark_tool_span();
    println!(
        "5. Tool execution span:               {:>8.2} ns/op",
        tool_span_duration.as_nanos() as f64 / ITERATIONS as f64
    );

    // Calculate overhead
    let overhead_ns = enabled_duration.as_nanos() as f64 - baseline_duration.as_nanos() as f64;
    let overhead_per_op = overhead_ns / ITERATIONS as f64;

    println!("\n----------------------------------------------");
    println!("Overhead Analysis:");
    println!("----------------------------------------------");
    println!(
        "  Telemetry overhead per span:        {:>8.2} ns",
        overhead_per_op
    );
    println!(
        "  Overhead percentage:                {:>8.2}%",
        (overhead_ns / baseline_duration.as_nanos() as f64) * 100.0
    );

    // Estimate real-world impact
    // Typical agent execution: ~500ms-2s with 3-10 spans
    let typical_spans = 5;
    let typical_execution_ms = 1000.0;
    let overhead_ms = (overhead_per_op * typical_spans as f64) / 1_000_000.0;
    let overhead_pct = (overhead_ms / typical_execution_ms) * 100.0;

    println!("\n----------------------------------------------");
    println!("Real-World Impact Estimate:");
    println!("----------------------------------------------");
    println!("  Typical agent execution:            1000 ms");
    println!("  Typical spans per execution:        {}", typical_spans);
    println!(
        "  Total telemetry overhead:           {:>8.4} ms",
        overhead_ms
    );
    println!(
        "  Percentage of execution time:       {:>8.4}%",
        overhead_pct
    );

    if overhead_pct < 0.01 {
        println!("\n  Result: NEGLIGIBLE overhead (<0.01%)");
    } else if overhead_pct < 0.1 {
        println!("\n  Result: MINIMAL overhead (<0.1%)");
    } else if overhead_pct < 1.0 {
        println!("\n  Result: ACCEPTABLE overhead (<1%)");
    } else {
        println!("\n  Result: HIGH overhead (>1%) - investigate");
    }

    println!("\n==============================================");
}

fn benchmark_disabled() -> Duration {
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        // Simulate what happens when telemetry is disabled
        // (tracer is None, so no span creation)
        let _tracer: Option<StoodTracer> = None;
        if let Some(_t) = &_tracer {
            // This branch never executes
        }
    }
    start.elapsed()
}

fn benchmark_enabled_noop() -> Duration {
    let config = TelemetryConfig::cloudwatch("us-east-1");
    let tracer = StoodTracer::new(config, Arc::new(NoOpExporter));

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let span = tracer.start_chat_span("claude-3-haiku");
        span.finish();
    }
    start.elapsed()
}

fn benchmark_full_span_lifecycle() -> Duration {
    let config = TelemetryConfig::cloudwatch("us-east-1");
    let tracer = StoodTracer::new(config, Arc::new(NoOpExporter));

    let start = Instant::now();
    for i in 0..ITERATIONS {
        let mut span = tracer.start_chat_span("claude-3-haiku");
        span.set_attribute("test.iteration", i as i64);
        span.set_attribute("test.model", "claude-3-haiku");
        span.record_tokens(150, 75);
        span.set_success();
        span.finish();
    }
    start.elapsed()
}

fn benchmark_agent_span() -> Duration {
    let config = TelemetryConfig::cloudwatch("us-east-1");
    let tracer = StoodTracer::new(config, Arc::new(NoOpExporter));

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        tracer.start_trace();
        let mut span = tracer.start_invoke_agent_span("my-agent", Some("agent-123"));
        span.set_attribute("agent.cycles", 3_i64);
        span.record_tokens(500, 250);
        span.set_success();
        span.finish();
    }
    start.elapsed()
}

fn benchmark_tool_span() -> Duration {
    let config = TelemetryConfig::cloudwatch("us-east-1");
    let tracer = StoodTracer::new(config, Arc::new(NoOpExporter));

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut span = tracer.start_execute_tool_span("get_weather", Some("tool-call-123"));
        span.set_attribute("tool.input_size", 256_i64);
        span.set_attribute("tool.output_size", 512_i64);
        span.set_success();
        span.finish();
    }
    start.elapsed()
}
