# Telemetry and Observability

Stood provides observability for AI agent performance monitoring, including CloudWatch Gen AI integration, file logging, performance tracing, and metrics collection.

## Overview

The telemetry module supports:

- **CloudWatch Gen AI Observability** - Production-ready integration with AWS CloudWatch for GenAI dashboards
- **File logging** - Via `LoggingConfig` and `PerformanceTracer`
- **Metrics types** - `EventLoopMetrics`, `CycleMetrics`, `TokenUsage` for tracking
- **Smart truncation** - Automatic handling of large prompts/responses to stay within CloudWatch limits
- **Batch splitting** - Automatic splitting of log batches exceeding 1MB

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

### With CloudWatch Gen AI Observability

```rust
use stood::agent::Agent;
use stood::telemetry::TelemetryConfig;
use stood::llm::models::Bedrock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure CloudWatch telemetry
    let telemetry_config = TelemetryConfig::cloudwatch("us-east-1")
        .with_service_name("my-agent-service")
        .with_agent_id("my-agent-001")
        .with_content_capture(true);  // Enable content capture for evaluations

    let mut agent = Agent::builder()
        .name("My Agent")
        .model(Bedrock::ClaudeHaiku45)
        .with_telemetry(telemetry_config)
        .build().await?;

    let result = agent.execute("What is 2+2?").await?;
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

## CloudWatch Gen AI Observability

Stood integrates with AWS CloudWatch Gen AI Observability for production monitoring of AI agent workloads.

### Configuration Options

```rust
use stood::telemetry::{TelemetryConfig, AwsCredentialSource};

// Basic CloudWatch configuration
let config = TelemetryConfig::cloudwatch("us-east-1");

// With custom service name and agent ID
let config = TelemetryConfig::cloudwatch("us-east-1")
    .with_service_name("qanda-service")
    .with_agent_id("qanda-agent-001");

// With custom credentials
let config = TelemetryConfig::cloudwatch_with_credentials(
    "us-east-1",
    AwsCredentialSource::Profile("production".to_string())
);

// Enable content capture for evaluations
let config = TelemetryConfig::cloudwatch("us-east-1")
    .with_content_capture(true);

// From environment variables
let config = TelemetryConfig::from_env();
```

### Key Identifiers

Two identifiers are important for CloudWatch integration:

| Identifier | Purpose | Example |
|------------|---------|---------|
| `service_name` | Your application name (OpenTelemetry service.name) | `qanda-service` |
| `agent_id` | Unique agent identifier for log group naming | `qanda-agent-001` |

The agent_id is used to construct the CloudWatch Log Group path:
```
/aws/bedrock-agentcore/runtimes/{agent_id}
```

### GenAI Semantic Conventions

Stood follows OpenTelemetry GenAI semantic conventions:

| Span Name | Operation | Key Attributes |
|-----------|-----------|----------------|
| `invoke_agent {name}` | Agent invocation | `gen_ai.agent.name`, `gen_ai.usage.*` |
| `chat {model}` | Model call | `gen_ai.request.model`, `gen_ai.provider.name` |
| `execute_tool {name}` | Tool execution | `gen_ai.tool.name`, `gen_ai.tool.type` |

### AWS Prerequisites

1. Configure AWS credentials (environment, profile, or IAM role)
2. Enable Transaction Search in CloudWatch Console
3. Set trace destination:
   ```bash
   aws xray update-trace-segment-destination --destination CloudWatchLogs
   ```
4. Attach required IAM permissions:
   ```json
   {
     "Effect": "Allow",
     "Action": [
       "logs:CreateLogGroup",
       "logs:CreateLogStream",
       "logs:PutLogEvents",
       "logs:DescribeLogGroups",
       "logs:DescribeLogStreams",
       "xray:PutTraceSegments",
       "xray:PutTelemetryRecords"
     ],
     "Resource": "*"
   }
   ```

## Smart Truncation

Large prompts and responses are automatically truncated to stay within CloudWatch's 1MB batch limit. The truncation system:

- **Preserves context** - Keeps the beginning (system prompt, user query) and end (final response) of content
- **UTF-8 safe** - Truncates at valid character boundaries to avoid invalid UTF-8 sequences
- **Transparent** - Inserts a marker showing how much content was removed

### Truncation Limits

| Limit | Value | Description |
|-------|-------|-------------|
| `MAX_CONTENT_FIELD_SIZE` | 32KB | Maximum size per content field before truncation |
| `TRUNCATION_HEAD_SIZE` | ~14KB | Content preserved at the start |
| `TRUNCATION_TAIL_SIZE` | ~14KB | Content preserved at the end |
| `MAX_CLOUDWATCH_BATCH_SIZE` | 950KB | Maximum batch size (1MB limit with headroom) |

### Truncation Marker

When content is truncated, a marker is inserted:
```
[... TRUNCATED 45230 bytes (44.2KB) ...]
```

This preserves the ability to evaluate input requirements and output quality while staying within CloudWatch limits.

## Environment Variables

```bash
# CloudWatch Configuration
STOOD_CLOUDWATCH_ENABLED=true         # Enable CloudWatch export
AWS_REGION=us-east-1                  # AWS region
OTEL_SERVICE_NAME=stood-agent         # Service name in traces
STOOD_AGENT_ID=my-agent-001           # Agent ID for log group naming
STOOD_GENAI_CONTENT_CAPTURE=true      # Capture message content

# Legacy Variables (still supported)
OTEL_ENABLED=true                     # Enable telemetry

# Logging
RUST_LOG=stood=info                   # Log level filter
```

## Available Types

### TelemetryConfig

Configuration for telemetry and observability:

```rust
use stood::telemetry::{TelemetryConfig, LogLevel};

// Disabled (default)
let config = TelemetryConfig::disabled();
assert!(!config.is_enabled());

// CloudWatch with region
let config = TelemetryConfig::cloudwatch("us-east-1");
assert!(config.is_enabled());

// Full configuration
let config = TelemetryConfig::cloudwatch("us-east-1")
    .with_service_name("my-service")
    .with_service_version("1.0.0")
    .with_agent_id("my-agent-001")
    .with_content_capture(false)
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

### 025_cloudwatch_observability

Demonstrates CloudWatch Gen AI integration:

```bash
# With telemetry disabled (default)
cargo run --example 025_cloudwatch_observability

# With telemetry enabled
STOOD_CLOUDWATCH_ENABLED=true cargo run --example 025_cloudwatch_observability
```

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
| `performance_benchmark` | Performance benchmarking |

## Testing CloudWatch Integration

Integration tests verify real AWS connectivity:

```bash
# Run CloudWatch integration tests (requires AWS credentials)
cargo test --test telemetry_cloudwatch_integration

# Run core safety tests
cargo test --test core_safety_telemetry_tests
```

## See Also

- [Architecture](architecture.md) - Overall system design
- [Examples](examples.md) - Usage examples and tutorials
- [CLOUDWATCH_GENAI_IMPLEMENTATION_GUIDE.md](CLOUDWATCH_GENAI_IMPLEMENTATION_GUIDE.md) - Detailed implementation guide
- [Source Code](../src/telemetry/mod.rs) - Telemetry module implementation
