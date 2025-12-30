# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the Stood agent library - a Rust implementation of an AI agent framework with multi providers support. The project consists of:

1. **Rust Library** (`src/`) - The main Stood agent library implementation
1. **Provider Integration** (`tests/provider_integration/`) - Test suite to verify LLM provider functionality
1. **Documentation** (`docs/`) - Project documentation
1. **Examples** (`examples/`) - Fully functional examples for developers and code assistants
1. **Stood Macros** (`stood-macros/`) - Procedural macros that are part of Stood

## Commands

### Rust Development
```bash
# Build the library
cargo build

# Run tests (includes integration tests that make real AWS Bedrock API calls)
cargo test

# Check for compilation errors without building
cargo check

# Format code
cargo fmt

# Run clippy linter
cargo clippy

# Build documentation
cargo doc --open

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Testing

### Integration Tests
Integration tests run automatically with `cargo test` and make real provider API calls. They require:
- Valid credentials (verified with test above)
- Internet connectivity

**IMPORTANT: Do not modify integration tests to require environment variables or flags. They should run by default with `cargo test`.**

## Architecture

### High-Level Design
The Stood library is designed with these core components:

1. **Provider Client** - Direct provider integration for Claude/Nova and other models
2. **Tools Component** - Rust function integration with compile-time validation
3. **Agent Component** - Orchestrates the agentic loop between providers and tools
4. **MCP Component** - Model Context Protocol support
5. **Telemetry Component** - CloudWatch Gen AI Observability integration

### Key Design Principles
- **Library-First**: Designed to be embedded in other Rust applications
- **Performance Optimized**: Leverages Rust's zero-cost abstractions
- **Type Safety**: Strong typing throughout to prevent runtime errors
- **Model Compatibility**: Defaults to Claude Haiku 4.5 for production use

## Telemetry

### CloudWatch Gen AI Observability

Stood integrates with AWS CloudWatch Gen AI Observability for production monitoring.

#### Configuration

```rust
use stood::agent::Agent;
use stood::telemetry::TelemetryConfig;

// Disabled (default)
let agent = Agent::builder()
    .model(Bedrock::ClaudeHaiku45)
    .build()
    .await?;

// Enabled with CloudWatch
let agent = Agent::builder()
    .model(Bedrock::ClaudeHaiku45)
    .with_telemetry(TelemetryConfig::cloudwatch("us-east-1"))
    .build()
    .await?;

// From environment
let agent = Agent::builder()
    .model(Bedrock::ClaudeHaiku45)
    .with_telemetry_from_env()
    .build()
    .await?;
```

#### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `STOOD_CLOUDWATCH_ENABLED` | `false` | Enable CloudWatch export |
| `AWS_REGION` | `us-east-1` | AWS region |
| `OTEL_SERVICE_NAME` | `stood-agent` | Service name in traces |

#### AWS Prerequisites

1. Configure AWS credentials (environment, profile, or IAM role)
2. Enable Transaction Search in CloudWatch Console
3. Set trace destination: `aws xray update-trace-segment-destination --destination CloudWatchLogs`
4. Attach `AWSXrayWriteOnlyPolicy` or equivalent IAM policy

#### GenAI Semantic Conventions

Stood follows OpenTelemetry GenAI semantic conventions:

| Span Name | Operation | Key Attributes |
|-----------|-----------|----------------|
| `invoke_agent {name}` | Agent invocation | `gen_ai.agent.name`, `gen_ai.usage.*` |
| `chat {model}` | Model call | `gen_ai.request.model`, `gen_ai.provider.name` |
| `execute_tool {name}` | Tool execution | `gen_ai.tool.name`, `gen_ai.tool.type` |

### Live Examples
Review examples for working versions of the API:
- `examples/025_cloudwatch_observability.rs` - Full CloudWatch integration
- `examples/023_telemetry/` - Telemetry configuration tests 
