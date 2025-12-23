# Telemetry and Observability

Stood provides enterprise-grade observability with OpenTelemetry integration, supporting comprehensive monitoring of AI agent performance, distributed tracing, and metrics collection.

## Quick Start

### Basic Telemetry Setup

```rust
use stood::agent::Agent;
use stood::telemetry::TelemetryConfig;
use stood::llm::models::Bedrock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Option 1: Explicit configuration
    let config = TelemetryConfig::default()
        .with_otlp_endpoint("http://localhost:4318")
        .with_service_name("my-agent");
    
    let agent = Agent::builder()
        .model(Bedrock::ClaudeHaiku45)
        .with_telemetry(config)
        .build().await?;

    // Option 2: Environment-based with auto-detection
    let agent = Agent::builder()
        .model(Bedrock::ClaudeHaiku45)
        .with_telemetry_from_env()
        .build().await?;

    Ok(())
}
```

**Telemetry is disabled by default** - you must explicitly enable it using one of the builder methods.

## Smart Auto-Detection

When using `.with_telemetry_from_env()`, Stood includes intelligent endpoint discovery:

### How Auto-Detection Works

1. **Environment Variable Check**: First checks for explicit `OTEL_EXPORTER_OTLP_ENDPOINT` configuration
2. **Port Scanning**: Tests common OTLP ports (4318, 4320) on localhost and docker containers  
3. **TCP Connection Test**: Performs quick TCP connection tests with 100ms timeout
4. **Graceful Degradation**: Falls back to console export (dev) or disables telemetry (prod)

### Debug Output
Auto-detection provides detailed logging:
```
🔍 Stood: Starting OTLP endpoint detection...
🔍 Stood: Testing endpoint: http://localhost:4318
❌ Stood: TCP connection failed to localhost:4318 - Connection refused
🔍 Stood: Testing endpoint: http://localhost:4320  
✅ Stood: TCP connection successful to localhost:4320
🎯 Stood: Auto-detected OTLP endpoint: http://localhost:4320
```

## Environment Variables

```bash
# Core Configuration
OTEL_ENABLED=false                 # Disable telemetry entirely  
OTEL_EXPORTER_OTLP_ENDPOINT       # Custom OTLP endpoint
OTEL_SERVICE_NAME                 # Service name for traces

# Advanced Configuration
STOOD_OTEL_ENABLE_CONSOLE_EXPORT  # Force console export
OTEL_BATCH_PROCESSOR              # Enable batch processing
STOOD_OTEL_DEBUG_TRACING          # Enable detailed debug tracing
STOOD_OTEL_EXPORT_MODE            # Export mode: "simple", "batch"
```

## OTLP Debug Logging

Stood includes comprehensive OTLP debug logging to troubleshoot telemetry issues:

**Log Location**: `~/.local/share/stood-telemetry/logs/otlp_exports.jsonl`

### What Gets Logged

1. **Telemetry Initialization** - Configuration, endpoint detection, tracer creation
2. **OTLP Export Attempts** - All attempts to send traces/metrics/logs to OTLP endpoints
3. **Span Operations** - Span creation, attribute setting, events, completion
4. **Endpoint Detection** - Auto-detection attempts and TCP connection tests

### Log Format
```json
{
  "timestamp": "2025-07-01T02:35:13.166Z",
  "module": "telemetry::otel::StoodTracer::init", 
  "export_type": "traces",
  "endpoint": "http://localhost:4320",
  "payload_summary": "OTLP exporter initialization",
  "payload_size_bytes": 0,
  "success": true,
  "error": null,
  "thread_id": "ThreadId(1)"
}
```

### Troubleshooting Steps

When telemetry issues occur:

1. **Check the debug log**: `cat ~/.local/share/stood-telemetry/logs/otlp_exports.jsonl`
2. **Look for initialization failures**: Search for `"success": false` entries
3. **Verify endpoint detection**: Look for auto-detection attempts and TCP connection results
4. **Check export attempts**: Ensure spans/metrics are being sent to the correct endpoints

### Common Issues

- **No endpoints detected**: Auto-detection failed, manually set `OTEL_EXPORTER_OTLP_ENDPOINT`
- **Connection refused**: OTLP collector not running or wrong port
- **Exporter build failed**: Invalid endpoint URL or network issues
- **No export attempts**: Telemetry disabled or tracer not properly initialized

## Configuration Options

### Production Configuration

```rust
use stood::telemetry::TelemetryConfig;

let config = TelemetryConfig::default()
    .with_otlp_endpoint("https://api.honeycomb.io")
    .with_service_name("production-agent")
    .with_service_version("1.0.0")
    .with_batch_processing()  // Higher throughput
    .with_log_level(LogLevel::WARN);  // Reduce noise
```

### Development Configuration

```rust
let config = TelemetryConfig::default()
    .with_console_export()  // Local debugging
    .with_simple_processing()  // Immediate export
    .with_log_level(LogLevel::DEBUG);
```

### Testing Configuration

```rust
let config = TelemetryConfig::for_testing();
```

## Collected Metrics

### Agent Performance
- `stood_agent_cycles_total` - Total number of agent execution cycles
- `stood_agent_cycle_duration_seconds` - Agent cycle duration histogram
- `stood_agent_errors_total` - Agent execution errors by type

### Model Interactions  
- `stood_model_tokens_input_total` - Input tokens consumed
- `stood_model_tokens_output_total` - Output tokens generated
- `stood_model_request_duration_seconds` - Model request latency
- `stood_model_requests_total` - Total model requests by model type

### Tool Execution
- `stood_tool_executions_total` - Tool executions by name and status
- `stood_tool_execution_duration_seconds` - Tool execution time
- `stood_tool_errors_total` - Tool execution errors by tool name

## GenAI Semantic Conventions

Stood follows OpenTelemetry GenAI semantic conventions for AI workload observability:

### Core Attributes
```rust
use stood::telemetry::semantic_conventions::*;

// Model attributes
GEN_AI_SYSTEM                    // "anthropic.bedrock"
GEN_AI_REQUEST_MODEL            // "claude-3-5-haiku-20241022"
GEN_AI_REQUEST_MAX_TOKENS       // 4096
GEN_AI_REQUEST_TEMPERATURE      // 0.7

// Usage attributes  
GEN_AI_USAGE_INPUT_TOKENS       // 150
GEN_AI_USAGE_OUTPUT_TOKENS      // 75
GEN_AI_USAGE_TOTAL_TOKENS       // 225

// Operation attributes
GEN_AI_OPERATION_NAME           // "agent_cycle", "tool_call"
GEN_AI_TOOL_NAME               // "calculator", "weather"
```

### Stood-Specific Attributes
```rust
// Agent identification
STOOD_AGENT_ID                  // Unique agent instance ID
STOOD_CONVERSATION_ID           // Conversation session ID
STOOD_CYCLE_ID                 // Individual cycle ID

// Performance tracking  
STOOD_STREAMING_ENABLED         // Boolean streaming status
STOOD_RETRY_ATTEMPT            // Retry attempt number
STOOD_MODEL_SUPPORTS_TOOLS     // Tool capability flag
```

## Cloud Provider Integration

Based on environment variables, Stood can auto-detect some cloud provider endpoints:

### Honeycomb

```bash
export HONEYCOMB_API_KEY=your-key
export OTEL_SERVICE_NAME=my-agent
```

### New Relic

```bash
export NEW_RELIC_LICENSE_KEY=your-key
export OTEL_SERVICE_NAME=my-agent
```

### Datadog

```bash
export DD_API_KEY=your-key
export DD_SITE=datadoghq.com  # or datadoghq.eu
```

### AWS X-Ray

⚠️ **Auto-detection implemented but not extensively tested**

```bash
export AWS_REGION=us-east-1  # or your region
export AWS_ACCESS_KEY_ID=your-key
export AWS_SECRET_ACCESS_KEY=your-secret
# OR
export AWS_PROFILE=your-profile
```

### Google Cloud Trace

⚠️ **Auto-detection implemented but not extensively tested**

```bash
export GOOGLE_CLOUD_PROJECT=your-project-id
# Requires proper GCP authentication (service account, gcloud auth, etc.)
```

### Custom Enterprise Endpoints

⚠️ **Advanced feature - not extensively tested**

```bash
# Comma-separated list of endpoints (tested for availability)
export STOOD_OTLP_ENDPOINTS=https://otlp.company.com:4318,https://backup.company.com:4318
```

## Prometheus Integration

### Key Metrics Queries

```promql
# Agent performance
rate(stood_agent_cycles_total[5m])
histogram_quantile(0.95, rate(stood_agent_cycle_duration_seconds_bucket[5m]))

# Token consumption
rate(stood_model_tokens_input_total[5m])
rate(stood_model_tokens_output_total[5m])

# Tool execution
rate(stood_tool_executions_total[5m])
stood_tool_execution_duration_seconds

# Error tracking
rate(stood_agent_errors_total[5m])
```

### Alerting Rules

```yaml
groups:
  - name: stood-agent-alerts
    rules:
      - alert: HighErrorRate
        expr: rate(stood_agent_errors_total[5m]) / rate(stood_agent_cycles_total[5m]) > 0.05
        for: 2m
        annotations:
          summary: "High error rate detected"
          
      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(stood_model_request_duration_seconds_bucket[5m])) > 10
        for: 5m
        annotations:
          summary: "High model request latency"
```

## Complete Demo

See the comprehensive telemetry demo at `examples/023_telemetry/` which includes:

- **Full Observability Stack**: Prometheus, Grafana, Jaeger, OpenTelemetry Collector
- **Pre-configured Dashboards**: Agent performance, token usage, error tracking
- **Docker Compose Setup**: One-command deployment
- **AWS Integration Examples**: CloudWatch and X-Ray configuration
- **Production Patterns**: Batching, sampling, alerting

### Quick Demo Start

```bash
cd examples/023_telemetry
./setup-telemetry.sh
cargo run --bin telemetry_demo
```

Then visit:
- **Grafana**: http://localhost:3000 (admin/admin)
- **Prometheus**: http://localhost:9090
- **Jaeger**: http://localhost:16686

## See Also

- [Architecture](architecture.md) - Overall system design
- [Examples](examples.md) - Usage examples and tutorials