# AWS CloudWatch Gen AI Observability: Knowledge Base

**Author:** Claude Code Research
**Date:** December 2025
**Purpose:** Comprehensive technical reference for implementing CloudWatch Gen AI Observability in Stood

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [CloudWatch Gen AI Observability Overview](#cloudwatch-gen-ai-observability-overview)
3. [OpenTelemetry Semantic Conventions for GenAI](#opentelemetry-semantic-conventions-for-genai)
4. [AWS OTLP Endpoints Configuration](#aws-otlp-endpoints-configuration)
5. [Rust Implementation Details](#rust-implementation-details)
6. [AgentCore Observability Structure](#agentcore-observability-structure)
7. [Current Stood Implementation Analysis](#current-stood-implementation-analysis)
8. [Gap Analysis](#gap-analysis)
9. [Sources and References](#sources-and-references)

---

## Executive Summary

AWS CloudWatch Gen AI Observability (GA October 2025) provides purpose-built monitoring for generative AI applications. It works with any framework that emits OpenTelemetry-compatible telemetry data, making it accessible to Stood without requiring AgentCore runtime hosting.

**Key Integration Path:** Stood can publish directly to CloudWatch OTLP endpoints using:
- Standard OpenTelemetry traces with Gen AI semantic conventions
- AWS SigV4 authentication via the `otlp-sigv4-client` Rust crate
- HTTP/Protobuf protocol to X-Ray and CloudWatch Logs endpoints

---

## CloudWatch Gen AI Observability Overview

### What It Is

Amazon CloudWatch Gen AI Observability is a monitoring solution that provides:
- **Model Invocations Dashboard**: Metrics on model usage, token consumption, costs
- **AgentCore Agents Dashboard**: Performance metrics for agents, memory, tools, gateways
- **End-to-end Tracing**: Visibility across LLMs, agents, knowledge bases, and tools

### Key Features

| Feature | Description |
|---------|-------------|
| Token Tracking | Input/output tokens, total consumption, per-query averages |
| Latency Metrics | Average, P90, P99 latency percentiles |
| Cost Attribution | By application, user role, or specific user |
| Error Tracking | Error rates, throttling events, failure analysis |
| Tool Monitoring | Built-in tools, gateways, memory operations |

### Supported Frameworks

CloudWatch Gen AI Observability works with:
- **AWS Strands Agents** (native support)
- **LangChain / LangGraph** (auto-instrumentation)
- **CrewAI** (auto-instrumentation)
- **Any OTEL-compatible framework** (including Stood)

### Regional Availability

Available in: US East (N. Virginia, Ohio), US West (Oregon), Europe (Frankfurt, Ireland), Asia Pacific (Mumbai, Tokyo, Singapore, Sydney)

### Pricing

No additional pricing for Gen AI Observability - standard CloudWatch pricing for underlying telemetry data applies.

**Sources:**
- [Launching Amazon CloudWatch generative AI observability (Preview)](https://aws.amazon.com/blogs/mt/launching-amazon-cloudwatch-generative-ai-observability-preview/)
- [Generative AI observability now generally available](https://aws.amazon.com/about-aws/whats-new/2025/10/generative-ai-observability-amazon-cloudwatch/)
- [CloudWatch Gen AI Observability Features](https://aws.amazon.com/cloudwatch/features/generative-ai-observability/)

---

## OpenTelemetry Semantic Conventions for GenAI

The OpenTelemetry community has defined standardized semantic conventions for GenAI systems. These are **critical** for CloudWatch Gen AI Observability to properly parse and display traces.

### Span Naming Conventions

| Span Type | Name Format | Span Kind |
|-----------|-------------|-----------|
| Inference | `{gen_ai.operation.name} {gen_ai.request.model}` | `CLIENT` |
| Embeddings | `embeddings {gen_ai.request.model}` | `CLIENT` |
| Execute Tool | `execute_tool {gen_ai.tool.name}` | `INTERNAL` |
| Create Agent | `create_agent {gen_ai.agent.name}` | `CLIENT` |
| Invoke Agent | `invoke_agent {gen_ai.agent.name}` | `CLIENT` |

### Core Required Attributes

```
gen_ai.operation.name    - Operation identifier (chat, generate_content, text_completion,
                           create_agent, invoke_agent, execute_tool, embeddings)
gen_ai.provider.name     - Provider identifier (aws.bedrock, anthropic, openai, etc.)
```

### Request Attributes (Recommended)

```
gen_ai.request.model              - Model name/ID being queried
gen_ai.request.max_tokens         - Maximum tokens to generate (int)
gen_ai.request.temperature        - Temperature setting (double, 0.0-2.0)
gen_ai.request.top_p              - Top-p sampling parameter (double)
gen_ai.request.top_k              - Top-k sampling parameter (double)
gen_ai.request.frequency_penalty  - Frequency penalty (double)
gen_ai.request.presence_penalty   - Presence penalty (double)
gen_ai.request.seed               - Seed for reproducibility (int)
gen_ai.request.choice.count       - Number of candidate completions (int)
gen_ai.request.stop_sequences     - Stop sequences (string[])
```

### Response Attributes (Recommended)

```
gen_ai.response.id              - Unique completion identifier
gen_ai.response.model           - Model that generated response
gen_ai.response.finish_reasons  - Array of stop reasons (string[])
```

### Usage Attributes (Recommended)

```
gen_ai.usage.input_tokens   - Tokens in the input (int)
gen_ai.usage.output_tokens  - Tokens in the output (int)
```

### Agent-Specific Attributes

```
gen_ai.agent.id           - Unique agent identifier
gen_ai.agent.name         - Human-readable agent name
gen_ai.agent.description  - Free-form agent description
gen_ai.conversation.id    - Session/thread identifier
gen_ai.data_source.id     - External storage identifier
```

### Tool Execution Attributes

```
gen_ai.tool.name          - Tool name
gen_ai.tool.type          - Tool type (function, extension, datastore)
gen_ai.tool.description   - Tool purpose description
gen_ai.tool.call.id       - Unique tool call identifier
gen_ai.tool.call.arguments - Parameters passed to tool (sensitive)
gen_ai.tool.call.result    - Tool execution result (sensitive)
gen_ai.tool.definitions    - Available tool definitions array
```

### Content Attributes (Opt-In, Sensitive)

```
gen_ai.input.messages      - Chat history provided to model (JSON)
gen_ai.output.messages     - Messages returned by model (JSON)
gen_ai.system_instructions - System prompts (JSON)
gen_ai.output.type         - Output format (text, json, image, speech)
```

### Well-Known Provider Values

```
openai, anthropic, aws.bedrock, azure.ai.openai, azure.ai.inference,
gcp.vertex_ai, gcp.gemini, gcp.gen_ai, cohere, mistral_ai, groq,
deepseek, perplexity, x_ai, ibm.watsonx.ai
```

### Well-Known Operation Values

```
chat, embeddings, text_completion, generate_content,
execute_tool, create_agent, invoke_agent
```

### Deprecated Attributes (Avoid)

```
gen_ai.system              -> use gen_ai.provider.name
gen_ai.prompt              -> use Event API
gen_ai.completion          -> use Event API
gen_ai.usage.prompt_tokens -> use gen_ai.usage.input_tokens
gen_ai.usage.completion_tokens -> use gen_ai.usage.output_tokens
```

**Stability Note:** All GenAI semantic conventions are marked as **Development** status. Use `OTEL_SEMCONV_STABILITY_OPT_IN=gen_ai_latest_experimental` for latest version.

**Sources:**
- [Semantic conventions for generative AI systems](https://opentelemetry.io/docs/specs/semconv/gen-ai/)
- [Semantic conventions for generative client AI spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-spans/)
- [Semantic Conventions for GenAI agent and framework spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/)
- [Gen AI Attributes Registry](https://opentelemetry.io/docs/specs/semconv/registry/attributes/gen-ai/)

---

## AWS OTLP Endpoints Configuration

### Endpoint URLs

| Signal | Endpoint Pattern | Example |
|--------|------------------|---------|
| Traces | `https://xray.{region}.amazonaws.com/v1/traces` | `https://xray.us-east-1.amazonaws.com/v1/traces` |
| Logs | `https://logs.{region}.amazonaws.com/v1/logs` | `https://logs.us-east-1.amazonaws.com/v1/logs` |

### Authentication

All endpoints require **AWS Signature Version 4 (SigV4)** authentication.

Required IAM Policy for Traces:
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": ["xray:PutTraceSegments", "xray:PutTelemetryRecords"],
    "Resource": "*"
  }]
}
```

Or attach `AWSXrayWriteOnlyPolicy` managed policy.

Required IAM Policy for Logs:
```json
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Action": ["logs:PutLogEvents", "logs:DescribeLogGroups", "logs:DescribeLogStreams"],
    "Resource": ["arn:aws:logs:*:*:log-group:*"]
  }]
}
```

### Protocol Configuration

| Setting | Value |
|---------|-------|
| Protocol | HTTP 1.1 (gRPC NOT supported) |
| OTLP Version | 1.x |
| Payload Formats | Binary (protobuf), JSON |
| Compression | gzip, none |

### Traces Endpoint Limits

| Limit | Value |
|-------|-------|
| Max uncompressed bytes/request | 5 MB |
| Max spans/request | 10,000 |
| Single span max size | 200 KB |
| Single resource & scope size | 16 KB |
| Timestamp range | 2 hours future / 14 days past |
| Max time gap in single request | 24 hours |

### Logs Endpoint Limits

| Limit | Value |
|-------|-------|
| Max uncompressed bytes/request | 1 MB (20 MB in select regions) |
| Max logs/request | 10,000 |
| Single LogEvent size | 1 MB (not changeable) |
| Requests per second | 5,000 per account/region |

### Required Headers for Logs

```
x-aws-log-group: <CloudWatch Log Group name>
x-aws-log-stream: <CloudWatch Log Stream name>
```

### Prerequisites

1. **Enable Transaction Search** in CloudWatch console (required for spans)
2. **Pre-create Log Groups and Streams** for logs export

**Sources:**
- [OTLP Endpoints - Amazon CloudWatch](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-OTLPEndpoint.html)
- [Exporting collector-less telemetry using ADOT SDK](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-OTLP-UsingADOT.html)
- [OpenTelemetry Protocol (OTLP) Endpoint - AWS X-Ray](https://docs.aws.amazon.com/xray/latest/devguide/xray-opentelemetry.html)

---

## Rust Implementation Details

### otlp-sigv4-client Crate

The `otlp-sigv4-client` crate provides AWS SigV4 signing for OpenTelemetry OTLP exporters in Rust.

**Crate:** [otlp-sigv4-client](https://docs.rs/otlp-sigv4-client/latest/otlp_sigv4_client/)

**Dependencies:**
```toml
[dependencies]
otlp-sigv4-client = "0.x"
aws-config = "1.x"
aws-credential-types = "1.x"
opentelemetry-otlp = { version = "0.17", features = ["http-proto", "reqwest-client"] }
```

**Usage Pattern:**
```rust
use otlp_sigv4_client::SigV4ClientBuilder;
use opentelemetry_otlp::HttpExporterBuilder;

// Load AWS credentials from environment
let aws_config = aws_config::load_from_env().await;
let credentials = aws_config.credentials_provider().unwrap();

// Create SigV4-signing HTTP client
let sigv4_client = SigV4ClientBuilder::new()
    .with_client(ReqwestClient::new())
    .with_credentials(credentials)
    .with_region("us-east-1")
    .with_service("xray")  // or "logs" for CloudWatch Logs
    .build()?;

// Configure OTLP exporter with SigV4 client
let exporter = HttpExporterBuilder::default()
    .with_http_client(sigv4_client)
    .with_endpoint("https://xray.us-east-1.amazonaws.com/v1/traces")
    .build_span_exporter()?;
```

### OpenTelemetry Rust SDK

**Key Crates:**
```toml
opentelemetry = "0.24"
opentelemetry-otlp = { version = "0.17", features = ["metrics", "http-proto", "reqwest-client"] }
opentelemetry_sdk = { version = "0.24", features = ["rt-tokio"] }
opentelemetry-semantic-conventions = "0.16"
```

**Sources:**
- [otlp-sigv4-client - Rust](https://docs.rs/otlp-sigv4-client/latest/otlp_sigv4_client/)
- [OpenTelemetry Rust - Exporters](https://opentelemetry.io/docs/languages/rust/exporters/)
- [opentelemetry-otlp - crates.io](https://crates.io/crates/opentelemetry-otlp)
- [opentelemetry-rust GitHub](https://github.com/open-telemetry/opentelemetry-rust)

---

## AgentCore Observability Structure

### Hierarchical Model

```
Session (Complete conversation context)
├── Trace 1 (Single request-response cycle)
│   ├── Span 1 (Operation: "process user query")
│   │   ├── Span 1.1 (Child: "parse input")
│   │   ├── Span 1.2 (Child: "retrieve context")
│   │   ├── Span 1.3 (Child: "generate response")
│   │   └── Span 1.4 (Child: "format output")
│   └── Span 2 (Tool invocation)
│       ├── Span 2.1 (Child: "validate parameters")
│       └── Span 2.2 (Child: "execute tool logic")
└── Trace 2 (Next request-response cycle)
    └── ...
```

### Session Level

- Unique identifier per session
- Context persistence across interactions
- Conversation history tracking
- State management for user-specific information

### Trace Level

- Complete request-response cycle
- Associated with specific session
- Contains:
  - Request details and timestamps
  - Processing steps sequence
  - Tool invocations
  - Resource utilization metrics
  - Error information
  - Response generation details

### Span Level

- Discrete, measurable unit of work
- Defined start and end times
- Contains:
  - Operation name
  - Parent-child relationships
  - Tags and attributes
  - Events (significant occurrences)
  - Status information

### Default Metrics in AgentCore

- Session-level metrics (Agent runtime)
- Memory resource metrics
- Gateway resource metrics
- Built-in tool metrics

**Sources:**
- [Understand observability for agentic resources in AgentCore](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/observability-telemetry.html)
- [Observe your agent applications on Amazon Bedrock AgentCore Observability](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/observability.html)
- [View observability data for your Amazon Bedrock AgentCore agents](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/observability-view.html)

---

## Current Stood Implementation Analysis

### Existing Telemetry Architecture

Stood already has a comprehensive telemetry system (~5,009 lines across 10 files):

| File | Lines | Purpose |
|------|-------|---------|
| `telemetry/mod.rs` | 1,260 | Main configuration, data structures |
| `telemetry/otel.rs` | 1,416 | OpenTelemetry integration |
| `telemetry/logging.rs` | 699 | Structured logging |
| `telemetry/metrics/` | 948 | Metrics collection |
| `telemetry/otlp_debug.rs` | 294 | OTLP debugging |
| `telemetry/test_harness.rs` | 328 | Testing utilities |

### Existing GenAI Semantic Conventions

Stood already implements many GenAI conventions:

**Request Attributes (Implemented):**
- `gen_ai.system` (deprecated, needs migration to `gen_ai.provider.name`)
- `gen_ai.request.model`
- `gen_ai.request.max_tokens`
- `gen_ai.request.temperature`
- `gen_ai.request.top_p`
- `gen_ai.request.top_k`
- `gen_ai.request.presence_penalty`
- `gen_ai.request.frequency_penalty`
- `gen_ai.request.streaming`
- `gen_ai.request.tools_count`
- `gen_ai.request.tools_names`

**Response Attributes (Implemented):**
- `gen_ai.response.id`
- `gen_ai.response.model`
- `gen_ai.response.finish_reasons`
- `gen_ai.response.type`
- `gen_ai.response.latency_ms`

**Usage Attributes (Implemented):**
- `gen_ai.usage.input_tokens`
- `gen_ai.usage.output_tokens`
- `gen_ai.usage.total_tokens`

### Current Span Types

1. **Agent Spans**: `agent.{operation}`
2. **Model Inference Spans**: `model.inference`
3. **Tool Execution Spans**: `tool.{tool_name}`
4. **Cycle Spans**: `agent.model_interaction`

### Current OTLP Export Capabilities

- HTTP/GRPC auto-detection
- Batch processing mode
- Smart endpoint discovery (localhost, Docker, cloud providers)
- Configurable export interval
- Compression support

### Missing for CloudWatch Gen AI Integration

1. **No AWS SigV4 authentication** for OTLP exports
2. **Span naming doesn't match OTEL spec** exactly
3. **Missing `gen_ai.provider.name`** (using deprecated `gen_ai.system`)
4. **Missing `gen_ai.operation.name`** attribute
5. **Missing agent-specific attributes** (`gen_ai.agent.id`, `gen_ai.agent.name`)
6. **Missing `gen_ai.conversation.id`** correlation
7. **Missing tool execution span format** (`execute_tool {tool_name}`)
8. **No CloudWatch-specific configuration mode**

---

## Gap Analysis

### Critical Gaps - ALL RESOLVED ✅

| Gap | Status | Implementation |
|-----|--------|----------------|
| SigV4 Authentication | ✅ Done | `src/telemetry/exporter.rs` with `otlp-sigv4-client` |
| `gen_ai.provider.name` | ✅ Done | Set to `"aws.bedrock"` on all spans |
| `gen_ai.operation.name` | ✅ Done | Set on all span types |
| Span naming format | ✅ Done | `{operation} {model}` format |

### High Priority Gaps - ALL RESOLVED ✅

| Gap | Status | Implementation |
|-----|--------|----------------|
| Agent spans | ✅ Done | `invoke_agent {agent_name}` |
| Tool spans | ✅ Done | `execute_tool {tool_name}` |
| Conversation tracking | ✅ Done | `gen_ai.conversation.id` on all spans |
| CloudWatch auto-config | ✅ Done | X-Ray endpoint auto-setup |

### Medium Priority Gaps - MOSTLY RESOLVED ✅

| Gap | Status | Implementation |
|-----|--------|----------------|
| `session.id` attribute | ✅ Done | Set on all spans and log events |
| `gen_ai.conversation.id` | ✅ Done | On all agent spans |
| `gen_ai.agent.id` | ✅ Done | Via `TelemetryConfig::with_agent_id()` |
| Agent attributes | ✅ Done | Standard `gen_ai.agent.*` |
| Tool attributes | ✅ Done | Full `gen_ai.tool.*` including input/output |
| Session correlation | ✅ Done | Session-Trace-Span hierarchy |
| Log correlation | ✅ Done | Log events linked via traceId/spanId |

### Remaining Work (Low Priority)

| Gap | Status | Notes |
|-----|--------|-------|
| Gen AI Dashboard | ⚠️ Partial | Requires Milestone 7 (log group creation) |
| Embeddings spans | Not needed | Stood doesn't use embeddings directly |

**Note:** The critical `session.id` discovery from earlier testing led to proper implementation. All spans and log events now include session correlation.

---

## Sources and References

### AWS Official Documentation

- [Launching Amazon CloudWatch generative AI observability (Preview)](https://aws.amazon.com/blogs/mt/launching-amazon-cloudwatch-generative-ai-observability-preview/)
- [Generative AI observability now generally available for Amazon CloudWatch](https://aws.amazon.com/about-aws/whats-new/2025/10/generative-ai-observability-amazon-cloudwatch/)
- [CloudWatch Gen AI Observability Features](https://aws.amazon.com/cloudwatch/features/generative-ai-observability/)
- [Generative AI observability - Amazon CloudWatch](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/GenAI-observability.html)
- [OTLP Endpoints - Amazon CloudWatch](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-OTLPEndpoint.html)
- [Exporting collector-less telemetry using ADOT SDK](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/CloudWatch-OTLP-UsingADOT.html)
- [OpenTelemetry Protocol (OTLP) Endpoint - AWS X-Ray](https://docs.aws.amazon.com/xray/latest/devguide/xray-opentelemetry.html)

### AgentCore Documentation

- [Observe your agent applications on Amazon Bedrock AgentCore Observability](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/observability.html)
- [Understand observability for agentic resources in AgentCore](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/observability-telemetry.html)
- [Add observability to your Amazon Bedrock AgentCore resources](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/observability-configure.html)
- [View observability data for your Amazon Bedrock AgentCore agents](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/observability-view.html)
- [Get started with AgentCore Observability](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/observability-get-started.html)
- [Build trustworthy AI agents with Amazon Bedrock AgentCore Observability](https://aws.amazon.com/blogs/machine-learning/build-trustworthy-ai-agents-with-amazon-bedrock-agentcore-observability/)

### OpenTelemetry Semantic Conventions

- [Semantic conventions for generative AI systems](https://opentelemetry.io/docs/specs/semconv/gen-ai/)
- [Semantic conventions for generative client AI spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-spans/)
- [Semantic Conventions for GenAI agent and framework spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/)
- [Semantic conventions for Generative AI events](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-events/)
- [Semantic conventions for generative AI metrics](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-metrics/)
- [Gen AI Attributes Registry](https://opentelemetry.io/docs/specs/semconv/registry/attributes/gen-ai/)

### Rust Implementation Resources

- [otlp-sigv4-client - Rust](https://docs.rs/otlp-sigv4-client/latest/otlp_sigv4_client/)
- [otlp-sigv4-client - Lib.rs](https://lib.rs/crates/otlp-sigv4-client)
- [OpenTelemetry Rust - Exporters](https://opentelemetry.io/docs/languages/rust/exporters/)
- [opentelemetry-otlp - crates.io](https://crates.io/crates/opentelemetry-otlp)
- [opentelemetry-rust GitHub](https://github.com/open-telemetry/opentelemetry-rust)
- [AWS Distro for OpenTelemetry](https://aws-otel.github.io/)

### Sample Code and Tutorials

- [GitHub: sample-amazon-cloudwatch-generative-ai-observability](https://github.com/aws-samples/sample-amazon-cloudwatch-generative-ai-observability)
- [Example: Use Application Signals to troubleshoot generative AI applications](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/Services-example-scenario-GenerativeAI.html)
- [Observability Quickstart - Amazon Bedrock AgentCore](https://aws.github.io/bedrock-agentcore-starter-toolkit/user-guide/observability/quickstart.html)

### Community and Blog Resources

- [Agentic AI Observability with Amazon CloudWatch - DEV Community](https://dev.to/acmopm/agentic-ai-observability-with-amazon-cloudwatch-transforming-enterprise-ai-monitoring-for-the-28k6)
- [CloudWatch for Agentic AI observability - goml.io](https://www.goml.io/blog/cloudwatch-for-agentic-ai-observability)
- [OpenTelemetry for GenAI and the OpenLLMetry project - Medium](https://horovits.medium.com/opentelemetry-for-genai-and-the-openllmetry-project-81b9cea6a771)
- [Datadog LLM Observability supports OpenTelemetry GenAI Semantic Conventions](https://www.datadoghq.com/blog/llm-otel-semantic-convention/)
- [How to monitor Amazon Bedrock AgentCore AI agent infrastructure in Grafana Cloud](https://grafana.com/blog/2025/11/28/how-to-monitor-amazon-bedrock-agentcore-ai-agent-infrastructure-in-grafana-cloud/)

---

## Auto-Instrumentation vs Manual Implementation

This section documents what the AgentCore SDK / ADOT auto-instruments automatically versus what Stood must implement manually to appear in the Gen AI Observability dashboard.

### What AgentCore SDK Auto-Instruments (Python)

When using `aws-opentelemetry-distro` with `opentelemetry-instrument`, the following are **automatically handled**:

| Capability | Auto-Instrumented | Notes |
|------------|-------------------|-------|
| OpenTelemetry SDK initialization | ✅ | Full SDK setup |
| Strands/LangChain/CrewAI spans | ✅ | Via framework instrumentors |
| Bedrock model invocation spans | ✅ | Via AWS SDK instrumentation |
| Tool execution spans | ✅ | Via framework instrumentors |
| Database/HTTP calls | ✅ | Via standard OTEL instrumentors |
| Token usage metrics | ✅ | Captured from model responses |
| SigV4 authentication | ✅ | Built into ADOT |
| OTLP export to CloudWatch | ✅ | Auto-configured endpoints |

### What Stood Must Implement Manually

Since Stood is a Rust library (not using Python ADOT), we must manually implement:

| Capability | Status | Implementation Required |
|------------|--------|------------------------|
| SigV4 authentication | ✅ Done | Custom HTTP client with AWS SigV4 signing |
| GenAI semantic conventions | ✅ Done | Manual span attributes (gen_ai.*) |
| Span naming format | ✅ Done | `invoke_agent {name}`, `chat {model}` |
| Token usage tracking | ✅ Done | Extract from model responses |
| **Session ID in baggage** | ❌ **MISSING** | Set `session.id` in OTEL baggage context |
| **gen_ai.conversation.id** | ❌ **MISSING** | Add to all spans for conversation grouping |
| **gen_ai.agent.id** | ❌ **MISSING** | Unique identifier per agent instance |
| Resource attributes | ⚠️ Partial | `service.name` set, missing AWS-specific |

### Critical Finding: Session ID in Baggage

The Gen AI Observability dashboard groups spans by **session** using the `session.id` baggage value, NOT just the `gen_ai.conversation.id` attribute. The Python SDK uses:

```python
from opentelemetry import baggage
from opentelemetry.context import attach

# This is how sessions are correlated
ctx = baggage.set_baggage("session.id", session_id)
attach(ctx)
```

**In Rust, we must set baggage before creating spans:**

```rust
use opentelemetry::baggage::BaggageExt;
use opentelemetry::Context;

let cx = Context::current()
    .with_baggage(vec![KeyValue::new("session.id", session_id)]);
// All spans created within this context will inherit the baggage
```

### AWS-Specific Resource Attributes

For non-AgentCore hosted agents (like Stood), these resource attributes improve dashboard integration:

```
service.name              - Agent/service identifier (we set this)
aws.log.group.names       - CloudWatch log group for correlation
cloud.resource_id         - ARN of the resource (optional)
```

Environment variables used by ADOT that Stood should respect:
```bash
OTEL_SERVICE_NAME=my-agent
OTEL_RESOURCE_ATTRIBUTES=service.name=my-agent,aws.log.group.names=/aws/spans
```

### Header Requirements

For trace correlation across services:

| Header | Purpose | Format |
|--------|---------|--------|
| `X-Amzn-Trace-Id` | X-Ray trace correlation | `Root=1-xxx;Parent=xxx;Sampled=1` |
| `traceparent` | W3C standard correlation | `00-traceid-parentid-01` |
| `baggage` | Context propagation | `session.id=xxx,userId=xxx` |

### Summary: What's Missing for Gen AI Dashboard

1. **Session ID propagation via baggage** - Required for the dashboard to group spans into sessions
2. **gen_ai.conversation.id on spans** - Required for conversation/thread tracking within sessions
3. **gen_ai.agent.id** - Unique identifier for the agent instance

Without these, spans ARE sent to CloudWatch (visible in `aws/spans` log group) but do NOT appear in the Gen AI Observability dashboard's Sessions/Traces views.

**Sources:**
- [Add observability to your Amazon Bedrock AgentCore resources](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/observability-configure.html)
- [Get started with AgentCore Observability](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/observability-get-started.html)
- [Semantic Conventions for GenAI agent spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/)

---

## December 2025 Research: AgentCore Traces Dashboard Integration

This section documents research conducted to resolve issues with spans not appearing in the AgentCore Traces dashboard despite being successfully exported to CloudWatch.

### Problem Statement

Spans were appearing in:
- ✅ CloudWatch `aws/spans` log group (Transaction Search)
- ✅ Model Invocations dashboard
- ❌ AgentCore Observability > Traces dashboard
- ❌ AgentCore Observability > Sessions view

### Root Cause Analysis

After analyzing the [Strands SDK tracer.py](https://github.com/strands-agents/sdk-python/blob/main/src/strands/telemetry/tracer.py) and [OpenTelemetry AWS Bedrock spec](https://opentelemetry.io/docs/specs/semconv/gen-ai/aws-bedrock/), the issue was identified:

**Our `invoke_agent` spans were missing the REQUIRED `gen_ai.provider.name` attribute.**

| Span Type | Attribute | Before Fix | After Fix |
|-----------|-----------|------------|-----------|
| `chat` | `gen_ai.provider.name` | ✅ "aws.bedrock" | ✅ "aws.bedrock" |
| `invoke_agent` | `gen_ai.provider.name` | ❌ **MISSING** | ✅ "aws.bedrock" |
| `execute_tool` | `gen_ai.provider.name` | ❌ **MISSING** | ✅ "aws.bedrock" |

### OpenTelemetry AWS Bedrock Spec Requirements

From the official [OpenTelemetry Semantic Conventions for AWS Bedrock](https://opentelemetry.io/docs/specs/semconv/gen-ai/aws-bedrock/):

> **`gen_ai.provider.name` MUST be set to `"aws.bedrock"`**

This is a **REQUIRED** attribute for all AWS Bedrock operations including:
- `chat` (model invocations)
- `invoke_agent` (agent operations)
- `execute_tool` (tool executions)

### SpanKind Clarification

Per [OpenTelemetry GenAI Agent Spans spec](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/):

| Agent Type | SpanKind | Examples |
|------------|----------|----------|
| Remote agent services | CLIENT | AWS Bedrock Agents, OpenAI Assistants |
| In-process agents | INTERNAL | LangChain, CrewAI, **Strands**, **Stood** |

**Stood is an in-process agent framework** → `SpanKind::Internal` is correct ✅

### Log Group Clarification

Two log group patterns exist for different purposes:

| Log Group Pattern | Purpose | Data Type |
|-------------------|---------|-----------|
| `aws/spans` | OTLP spans for Transaction Search | Trace spans |
| `/aws/bedrock-agentcore/runtimes/<agent-id>` | Agent runtime stdout/stderr | Application logs |

**Stood uses `aws/spans`** - This is correct for sending OTLP spans.

The `/aws/bedrock-agentcore/runtimes/` log groups are only created when running agents inside the AgentCore Runtime (serverless deployment).

### How Strands SDK Implements Observability

The Strands SDK sets either `gen_ai.system` OR `gen_ai.provider.name` based on convention version:

```python
def _get_common_attributes(self, operation_name: str) -> Dict[str, AttributeValue]:
    common_attributes = {"gen_ai.operation.name": operation_name}
    if self.use_latest_genai_conventions:
        common_attributes.update({"gen_ai.provider.name": "strands-agents"})
    else:
        common_attributes.update({"gen_ai.system": "strands-agents"})
    return dict(common_attributes)
```

However, per the AWS Bedrock spec, we use `gen_ai.provider.name = "aws.bedrock"` to identify Bedrock as the provider (not the framework name).

### Session ID Propagation (ADOT Pattern)

For non-AgentCore hosted agents, session correlation is done via OpenTelemetry baggage:

```python
# Python ADOT pattern
from opentelemetry import baggage
from opentelemetry.context import attach

ctx = baggage.set_baggage("session.id", session_id)
attach(ctx)
```

In Rust:
```rust
use opentelemetry::baggage::BaggageExt;
use opentelemetry::Context;

let cx = Context::current()
    .with_baggage(vec![KeyValue::new("session.id", session_id)]);
```

### Fix Applied (Phase 1 - Span Attributes)

In `src/telemetry/tracer.rs`:

1. Added `gen_ai.provider.name = "aws.bedrock"` to:
   - `start_invoke_agent_span()`
   - `start_execute_tool_span()`

2. Removed `gen_ai.system = "stood"` from `create_span_with_parent()`
   - This was a legacy attribute; per-span `gen_ai.provider.name` replaces it

### Fix Applied (Phase 2 - Resource Attributes)

**Critical Finding:** The Gen AI Observability Dashboard requires the `aws.log.group.names` RESOURCE attribute to identify and display agents. This is different from span attributes - it's set once on the TracerProvider and applies to all spans.

In `src/telemetry/exporter.rs`:

Added `aws.log.group.names` to the OTLP Resource:

```rust
KeyValue {
    key: "aws.log.group.names".to_string(),
    value: AnyValue {
        string_value: Some(format!(
            "/aws/bedrock-agentcore/runtimes/{}",
            agent_id  // Uses agent_id, not service_name
        )),
        ...
    },
},
```

**⚠️ CRITICAL CORRECTION (December 2025):**

**The log group MUST physically exist in CloudWatch.** Setting `aws.log.group.names` as a resource attribute is NOT sufficient for non-AgentCore agents.

Per [AWS AgentCore Observability docs](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/observability-configure.html):

> "For agents running outside of the AgentCore runtime, you also need to create an agent log-group which you include in your environment variables."

**Required Steps for Non-AgentCore Agents:**
1. Create log group: `/aws/bedrock-agentcore/runtimes/{agent_id}`
2. Create log stream: `runtime-logs`
3. Set `aws.log.group.names` resource attribute to match
4. IAM policy must include `logs:CreateLogGroup`, `logs:CreateLogStream`

### Required Resource Attributes for Gen AI Dashboard

| Resource Attribute | Value | Required |
|-------------------|-------|----------|
| `service.name` | Your agent name (e.g., "stood-agent") | Yes |
| `aws.log.group.names` | `/aws/bedrock-agentcore/runtimes/<agent-name>` | **Yes - CRITICAL** |
| `service.version` | Version string | Recommended |

### References

- [OpenTelemetry AWS Bedrock Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/aws-bedrock/)
- [OpenTelemetry GenAI Agent Spans](https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/)
- [Strands SDK Tracer Implementation](https://github.com/strands-agents/sdk-python/blob/main/src/strands/telemetry/tracer.py)
- [AWS AgentCore Observability Documentation](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/observability-configure.html)
- [AWS AgentCore Observability Quickstart](https://aws.github.io/bedrock-agentcore-starter-toolkit/user-guide/observability/quickstart.html)

---

## December 2025 Research: Log Group Requirement for Non-AgentCore Agents

### Critical Discovery

For agents running outside the AgentCore runtime (like Stood), the CloudWatch Gen AI Observability Dashboard has a **hard requirement** that is not well documented:

**The log group `/aws/bedrock-agentcore/runtimes/{agent_id}` and log stream `runtime-logs` MUST physically exist in CloudWatch before spans will appear in the dashboard.**

### Evidence

From [AWS AgentCore Observability Configuration](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/observability-configure.html):

> "For agents running outside of the AgentCore runtime, you also need to create an agent log-group which you include in your environment variables."

Required environment variables for non-AgentCore agents:
```bash
OTEL_RESOURCE_ATTRIBUTES=service.name=<agent-name>,aws.log.group.names=/aws/bedrock-agentcore/runtimes/<agent-id>
OTEL_EXPORTER_OTLP_LOGS_HEADERS=x-aws-log-group=/aws/bedrock-agentcore/runtimes/<agent-id>,x-aws-log-stream=runtime-logs
```

### What We Had (Insufficient)

- ✅ Spans exporting to `/aws/spans` via X-Ray OTLP endpoint
- ✅ `aws.log.group.names` resource attribute set
- ✅ `gen_ai.provider.name` on all spans
- ✅ Correct GenAI semantic conventions
- ❌ Log group does NOT exist
- ❌ Spans do NOT appear in Gen AI Dashboard

### What Is Required

1. **Create Log Group:**
   ```bash
   aws logs create-log-group --log-group-name /aws/bedrock-agentcore/runtimes/{agent_id}
   ```

2. **Create Log Stream:**
   ```bash
   aws logs create-log-stream \
     --log-group-name /aws/bedrock-agentcore/runtimes/{agent_id} \
     --log-stream-name runtime-logs
   ```

3. **IAM Permissions:**
   ```json
   {
     "Effect": "Allow",
     "Action": [
       "logs:CreateLogGroup",
       "logs:CreateLogStream",
       "logs:DescribeLogGroups",
       "logs:DescribeLogStreams"
     ],
     "Resource": "arn:aws:logs:*:*:log-group:/aws/bedrock-agentcore/*"
   }
   ```

4. **Initialization Order:**
   Log group/stream creation MUST happen BEFORE telemetry export initialization.

### Implementation for Stood

The Stood library needs to:
1. Accept an `agent_id` in `TelemetryConfig`
2. Create log group/stream on agent startup (before tracer init)
3. Use the same `agent_id` for `aws.log.group.names` resource attribute
4. Fail gracefully if log group creation fails (agent should still work)

See **CLOUDWATCH_GENAI_IMPLEMENTATION_GUIDE.md Milestone 7** for implementation details.

### References

- [AWS AgentCore Observability Configuration](https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/observability-configure.html)
- [AWS CloudWatch GenAI Observability Getting Started](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/AgentCore-GettingStarted.html)
- [GitHub: sample-amazon-cloudwatch-generative-ai-observability](https://github.com/aws-samples/sample-amazon-cloudwatch-generative-ai-observability)

---

## December 2025: AgentCore Evaluations Integration

### Discovery: Evaluations Require LangChain Format

Through extensive testing with the AgentCore Evaluations API, we discovered that evaluations (Faithfulness, Correctness, etc.) require a **specific telemetry format** that matches LangChain/LangGraph instrumentation.

### Key Findings

1. **Evaluations query OTEL log events**, not just spans
2. **The instrumentation scope must be `opentelemetry.instrumentation.langchain`**
3. **Log events must be linked to spans** via matching `traceId` and `spanId`
4. **Tool information must be included** in the output for Faithfulness evaluation

### Required Log Event Format

```json
{
  "spanId": "4c0129d0910c46c9",
  "traceId": "62c22481c946bc5cbafb7a0614ad45c8",
  "scope": {
    "name": "opentelemetry.instrumentation.langchain"
  },
  "attributes": {
    "event.name": "opentelemetry.instrumentation.langchain",
    "session.id": "8d000050-6eaf-4c27-b2a8-0a17036cee67"
  },
  "body": {
    "input": {
      "messages": [{"role": "user", "content": "..."}]
    },
    "output": {
      "messages": [{"role": "assistant", "content": "..."}]
    }
  }
}
```

### Tool Information for Faithfulness

The Faithfulness evaluator checks if the agent's response is grounded in tool outputs. For this to work, tool results must be included in the log event output:

```json
{
  "output": {
    "messages": [
      {"role": "user", "content": "Tool: scan_star_system\nInput: {\"system_name\":\"Tau Ceti\"}\nOutput: Scan complete for Tau Ceti..."},
      {"role": "assistant", "content": "Based on the scan results..."}
    ]
  }
}
```

### Implementation in Stood

Files involved:
- `src/telemetry/log_event.rs` - Formats log events in LangChain format
- `src/telemetry/tracer.rs` - Exports log events linked to spans
- `src/agent/event_loop.rs` - Captures tool inputs/outputs in ToolResult

Key code pattern:
```rust
// ToolResult now includes input for complete telemetry
struct ToolResult {
    tool_use_id: String,
    tool_name: String,
    input: Value,      // NEW - captures tool input
    success: bool,
    output: Option<Value>,
    error: Option<String>,
    duration: Duration,
}
```

### Running Evaluations

```bash
# Run the example agent to generate telemetry
cargo run --example 026_nebula_evaluation_test

# Wait 2-3 minutes for data to propagate, then run evaluations
./scripts/evaluate_agent.py --defaults
```

The evaluation script:
1. Queries CloudWatch for log events in the `aws/spans` log group
2. Filters for events with `opentelemetry.instrumentation.langchain` scope
3. Extracts input/output messages
4. Calls the AgentCore Evaluations API (Faithfulness, Completeness, Helpfulness, Correctness)

### Evaluation vs Dashboard: Different Requirements

| Feature | Log Events | Spans | Log Group Required |
|---------|------------|-------|-------------------|
| AgentCore Evaluations | ✅ Required | ✅ For linking | ❌ Uses `aws/spans` |
| Gen AI Dashboard | ❌ Optional | ✅ Required | ✅ Must exist |

This explains why evaluations work without Milestone 7 (log group creation), but the Gen AI Dashboard does not.

---

## Conclusion

CloudWatch Gen AI Observability provides a powerful, standardized way to monitor AI agents. Stood's telemetry implementation status:

### Completed ✅

1. ✅ AWS SigV4 authentication for OTLP exports
2. ✅ Span naming aligned with OTEL GenAI semantic conventions
3. ✅ CloudWatch-specific auto-configuration
4. ✅ Session-level correlation (session.id as span attribute)
5. ✅ `gen_ai.provider.name` on all span types
6. ✅ `aws.log.group.names` resource attribute
7. ✅ **OTEL Log Events export** for AgentCore Evaluations
8. ✅ **LangChain telemetry format** (`opentelemetry.instrumentation.langchain` scope)
9. ✅ **Tool input/output capture** for Faithfulness evaluation
10. ✅ **Span-to-log event linking** via traceId/spanId

### Two Separate Features

Stood now supports TWO distinct CloudWatch GenAI features:

| Feature | Status | Requirement |
|---------|--------|-------------|
| **AgentCore Evaluations** | ✅ Working | LangChain format log events, linked to spans |
| **Gen AI Dashboard** | ⚠️ Partial | Requires physical log group creation (Milestone 7) |

### AgentCore Evaluations (Fully Working)

Evaluations work by querying OTEL log events in the `aws/spans` log group. Stood exports:
- Log events with `opentelemetry.instrumentation.langchain` scope
- Input/output messages in LangChain format
- Tool names, inputs, and outputs for context retrieval
- Session IDs linking all events in a conversation

Run evaluations with:
```bash
./scripts/evaluate_agent.py --defaults
```

### Gen AI Dashboard (Milestone 7 Still Required)

For spans to appear in the Gen AI Dashboard Sessions/Traces views:
- The log group `/aws/bedrock-agentcore/runtimes/{agent_id}` MUST physically exist
- This is only required for the dashboard, NOT for evaluations

The implementation maintains backwards compatibility with traditional OTEL backends while enabling first-class CloudWatch Gen AI Observability support.
