# Telemetry and Observability

Stood provides observability for AI agent performance monitoring, including file logging, performance tracing, and metrics collection.

## Current Status

The telemetry module supports:

- **File logging** - Production-ready via `LoggingConfig` and `PerformanceTracer`
- **Metrics types** - `EventLoopMetrics`, `CycleMetrics`, `TokenUsage` for tracking
- **OpenTelemetry integration** - Under active development for CloudWatch Gen AI

## Quick Start

### Basic Setup (Telemetry Disabled)

```rust
use stood::agent::Agent;
use stood::llm::models::Bedrock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Telemetry is disabled by default - agent works without configuration
    let mut agent = Agent::builder()
        .model(Bedrock::ClaudeHaiku45)
        .build().await?;

    let result = agent.execute("Hello, world").await?;
    println!("Response: {}", result.response);
    Ok(())
}
```

### With File Logging

```rust
use stood::telemetry::{init_logging, LoggingConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = LoggingConfig {
        log_dir: std::env::current_dir()?.join("logs"),
        max_file_size: 10 * 1024 * 1024, // 10MB
        max_files: 5,
        file_log_level: "DEBUG".to_string(),
        console_log_level: "INFO".to_string(),
        console_enabled: true,
        json_format: true,
        enable_performance_tracing: true,
        enable_cycle_detection: true,
    };

    let _guard = init_logging(config)?;

    // Your agent code here - logs go to logs/stood.log
    Ok(())
}
```

### With TelemetryConfig

```rust
use stood::agent::Agent;
use stood::telemetry::TelemetryConfig;
use stood::llm::models::Bedrock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = TelemetryConfig::default()
        .with_enabled(true)
        .with_service_name("my-agent");

    let mut agent = Agent::builder()
        .model(Bedrock::ClaudeHaiku45)
        .with_telemetry(config)
        .build().await?;

    Ok(())
}
```

## Environment Variables

```bash
# Core Configuration
OTEL_ENABLED=false                 # Enable/disable telemetry (default: false)
OTEL_SERVICE_NAME=stood-agent      # Service name for identification

# Logging
RUST_LOG=stood=info               # Log level filter
```

## Available Types

### TelemetryConfig

Configuration for telemetry and observability:

```rust
use stood::telemetry::{TelemetryConfig, LogLevel};

let config = TelemetryConfig::default()
    .with_enabled(true)
    .with_service_name("my-agent")
    .with_service_version("1.0.0")
    .with_log_level(LogLevel::DEBUG);

// From environment variables
let config = TelemetryConfig::from_env();

// For testing
let config = TelemetryConfig::for_testing();
```

### EventLoopMetrics

Metrics collected during agent execution:

```rust
use stood::telemetry::{EventLoopMetrics, CycleMetrics, TokenUsage};
use uuid::Uuid;
use std::time::Duration;

let mut metrics = EventLoopMetrics::new();

// Add a cycle
let cycle = CycleMetrics {
    cycle_id: Uuid::new_v4(),
    duration: Duration::from_millis(150),
    model_invocations: 1,
    tool_calls: 2,
    tokens_used: TokenUsage::new(100, 50),
    trace_id: None,
    span_id: None,
    start_time: chrono::Utc::now(),
    success: true,
    error: None,
};
metrics.add_cycle(cycle);

// Get summary
let summary = metrics.summary();
println!("Total cycles: {}", summary.total_cycles);
println!("Total tokens: {}", summary.total_tokens.total_tokens);
```

### TokenUsage

Track token consumption:

```rust
use stood::telemetry::TokenUsage;

let mut usage = TokenUsage::new(100, 50);  // 100 input, 50 output
assert_eq!(usage.total_tokens, 150);

// Accumulate usage
let more = TokenUsage::new(25, 25);
usage.add(&more);
assert_eq!(usage.total_tokens, 200);
```

### LoggingConfig

Configure file and console logging:

```rust
use stood::telemetry::{init_logging, LoggingConfig};

let config = LoggingConfig {
    log_dir: std::path::PathBuf::from("./logs"),
    max_file_size: 10 * 1024 * 1024,  // 10MB
    max_files: 5,
    file_log_level: "DEBUG".to_string(),
    console_log_level: "INFO".to_string(),
    console_enabled: true,
    json_format: true,
    enable_performance_tracing: true,
    enable_cycle_detection: true,
};

let _guard = init_logging(config)?;
```

### PerformanceTracer

Track operation timing and performance:

```rust
use stood::telemetry::PerformanceTracer;
use std::time::Duration;

let tracer = PerformanceTracer::new();

// Start an operation
let guard = tracer.start_operation("my_operation");
guard.add_context("key", "value");

// Do work...

guard.checkpoint("midpoint");

// Do more work...

// Guard automatically records completion when dropped
```

## GenAI Semantic Conventions

Stood follows OpenTelemetry GenAI semantic conventions:

```rust
use stood::telemetry::semantic_conventions::*;

// Model attributes
GEN_AI_SYSTEM                    // "gen_ai.system"
GEN_AI_REQUEST_MODEL             // "gen_ai.request.model"
GEN_AI_REQUEST_MAX_TOKENS        // "gen_ai.request.max_tokens"
GEN_AI_REQUEST_TEMPERATURE       // "gen_ai.request.temperature"

// Usage attributes
GEN_AI_USAGE_INPUT_TOKENS        // "gen_ai.usage.input_tokens"
GEN_AI_USAGE_OUTPUT_TOKENS       // "gen_ai.usage.output_tokens"

// Operation attributes
GEN_AI_OPERATION_NAME            // "gen_ai.operation.name"
GEN_AI_TOOL_NAME                 // "gen_ai.tool.name"

// Stood-specific attributes
STOOD_AGENT_ID                   // "stood.agent.id"
STOOD_CONVERSATION_ID            // "stood.conversation.id"
STOOD_CYCLE_ID                   // "stood.cycle.id"
```

## Examples

### 009_logging_demo

Demonstrates file logging and performance tracing:

```bash
cargo run --example 009_logging_demo
```

### 023_telemetry Examples

Several telemetry examples are available in `examples/023_telemetry/`:

| Example | Description |
|---------|-------------|
| `simple_telemetry_test` | Basic telemetry initialization |
| `smart_telemetry_test` | Auto-detection and fallback behavior |
| `metrics_test` | Metrics collection with agent |

## See Also

- [Architecture](architecture.md) - Overall system design
- [Examples](examples.md) - Usage examples and tutorials
- [Source Code](../src/telemetry/mod.rs) - Telemetry module implementation
