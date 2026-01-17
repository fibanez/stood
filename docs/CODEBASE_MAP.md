# Stood Library - Codebase Map

A comprehensive documentation of all modules in the Stood AI agent library codebase.

---

## Table of Contents

### Core Modules
1. [Library Entry Point (src/lib.rs)](#1-library-entry-point-srclibrs)
2. [Agent Module (src/agent/)](#2-agent-module-srcagent)
3. [LLM Module (src/llm/)](#3-llm-module-srcllm)
4. [Tools Module (src/tools/)](#4-tools-module-srctools)
5. [MCP Module (src/mcp/)](#5-mcp-module-srcmcp)
6. [Telemetry Module (src/telemetry/)](#6-telemetry-module-srctelemetry)
7. [Streaming Module (src/streaming/)](#7-streaming-module-srcstreaming)
8. [Context Manager Module (src/context_manager/)](#8-context-manager-module-srccontext_manager)
9. [Conversation Manager Module (src/conversation_manager/)](#9-conversation-manager-module-srcconversation_manager)
10. [Performance Module (src/performance/)](#10-performance-module-srcperformance)
11. [Types Module (src/types/)](#11-types-module-srctypes)
12. [Error Module (src/error.rs)](#12-error-module-srcerrrors)
13. [Utils Module (src/utils/)](#13-utils-module-srcutils)
14. [Stood Macros Crate (stood-macros/)](#14-stood-macros-crate-stood-macros)
15. [Tests Module (src/tests/)](#15-tests-module-srctests)

### Extensibility & Flow Control
16. [Middleware System (src/tools/middleware.rs)](#16-middleware-system-srctoolsmiddlewarers)
17. [Callback System (src/agent/callbacks/)](#17-callback-system-srcagentcallbacks)
18. [Evaluation Strategies (src/agent/evaluation.rs)](#18-evaluation-strategies-srcagentevaluationrs)
19. [Execution Configuration (src/agent/config.rs)](#19-execution-configuration-srcagentconfigrs)

---

## 1. Library Entry Point (src/lib.rs)

### Purpose
The main entry point for the Stood library. Declares and re-exports all public modules and types, providing a clean API surface for library consumers.

### Implementation Details
- **Rust Constructs**: Module declarations (`mod`), public re-exports (`pub use`), conditional compilation (`#[cfg]`)
- **Special Crates**: None directly (imports from submodules)
- **Technical Implementation**: Hierarchical module organization with selective re-exports to flatten the API

### Key Exports
- `Agent`, `AgentBuilder`, `AgentConfig` - Core agent types
- `StoodError`, `StoodResult` - Error handling
- `Tool`, `ToolRegistry`, `ToolExecutor` - Tool system
- `TelemetryConfig`, `StoodTracer` - Observability
- `Bedrock`, `BedrockProvider` - AWS Bedrock integration
- Macros: `perf_checkpoint!`, `perf_timed!`

### Integration Points
- **Used By**: All external consumers of the library
- **Uses**: All submodules

---

## 2. Agent Module (src/agent/)

### Purpose
The core orchestration component that manages the agentic execution loop. Provides a builder pattern for creating agents, handles conversation flow, tool coordination, and integrates with LLM providers for intelligent task execution.

### Files
- `mod.rs` - Main agent implementation, `Agent` struct, `AgentBuilder`
- `event_loop.rs` - Core agentic loop with 5-phase execution
- `config.rs` - Agent configuration types
- `callbacks.rs` - Callback system for agent events
- `context.rs` - Agent execution context
- `evaluation.rs` - Task completion evaluation strategies

### Implementation Details
- **Rust Constructs**:
  - Builder pattern (`AgentBuilder`)
  - Async/await for LLM calls
  - `Arc<dyn LlmProvider>` for provider abstraction
  - Trait objects for polymorphism
- **Special Crates**:
  - `async_trait` - Async trait support
  - `tokio` - Async runtime
  - `tokio_util::sync::CancellationToken` - Graceful cancellation
  - `tracing` - Structured logging
  - `uuid` - Unique identifiers
  - `chrono` - Timestamps
- **Technical Implementation**:
  - 5-phase event loop: Reasoning -> Tool Selection -> Tool Execution -> Reflection -> Response Generation
  - Isolated evaluation context to prevent conversation pollution
  - Comprehensive metrics collection for each cycle

### Key Methods

**Agent:**
- `builder() -> AgentBuilder` - Create new agent builder
- `invoke(&mut self, input: &str) -> Result<AgentResponse>` - Execute single turn
- `invoke_stream(&mut self, input: &str, callback: StreamCallback) -> Result<AgentResponse>` - Streaming execution
- `add_tool(&mut self, tool: Box<dyn Tool>)` - Register a tool
- `conversation(&self) -> &ConversationManager` - Access conversation history
- `provider(&self) -> &Arc<dyn LlmProvider>` - Access LLM provider

**AgentBuilder:**
- `model(model: impl LlmModel)` - Set the LLM model
- `system_prompt(prompt: &str)` - Set system instructions
- `tool(tool: Box<dyn Tool>)` - Add a tool
- `with_telemetry(config: TelemetryConfig)` - Enable telemetry
- `build() -> Result<Agent>` - Build the agent

**EventLoop:**
- `new(agent, tool_registry, config) -> Result<Self>` - Create event loop
- `run(input: &str) -> Result<EventLoopResult>` - Execute the loop
- `run_with_streaming(input: &str, callback: StreamCallback) -> Result<EventLoopResult>`

### Integration Points
- **Uses**: `llm::LlmProvider`, `tools::ToolRegistry`, `conversation_manager::ConversationManager`, `context_manager::ContextManager`, `telemetry::StoodTracer`, `streaming::StreamConfig`
- **Used By**: Application code, examples

---

## 3. LLM Module (src/llm/)

### Purpose
Provides a unified abstraction layer for interacting with various LLM providers. Defines core traits and implements provider-specific logic for AWS Bedrock (Claude, Nova, Mistral, Llama models).

### Files
- `mod.rs` - Module exports and trait re-exports
- `traits.rs` - Core `LlmProvider` and `LlmModel` traits
- `providers/bedrock.rs` - AWS Bedrock implementation
- `providers/mod.rs` - Provider exports
- `models/bedrock.rs` - Bedrock model definitions (Claude, Nova, etc.)
- `models/mod.rs` - Model exports

### Implementation Details
- **Rust Constructs**:
  - `#[async_trait]` for async trait methods
  - `Box<dyn Stream<Item = StreamEvent>>` for streaming
  - `std::any::Any` for runtime type checking
- **Special Crates**:
  - `aws_sdk_bedrockruntime` - AWS Bedrock Runtime SDK
  - `aws_config` - AWS SDK configuration
  - `async_trait` - Async trait support
  - `futures::Stream` - Streaming response handling
  - `serde_json` - JSON serialization
- **Technical Implementation**:
  - Provider-first architecture: ALL formatting logic lives in the provider
  - Model types are pure metadata (no logic)
  - Unified streaming interface across providers
  - Model-specific request builders (Claude, Nova, Mistral)

### Key Traits

**LlmProvider:**
- `chat(model_id, messages, config) -> Result<ChatResponse>` - Basic chat
- `chat_with_tools(model_id, messages, tools, config) -> Result<ChatResponse>` - Chat with tool use
- `chat_streaming(model_id, messages, config) -> Result<Box<dyn Stream>>` - Streaming chat
- `chat_streaming_with_tools(...)` - Streaming with tools
- `health_check() -> Result<HealthStatus>` - Provider health
- `capabilities() -> ProviderCapabilities` - Feature support
- `supported_models() -> Vec<&'static str>` - Available models

**LlmModel:**
- `model_id() -> &'static str` - Unique identifier
- `provider() -> ProviderType` - Provider type
- `context_window() -> usize` - Max context tokens
- `max_output_tokens() -> usize` - Max output tokens
- `capabilities() -> ModelCapabilities` - Feature flags

### Key Types
- `ChatConfig` - Request configuration
- `ChatResponse` - Response with content and usage
- `StreamEvent` - Streaming events (text, tool calls, etc.)
- `ProviderType` - Enum: Bedrock, Anthropic, OpenAI, etc.
- `ProviderCapabilities` - Feature flags per provider

### Integration Points
- **Uses**: `types::Messages`, `types::ContentBlock`
- **Used By**: `agent::Agent`, `agent::EventLoop`

---

## 4. Tools Module (src/tools/)

### Purpose
Comprehensive tool system for agent function calling. Provides trait definitions, registry for tool management, parallel execution with concurrency control, validation, and metrics collection.

### Files
- `mod.rs` - Core `Tool` trait, `ToolResult`, `ToolError`, `ToolRegistry`
- `executor.rs` - Parallel tool execution with timeouts
- `middleware.rs` - Tool execution middleware chain

### Implementation Details
- **Rust Constructs**:
  - `#[async_trait]` for async tool execution
  - `Arc<dyn Tool>` for thread-safe tool references
  - `Semaphore` for concurrency limiting
  - `tokio::select!` for timeout handling
- **Special Crates**:
  - `async_trait` - Async trait methods
  - `tokio` - Async execution, semaphores
  - `serde_json` - Parameter schemas
  - `jsonschema` - Input validation (optional)
- **Technical Implementation**:
  - Registry pattern for tool discovery
  - Parallel execution with configurable concurrency
  - Two execution strategies: Legacy (semaphore) and Parallel
  - Comprehensive metrics per tool execution

### Key Traits & Types

**Tool Trait:**
- `name() -> &str` - Tool identifier
- `description() -> &str` - Human-readable description
- `parameters_schema() -> Value` - JSON Schema for inputs
- `execute(params, agent_context) -> Result<ToolResult, ToolError>` - Execute tool

**ToolRegistry:**
- `register(tool: Box<dyn Tool>)` - Add a tool
- `get(name: &str) -> Option<Arc<dyn Tool>>` - Retrieve tool
- `list() -> Vec<&str>` - List all tool names
- `tool_specs() -> Vec<ToolSpec>` - Get all specs for LLM

**ToolExecutor:**
- `execute_tool(tool, tool_use) -> (ToolResult, Option<Metrics>)` - Single execution
- `execute_tools_parallel(executions) -> Vec<(ToolResult, Option<Metrics>)>` - Batch execution
- `set_max_concurrent(limit)` - Runtime configuration

**ToolResult:**
- `success(content: Value) -> Self` - Success result
- `error(message: String) -> Self` - Error result

**ToolError (enum):**
- `InvalidParameters { message }` - Bad input
- `ToolNotFound { name }` - Unknown tool
- `ExecutionFailed { message }` - Runtime failure
- `DuplicateTool { name }` - Registration conflict

### Integration Points
- **Uses**: `types::ToolSpec`, `agent::AgentContext`
- **Used By**: `agent::EventLoop`, `stood_macros::tool`

---

## 5. MCP Module (src/mcp/)

### Purpose
Model Context Protocol (MCP) support for integrating external tool servers. Enables agents to discover and use tools from MCP-compliant servers via JSON-RPC over stdio.

### Files
- `mod.rs` - MCP client, server integration, protocol types

### Implementation Details
- **Rust Constructs**:
  - Async channels for communication
  - JSON-RPC 2.0 protocol handling
  - Process spawning for server management
- **Special Crates**:
  - `tokio::process` - Child process management
  - `serde_json` - JSON-RPC messages
  - `tokio::sync::mpsc` - Message passing
- **Technical Implementation**:
  - Stdio transport for MCP server communication
  - Automatic tool discovery via `tools/list`
  - Dynamic tool registration from external servers
  - Protocol version negotiation

### Key Types
- `McpClient` - Client for communicating with MCP servers
- `McpServerConfig` - Server configuration (command, args, env)
- `McpTool` - Tool wrapper for MCP-provided tools
- `McpError` - MCP-specific errors

### Key Methods
- `connect(config) -> Result<McpClient>` - Connect to server
- `list_tools() -> Result<Vec<ToolSpec>>` - Discover tools
- `call_tool(name, params) -> Result<Value>` - Execute tool
- `disconnect()` - Clean shutdown

### Integration Points
- **Uses**: `tools::Tool`, `tools::ToolSpec`
- **Used By**: `agent::Agent` (via tool registration)

---

## 6. Telemetry Module (src/telemetry/)

### Purpose
Production-grade observability integration with AWS CloudWatch Gen AI Observability. Implements OpenTelemetry GenAI semantic conventions for tracing agent executions, model calls, and tool usage.

### Files
- `mod.rs` - Public API, `TelemetryConfig`, metrics types
- `tracer.rs` - `StoodTracer` and `StoodSpan` implementations
- `exporter.rs` - `CloudWatchExporter`, `SpanExporter` trait
- `genai.rs` - GenAI semantic convention attributes
- `session.rs` - Session management for conversation tracking
- `log_event.rs` - Log event types for evaluations
- `log_group.rs` - CloudWatch log group management
- `aws_auth.rs` - AWS authentication helpers
- `logging.rs` - Structured logging configuration

### Implementation Details
- **Rust Constructs**:
  - `Arc<Mutex<>>` for thread-safe span collection
  - `AtomicU64` for span ordering
  - `async` batch export
- **Special Crates**:
  - `opentelemetry` - OTEL primitives (Context, KeyValue)
  - `aws_sdk_cloudwatchlogs` - CloudWatch Logs SDK
  - `tracing` / `tracing_subscriber` - Structured logging
  - `chrono` - Timestamps
- **Technical Implementation**:
  - Custom OTLP-compatible span exporter for CloudWatch
  - GenAI semantic conventions (gen_ai.* attributes)
  - Session-based grouping for dashboard visualization
  - Batched async export to minimize overhead

### Key Types

**TelemetryConfig:**
- `cloudwatch(region: &str) -> Self` - CloudWatch enabled
- `disabled() -> Self` - No telemetry
- `from_env() -> Self` - Environment-based config
- `with_service_name(name: &str)` - Custom service name

**StoodTracer:**
- `init(config) -> Result<Option<Self>>` - Initialize tracer
- `start_session() -> Session` - Begin session tracking
- `start_agent_span(name, config) -> StoodSpan` - Agent invocation span
- `start_chat_span(model_id) -> StoodSpan` - Model call span
- `start_tool_span(tool_name) -> StoodSpan` - Tool execution span
- `export() -> Result<()>` - Flush pending spans

**StoodSpan:**
- `set_attribute(key, value)` - Add span attribute
- `add_event(name, attributes)` - Add span event
- `record_usage(input_tokens, output_tokens)` - Token usage
- `end()` - Close span

### Semantic Conventions (GenAI)
| Span Name | Key Attributes |
|-----------|----------------|
| `invoke_agent {name}` | `gen_ai.agent.name`, `gen_ai.usage.*` |
| `chat {model}` | `gen_ai.request.model`, `gen_ai.provider.name` |
| `execute_tool {name}` | `gen_ai.tool.name`, `gen_ai.tool.type` |

### Integration Points
- **Uses**: `aws_sdk_cloudwatchlogs`
- **Used By**: `agent::EventLoop`, `agent::Agent`

---

## 7. Streaming Module (src/streaming/)

### Purpose
Real-time streaming support for LLM responses. Provides unified streaming abstractions, callback mechanisms, and event types for progressive response delivery.

### Files
- `mod.rs` - `StreamConfig`, `StreamEvent`, `StreamCallback`, streaming utilities

### Implementation Details
- **Rust Constructs**:
  - `Pin<Box<dyn Stream>>` for async streaming
  - `Arc<dyn Fn(StreamEvent)>` for callbacks
  - Enum-based event types
- **Special Crates**:
  - `futures::Stream` - Streaming abstraction
  - `tokio_stream` - Tokio stream utilities
  - `async_stream` - Stream creation macros
- **Technical Implementation**:
  - Pull-based streaming with backpressure
  - Event-based callback system
  - Accumulated text tracking
  - Tool call state machine for partial updates

### Key Types

**StreamEvent (enum):**
- `TextDelta { text }` - Incremental text
- `ToolCallStart { id, name }` - Tool call begins
- `ToolCallDelta { id, input_delta }` - Tool input chunk
- `ToolCallComplete { id, name, input }` - Tool call ready
- `UsageUpdate { input_tokens, output_tokens }` - Token counts
- `Error { message }` - Stream error
- `Done` - Stream complete

**StreamConfig:**
- `enabled: bool` - Enable streaming
- `buffer_size: usize` - Event buffer
- `emit_usage: bool` - Include usage events

**StreamCallback:**
- Type alias: `Arc<dyn Fn(StreamEvent) + Send + Sync>`

### Integration Points
- **Uses**: `llm::StreamEvent`
- **Used By**: `agent::Agent::invoke_stream()`, `agent::EventLoop`

---

## 8. Context Manager Module (src/context_manager/)

### Purpose
Manages agent execution context including environment, state, and metadata available during tool execution and agent reasoning.

### Files
- `mod.rs` - `ContextManager`, context types

### Implementation Details
- **Rust Constructs**:
  - `HashMap<String, Value>` for key-value storage
  - `Arc<RwLock<>>` for concurrent access
- **Special Crates**:
  - `serde_json` - Value storage
  - `tokio::sync::RwLock` - Async read-write lock
- **Technical Implementation**:
  - Hierarchical context scoping
  - Thread-safe concurrent access
  - Serializable context state

### Key Methods
- `new() -> Self` - Create context manager
- `set(key: &str, value: Value)` - Store value
- `get(key: &str) -> Option<Value>` - Retrieve value
- `remove(key: &str) -> Option<Value>` - Delete value
- `clear()` - Reset context
- `snapshot() -> HashMap<String, Value>` - Copy current state

### Integration Points
- **Uses**: None
- **Used By**: `agent::Agent`, `tools::Tool::execute()`

---

## 9. Conversation Manager Module (src/conversation_manager/)

### Purpose
Manages conversation history and message flow. Handles message storage, system prompt management, context windowing, and conversation state.

### Files
- `mod.rs` - `ConversationManager`, conversation operations

### Implementation Details
- **Rust Constructs**:
  - `Vec<Message>` for message storage
  - Iterator methods for filtering
- **Special Crates**:
  - `uuid` - Message identifiers
  - `chrono` - Timestamps
- **Technical Implementation**:
  - Append-only message log
  - System prompt as first message
  - Token-aware truncation (future)
  - Role-based message access

### Key Methods
- `new() -> Self` - Create empty conversation
- `with_system_prompt(prompt: &str) -> Self` - With system instructions
- `add_user_message(text: &str)` - Add user turn
- `add_assistant_message(text: &str)` - Add assistant turn
- `add_tool_use(id, name, input)` - Record tool call
- `add_tool_result(tool_use_id, content, is_error)` - Record tool result
- `messages() -> &Messages` - Get all messages
- `system_prompt() -> Option<&str>` - Get system prompt
- `clear()` - Reset conversation

### Integration Points
- **Uses**: `types::Messages`, `types::Message`, `types::ContentBlock`
- **Used By**: `agent::Agent`, `agent::EventLoop`

---

## 10. Performance Module (src/performance/)

### Purpose
Production-grade performance optimization for high-throughput agent deployments. Provides connection pooling, request batching, adaptive concurrency control, and memory optimization.

### Files
- `mod.rs` - `PerformanceOptimizer`, `PerformanceConfig`
- `connection_pool.rs` - `BedrockConnectionPool`, `PooledConnection`
- `batch_processor.rs` - `RequestBatchProcessor`, batching logic
- `memory_optimizer.rs` - `MemoryOptimizer`, context pruning
- `metrics.rs` - `PerformanceMetrics`, monitoring data

### Implementation Details
- **Rust Constructs**:
  - `Arc<RwLock<>>` for shared metrics
  - `Semaphore` for concurrency limiting
  - `AtomicUsize` for counters
  - Background `tokio::spawn` tasks
- **Special Crates**:
  - `tokio::sync::{Mutex, RwLock, Semaphore}` - Async synchronization
  - `aws_sdk_bedrockruntime::Client` - Pooled clients
  - `tracing` - Performance logging
- **Technical Implementation**:
  - Connection pool with health checks
  - Batch grouping with configurable timeout
  - Adaptive concurrency based on latency/errors
  - Background optimization tasks

### Key Types

**PerformanceConfig:**
- `max_connections: usize` - Pool size (default: 10)
- `connection_idle_timeout: Duration` - Cleanup interval
- `health_check_interval: Duration` - Health check frequency
- `max_batch_size: usize` - Batch limit (default: 10)
- `batch_timeout: Duration` - Batch collection window
- `memory_threshold: usize` - Memory optimization trigger
- `adaptive_concurrency: bool` - Enable auto-tuning
- `concurrency_factor: f64` - Adjustment factor

**PerformanceOptimizer:**
- `new(config) -> Result<Self>` - Create optimizer
- `get_connection() -> Result<PooledConnection>` - Get pooled client
- `submit_batch_request(request)` - Queue for batching
- `optimize_memory() -> Result<usize>` - Free memory
- `get_metrics() -> PerformanceMetrics` - Current stats
- `adjust_concurrency()` - Manual tuning
- `start_background_tasks() -> Vec<JoinHandle>` - Start optimizers

### Performance Characteristics
- Connection pooling: 60-80% reduction in connection overhead
- Request batching: 30-50% throughput increase
- Memory optimization: 40-60% memory reduction
- Connection acquisition: <1ms (pool hit) vs 100-200ms (new)

### Integration Points
- **Uses**: `aws_sdk_bedrockruntime::Client`
- **Used By**: Advanced deployments (optional)

---

## 11. Types Module (src/types/)

### Purpose
Core type definitions used throughout the library. Provides data structures for messages, content blocks, tool specifications, and agent configuration.

### Files
- `mod.rs` - Re-exports, type aliases (`RequestId`, `AgentId`, `StoodResult`)
- `messages.rs` - `Message`, `Messages`, `MessageRole`
- `content.rs` - `ContentBlock`, `ToolResultContent`, `ReasoningQuality`
- `tools.rs` - `ToolSpec`, `ToolUse`, `ToolChoice`, `StopReason`
- `agent.rs` - `AgentConfig`, `AgentResponse`

### Implementation Details
- **Rust Constructs**:
  - Enum variants with data (`ContentBlock`)
  - Serde derive macros for serialization
  - Builder-style construction
- **Special Crates**:
  - `serde` / `serde_json` - Serialization
  - `uuid` - Identifiers
  - `chrono` - Timestamps
- **Technical Implementation**:
  - Tagged enums for JSON serialization
  - Rich content block types (text, tool_use, tool_result, thinking)
  - Extensible metadata via `HashMap<String, Value>`

### Key Types

**MessageRole (enum):**
- `User` - Human input
- `Assistant` - AI response
- `System` - Instructions

**Message:**
- `id: Uuid` - Unique identifier
- `role: MessageRole` - Sender role
- `content: Vec<ContentBlock>` - Content blocks
- `metadata: HashMap<String, Value>` - Extra data
- `timestamp: DateTime<Utc>` - Creation time

**ContentBlock (enum):**
- `Text { text }` - Plain text
- `ToolUse { id, name, input }` - Tool call request
- `ToolResult { tool_use_id, content, is_error }` - Tool execution result
- `Thinking { content, quality, timestamp }` - Claude thinking
- `ReasoningContent { reasoning }` - Bedrock reasoning

**ToolSpec:**
- `name: String` - Tool identifier
- `description: String` - Human description
- `input_schema: Value` - JSON Schema

**StopReason (enum):**
- `EndTurn` - Normal completion
- `ToolUse` - Wants to call tools
- `MaxTokens` - Limit reached
- `StopSequence` - Stop token found
- `ContentFiltered` - Safety filter

### Integration Points
- **Uses**: Standard library, serde
- **Used By**: All modules (core data types)

---

## 12. Error Module (src/error.rs)

### Purpose
Comprehensive error handling with categorized errors, AWS Bedrock error mapping, automatic retry logic, and detailed error context for debugging and telemetry.

### Implementation Details
- **Rust Constructs**:
  - `#[derive(Error)]` from thiserror
  - `impl From<T>` for error conversion
  - Match-based error classification
- **Special Crates**:
  - `thiserror` - Error derive macro
  - `aws_sdk_bedrockruntime::Error` - AWS error types
  - `fastrand` - Jitter for retries
- **Technical Implementation**:
  - Categorized error variants
  - Automatic retry delay calculation
  - Exponential backoff with jitter
  - AWS error context extraction

### Key Types

**StoodError (enum):**
| Variant              | Description            | Retryable |
|----------------------|------------------------|-----------|
| `InvalidInput`       | Bad user input         | No        |
| `ConfigurationError` | Setup issues           | No        |
| `ModelError`         | LLM failures           | No        |
| `ToolError`          | Tool execution failure | No        |
| `ConversationError`  | State issues           | No        |
| `AccessDenied`       | AWS auth failure       | No        |
| `ServiceUnavailable` | AWS unavailable        | Yes       |
| `ValidationError`    | AWS validation         | No        |
| `ThrottlingError`    | Rate limited           | Yes       |
| `ResourceNotFound`   | Missing resource       | No        |
| `QuotaExceeded`      | Limit exceeded         | Yes       |
| `NetworkError`       | Connection issues      | Yes       |
| `SerializationError` | JSON failures          | No        |
| `TimeoutError`       | Operation timeout      | Yes       |
| `InternalError`      | Library bugs           | No        |

**StoodError Methods:**
- `is_retryable() -> bool` - Check retry eligibility
- `is_auth_error() -> bool` - Auth classification
- `is_user_error() -> bool` - Input classification
- `retry_delay_ms() -> Option<u64>` - Recommended delay
- `max_retries() -> u32` - Retry limit
- `should_use_exponential_backoff() -> bool`

**BedrockErrorContext:**
- `from_bedrock_error(error) -> Self` - Extract context
- `to_detailed_string() -> String` - Debug info
- `is_model_availability_error() -> bool`
- `to_stood_error_with_model(model_id) -> StoodError`

**RetryConfig:**
- `max_retries: u32` - Attempt limit
- `base_delay_ms: u64` - Initial delay
- `exponential_backoff: bool` - Backoff strategy
- `max_delay_ms: u64` - Delay cap
- `jitter_factor: f64` - Randomization

**retry_with_backoff(operation, config) -> Result<T>:**
- Automatic retry with error-based configuration
- Exponential backoff with jitter
- Respects retry eligibility

### Integration Points
- **Uses**: `aws_sdk_bedrockruntime::Error`, `tools::ToolError`
- **Used By**: All modules for error handling

---

## 13. Utils Module (src/utils/)

### Purpose
Utility functions and helpers shared across the library.

### Files
- `mod.rs` - Module exports
- `logging.rs` - Logging utilities, string truncation

### Key Functions
- `truncate_string(s: &str, max_len: usize) -> String` - Safe UTF-8 truncation

### Integration Points
- **Used By**: `error.rs`, logging throughout

---

## 14. Stood Macros Crate (stood-macros/)

### Purpose
Procedural macros for automatic tool generation from Rust functions. The `#[tool]` macro transforms annotated async functions into full `Tool` trait implementations.

### Files
- `src/lib.rs` - Complete macro implementation

### Implementation Details
- **Rust Constructs**:
  - `proc_macro_attribute` - Attribute macro
  - `syn` - Rust syntax parsing
  - `quote` - Code generation
  - `TokenStream` manipulation
- **Special Crates**:
  - `proc-macro2` - Token stream manipulation
  - `syn` - Parse Rust code
  - `quote` - Generate Rust code
- **Technical Implementation**:
  - Parses function signature and doc comments
  - Generates JSON Schema from parameter types
  - Creates tool struct and trait implementation
  - Handles optional parameters (`Option<T>`)
  - Extracts parameter descriptions from doc comments

### Usage Example
```rust
use stood_macros::tool;

#[tool]
/// Calculate the sum of two numbers
async fn add(
    /// First number
    a: f64,
    /// Second number
    b: f64
) -> Result<f64, String> {
    Ok(a + b)
}

// Generates:
// - `add()` function returning `Box<dyn Tool>`
// - `AddTool` struct implementing `Tool` trait
// - JSON Schema from parameter types and docs
```

### Generated Artifacts
1. Renamed implementation function (`{name}_impl`)
2. Tool struct (`{PascalName}Tool`)
3. `Tool` trait implementation
4. Constructor function (`{name}() -> Box<dyn Tool>`)

### Type Mapping (Rust -> JSON Schema)
| Rust Type | JSON Schema |
|-----------|-------------|
| `String`, `&str` | `"string"` |
| `i8..i128`, `u8..u128` | `"integer"` |
| `f32`, `f64` | `"number"` |
| `bool` | `"boolean"` |
| `Vec<T>` | `"array"` |
| `HashMap`, `BTreeMap` | `"object"` |
| `Option<T>` | inner type (optional) |

### Integration Points
- **Uses**: `stood::tools::Tool`, `stood::tools::ToolError`
- **Used By**: User code defining custom tools

---

## 15. Tests Module (src/tests/)

### Purpose
Integration and unit tests for core library functionality. Tests cover safety constraints, tool execution, and provider integration.

### Files
- `core_safety.rs` - Safety and constraint tests
- Various test files for specific modules

### Test Categories
1. **Unit Tests** - Per-module tests in `#[cfg(test)]` blocks
2. **Integration Tests** - `tests/` directory
3. **Provider Integration** - `tests/provider_integration/` (real API calls)

### Running Tests
```bash
# All tests (includes real API calls)
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

---

## 16. Middleware System (src/tools/middleware.rs)

### Purpose
Intercepts tool execution at pre/post execution points. Enables parameter modification, execution abort/skip, result transformation, and context injection - all without modifying tool implementations.

### Architecture
```
Tool Request → before_tool() → Tool Execution → after_tool() → Result
                    ↓                                  ↓
             Can: Modify params           Can: Modify result
                  Abort/Skip                   Inject context
```

### Implementation Details
- **Rust Constructs**:
  - `#[async_trait]` for async middleware methods
  - `Arc<dyn ToolMiddleware>` for thread-safe middleware references
  - `Vec<Arc<dyn ToolMiddleware>>` for middleware stack
  - Builder pattern for `ToolContext`
- **Special Crates**:
  - `async_trait` - Async trait support
  - `serde_json::Value` - Parameter manipulation
  - `tracing` - Debug logging
- **Technical Implementation**:
  - LIFO ordering: `before_tool` in registration order, `after_tool` in reverse
  - Short-circuit on Abort/Skip
  - Parameters flow through the chain, allowing cumulative modifications

### Key Traits & Types

**ToolMiddleware Trait:**
```rust
#[async_trait]
pub trait ToolMiddleware: Send + Sync + Debug {
    async fn before_tool(&self, tool_name: &str, params: &Value, ctx: &ToolContext)
        -> ToolMiddlewareAction;
    async fn after_tool(&self, tool_name: &str, result: &ToolResult, ctx: &ToolContext)
        -> AfterToolAction;
    fn name(&self) -> &str;
}
```

**ToolMiddlewareAction (enum):**
| Variant | Effect |
|---------|--------|
| `Continue` | Proceed with original/current parameters |
| `ModifyParams(Value)` | Replace parameters, continue to next middleware |
| `Abort { reason, synthetic_result }` | Stop execution, return synthetic result |
| `Skip` | Stop execution, no result added to conversation |

**AfterToolAction (enum):**
| Variant | Effect |
|---------|--------|
| `PassThrough` | Use result as-is |
| `ModifyResult(ToolResult)` | Replace the result |
| `InjectContext(String)` | Add context message after result |

**ToolContext:**
- `agent_id: String` - Agent identifier
- `agent_name: Option<String>` - Agent display name
- `agent_type: String` - Agent classification
- `execution_start: Instant` - Timing reference
- `tool_count_this_turn: usize` - Tools executed this turn
- `message_count: usize` - Conversation length

**MiddlewareStack:**
- `add(middleware: Arc<dyn ToolMiddleware>)` - Register middleware
- `process_before_tool(...) -> (Action, Value)` - Run pre-execution chain
- `process_after_tool(...) -> (Action, ToolResult)` - Run post-execution chain

### Use Cases
1. **Logging** - Log all tool calls and results
2. **Caching** - Return cached results for repeated calls
3. **Authorization** - Block unauthorized tool access
4. **Parameter Validation** - Add extra validation rules
5. **Rate Limiting** - Throttle tool execution
6. **Auditing** - Record tool usage for compliance

### Integration Points
- **Uses**: `tools::ToolResult`, `agent::AgentContext`
- **Used By**: `agent::EventLoop` (tool execution phase)

---

## 17. Callback System (src/agent/callbacks/)

### Purpose
Real-time event notification system for monitoring agent execution. Enables streaming output, progress tracking, performance monitoring, and custom integrations without modifying agent logic.

### Files
- `mod.rs` - Re-exports and module organization
- `traits.rs` - `CallbackHandler`, `SyncCallbackHandler` traits
- `events.rs` - `CallbackEvent`, `ToolEvent`, `TokenUsage` types
- `handlers.rs` - Built-in handlers: `NullCallbackHandler`, `PrintingCallbackHandler`, etc.
- `config.rs` - `CallbackHandlerConfig`, `PrintingConfig`
- `batching.rs` - `BatchingCallbackHandler` for high-frequency optimization
- `error.rs` - `CallbackError` type

### Implementation Details
- **Rust Constructs**:
  - `#[async_trait]` for async callback methods
  - `Arc<dyn CallbackHandler>` for handler sharing
  - Default trait method implementations (opt-in callbacks)
  - Blanket impl for sync-to-async conversion
- **Special Crates**:
  - `async_trait` - Async trait methods
  - `tokio::sync::{Mutex, Notify}` - Batching synchronization
  - `tracing` - Internal logging
  - `chrono` - Event timestamps
- **Technical Implementation**:
  - Event-driven architecture with typed events
  - Composite pattern for multiple handlers
  - Background task for batch flushing
  - Automatic sync-to-async wrapper

### Key Traits

**CallbackHandler Trait:**
```rust
#[async_trait]
pub trait CallbackHandler: Send + Sync {
    // Content streaming
    async fn on_content(&self, content: &str, is_complete: bool) -> Result<(), CallbackError>;

    // Tool execution events
    async fn on_tool(&self, event: ToolEvent) -> Result<(), CallbackError>;

    // Completion notification
    async fn on_complete(&self, result: &AgentResult) -> Result<(), CallbackError>;

    // Error handling
    async fn on_error(&self, error: &StoodError) -> Result<(), CallbackError>;

    // Parallel execution tracking
    async fn on_parallel_start(&self, tool_count: usize, max_parallel: usize) -> Result<(), CallbackError>;
    async fn on_parallel_progress(&self, completed: usize, total: usize, running: usize) -> Result<(), CallbackError>;
    async fn on_parallel_complete(&self, duration: Duration, success: usize, failure: usize) -> Result<(), CallbackError>;

    // Evaluation events
    async fn on_evaluation(&self, strategy: &str, decision: bool, reasoning: &str, duration: Duration) -> Result<(), CallbackError>;

    // Full event dispatch
    async fn handle_event(&self, event: CallbackEvent) -> Result<(), CallbackError>;
}
```

### CallbackEvent Types

| Event                             | When Fired            | Key Data                            |
|-----------------------------------|-----------------------|-------------------------------------|
| `EventLoopStart`                  | Loop begins           | `loop_id`, `prompt`, `config`       |
| `CycleStart`                      | Each cycle            | `cycle_id`, `cycle_number`          |
| `ModelStart`                      | LLM call begins       | `provider`, `model_id`, `messages`  |
| `ModelComplete`                   | LLM responds          | `response`, `stop_reason`, `tokens` |
| `ContentDelta`                    | Streaming text        | `delta`, `complete`, `reasoning`    |
| `ToolStart`                       | Tool execution begins | `tool_name`, `tool_use_id`, `input` |
| `ToolComplete`                    | Tool finishes         | `output`, `error`, `duration`       |
| `ParallelStart/Progress/Complete` | Parallel execution    | counts, durations                   |
| `EvaluationStart/Complete`        | Loop evaluation       | `strategy`, `decision`, `reasoning` |
| `EventLoopComplete`               | Loop ends             | `result`, `total_duration`          |
| `Error`                           | Error occurs          | `error`, `context`                  |

### Built-in Handlers

| Handler | Purpose |
|---------|---------|
| `NullCallbackHandler` | No-op (default) |
| `PrintingCallbackHandler` | Console output with configurable verbosity |
| `PerformanceCallbackHandler` | Performance logging via tracing |
| `CompositeCallbackHandler` | Combines multiple handlers |
| `BatchingCallbackHandler` | Batches high-frequency events |

### Configuration

**PrintingConfig:**
- `stream_output: bool` - Print streaming content
- `show_tools: bool` - Print tool events
- `show_performance: bool` - Print execution summary
- `show_reasoning: bool` - Print thinking content

**BatchConfig:**
- `max_batch_size: usize` - Events before flush (default: 10)
- `max_batch_delay: Duration` - Time before flush (default: 50ms)
- `batch_content_deltas: bool` - Batch text events
- `batch_tool_events: bool` - Batch tool events

**CallbackHandlerConfig (enum):**
- `None` - No callbacks
- `Printing(PrintingConfig)` - Console output
- `Performance(Level)` - Tracing integration
- `Custom(Arc<dyn CallbackHandler>)` - User-provided handler
- `Composite(Vec<CallbackHandlerConfig>)` - Multiple handlers

### Integration Points
- **Uses**: `agent::result::AgentResult`, `error::StoodError`
- **Used By**: `agent::EventLoop`, `agent::Agent`

---

## 18. Evaluation Strategies (src/agent/evaluation.rs)

### Purpose
Configurable strategies for determining when the agentic loop should continue or terminate. Enables model-driven, task-based, multi-perspective, and agent-based evaluation approaches.

### Implementation Details
- **Rust Constructs**:
  - Enum with variant-specific data
  - `Box<Agent>` for nested agent evaluation
  - Weighted scoring for multi-perspective
- **Technical Implementation**:
  - Isolated evaluation context prevents conversation pollution
  - Each strategy has custom continuation logic
  - Configurable iteration limits

### EvaluationStrategy (enum)

| Strategy | Description | Use Case |
|----------|-------------|----------|
| `None` (default) | Model decides naturally | Simple tasks, model-driven flow |
| `TaskEvaluation` | Same agent evaluates completion | Complex tasks requiring verification |
| `MultiPerspective` | Multiple weighted evaluations | Quality-critical applications |
| `AgentBased` | Separate evaluator agent | External validation, compliance |

### Configuration

**EvaluationStrategy::None:**
- Default behavior - model continues if it requests tool calls
- No additional LLM calls for evaluation
- Most efficient option

**EvaluationStrategy::TaskEvaluation:**
```rust
TaskEvaluation {
    evaluation_prompt: String,  // Custom prompt for evaluation
    max_iterations: u32,        // Safety limit (default: 5)
}
```

**EvaluationStrategy::MultiPerspective:**
```rust
MultiPerspective {
    perspectives: Vec<PerspectiveConfig>,
}

PerspectiveConfig {
    name: String,      // e.g., "quality_check"
    prompt: String,    // Evaluation prompt
    weight: f32,       // 0.0 to 1.0
}
```

**EvaluationStrategy::AgentBased:**
```rust
AgentBased {
    evaluator_agent: Box<Agent>,   // Separate agent instance
    evaluation_prompt: String,      // Prompt for evaluator
}
```

### Key Methods
- `task_evaluation(prompt) -> Self` - Create task evaluation strategy
- `multi_perspective(perspectives) -> Self` - Create multi-perspective
- `agent_based(agent, prompt) -> Self` - Create agent-based
- `name() -> &'static str` - Strategy identifier for logging
- `requires_evaluation() -> bool` - Whether LLM calls needed

### Integration Points
- **Uses**: `agent::Agent` (for AgentBased)
- **Used By**: `agent::EventLoop` (reflection phase)

---

## 19. Execution Configuration (src/agent/config.rs)

### Purpose
Unified configuration for agent execution behavior. Controls callbacks, streaming, timeouts, logging, and event loop settings through a single configuration object.

### Key Types

**ExecutionConfig:**
```rust
pub struct ExecutionConfig {
    pub callback_handler: CallbackHandlerConfig,  // Event handling
    pub event_loop: EventLoopConfig,              // Loop behavior
    pub streaming: bool,                          // Enable streaming
    pub timeout: Option<Duration>,                // Max execution time
    pub log_level: LogLevel,                      // Debug output level
}
```

**EventLoopConfig:**
```rust
pub struct EventLoopConfig {
    pub max_cycles: u32,                          // Max iterations (default: 10)
    pub max_duration: Duration,                   // Total timeout (default: 5min)
    pub enable_streaming: bool,                   // Streaming responses
    pub tool_config: ExecutorConfig,              // Tool execution settings
    pub enable_telemetry: bool,                   // CloudWatch integration
    pub stream_config: StreamConfig,              // Streaming behavior
    pub retry_config: RetryConfig,                // Error recovery
    pub evaluation_strategy: EvaluationStrategy,  // Continuation logic
    pub max_tool_iterations: u32,                 // Tools per cycle (default: 7)
    pub cancellation_token: Option<CancellationToken>, // Early termination
}
```

**LogLevel (enum):**
- `Off` - No debug logging
- `Info` - Basic execution flow
- `Debug` - Step-by-step details
- `Trace` - Full verbose output

### Factory Methods

| Method | Effect |
|--------|--------|
| `ExecutionConfig::default()` | Silent, streaming enabled |
| `ExecutionConfig::with_printing()` | Console output |
| `ExecutionConfig::verbose()` | Detailed console output |
| `ExecutionConfig::silent()` | No callbacks |
| `ExecutionConfig::minimal()` | Minimal console output |
| `ExecutionConfig::with_handler(handler)` | Custom callback handler |
| `ExecutionConfig::with_composite(handlers)` | Multiple handlers |
| `ExecutionConfig::with_timeout(duration)` | Custom timeout |
| `ExecutionConfig::with_performance(level)` | Performance tracing |

### Builder Pattern
```rust
let config = ExecutionConfig::default()
    .timeout(Duration::from_secs(60))
    .streaming(true)
    .log_level(LogLevel::Debug)
    .event_loop_config(custom_event_loop_config);
```

### Integration Points
- **Uses**: `callbacks::CallbackHandlerConfig`, `event_loop::EventLoopConfig`
- **Used By**: `agent::AgentBuilder`, `agent::Agent`

---

## Flow Configuration Summary

### Injection Points

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Agent Execution Flow                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  User Input                                                          │
│      │                                                               │
│      ▼                                                               │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │ ExecutionConfig                                               │   │
│  │ • CallbackHandlerConfig → Which events to capture            │   │
│  │ • EventLoopConfig → How many cycles, timeouts, etc.          │   │
│  │ • LogLevel → Debug output verbosity                          │   │
│  └──────────────────────────────────────────────────────────────┘   │
│      │                                                               │
│      ▼                                                               │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │                     EVENT LOOP                              │     │
│  │  ┌──────────────────────────────────────────────────────┐  │     │
│  │  │ Callback: EventLoopStart                             │  │     │
│  │  └──────────────────────────────────────────────────────┘  │     │
│  │                          │                                  │     │
│  │                          ▼                                  │     │
│  │  ┌──────────────────────────────────────────────────────┐  │     │
│  │  │ CYCLE N                                              │  │     │
│  │  │  ┌────────────────────────────────────────────────┐  │  │     │
│  │  │  │ Callback: CycleStart, ModelStart               │  │  │     │
│  │  │  └────────────────────────────────────────────────┘  │  │     │
│  │  │                       │                              │  │     │
│  │  │                       ▼                              │  │     │
│  │  │  ┌────────────────────────────────────────────────┐  │  │     │
│  │  │  │ LLM Call (streaming callbacks: ContentDelta)   │  │  │     │
│  │  │  └────────────────────────────────────────────────┘  │  │     │
│  │  │                       │                              │  │     │
│  │  │                       ▼                              │  │     │
│  │  │  ┌────────────────────────────────────────────────┐  │  │     │
│  │  │  │ TOOL EXECUTION                                  │  │  │     │
│  │  │  │  ┌──────────────────────────────────────────┐  │  │  │     │
│  │  │  │  │ Middleware: before_tool()                │  │  │  │     │
│  │  │  │  │ • Can modify params                      │  │  │  │     │
│  │  │  │  │ • Can abort/skip                         │  │  │  │     │
│  │  │  │  └──────────────────────────────────────────┘  │  │  │     │
│  │  │  │                     │                          │  │  │     │
│  │  │  │  ┌──────────────────────────────────────────┐  │  │  │     │
│  │  │  │  │ Callback: ToolStart                      │  │  │  │     │
│  │  │  │  └──────────────────────────────────────────┘  │  │  │     │
│  │  │  │                     │                          │  │  │     │
│  │  │  │                     ▼                          │  │  │     │
│  │  │  │            [Tool Execution]                    │  │  │     │
│  │  │  │                     │                          │  │  │     │
│  │  │  │  ┌──────────────────────────────────────────┐  │  │  │     │
│  │  │  │  │ Middleware: after_tool()                 │  │  │  │     │
│  │  │  │  │ • Can modify result                      │  │  │  │     │
│  │  │  │  │ • Can inject context                     │  │  │  │     │
│  │  │  │  └──────────────────────────────────────────┘  │  │  │     │
│  │  │  │                     │                          │  │  │     │
│  │  │  │  ┌──────────────────────────────────────────┐  │  │  │     │
│  │  │  │  │ Callback: ToolComplete                   │  │  │  │     │
│  │  │  │  └──────────────────────────────────────────┘  │  │  │     │
│  │  │  └────────────────────────────────────────────────┘  │  │     │
│  │  │                       │                              │  │     │
│  │  │                       ▼                              │  │     │
│  │  │  ┌────────────────────────────────────────────────┐  │  │     │
│  │  │  │ EVALUATION (EvaluationStrategy)               │  │  │     │
│  │  │  │ • None: Model-driven (default)                │  │  │     │
│  │  │  │ • TaskEvaluation: Same agent evaluates        │  │  │     │
│  │  │  │ • MultiPerspective: Weighted evaluation       │  │  │     │
│  │  │  │ • AgentBased: Separate evaluator agent        │  │  │     │
│  │  │  └────────────────────────────────────────────────┘  │  │     │
│  │  │                       │                              │  │     │
│  │  │  ┌────────────────────────────────────────────────┐  │  │     │
│  │  │  │ Callback: EvaluationComplete                  │  │  │     │
│  │  │  └────────────────────────────────────────────────┘  │  │     │
│  │  └──────────────────────────────────────────────────────┘  │     │
│  │                          │                                  │     │
│  │               Continue? ◄┴─► Stop                          │     │
│  │                          │                                  │     │
│  │  ┌──────────────────────────────────────────────────────┐  │     │
│  │  │ Callback: EventLoopComplete                          │  │     │
│  │  └──────────────────────────────────────────────────────┘  │     │
│  └────────────────────────────────────────────────────────────┘     │
│      │                                                               │
│      ▼                                                               │
│  Agent Response                                                      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Module Dependency Graph

```
                    ┌─────────────┐
                    │   lib.rs    │
                    │  (exports)  │
                    └──────┬──────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
   ┌─────────┐      ┌─────────────┐     ┌─────────┐
   │  agent  │◄────►│    tools    │◄───►│   llm   │
   └────┬────┘      └──────┬──────┘     └────┬────┘
        │                  │                  │
        │                  │                  │
        ▼                  ▼                  ▼
   ┌─────────┐      ┌─────────────┐     ┌─────────┐
   │streaming│      │stood_macros │     │ bedrock │
   └─────────┘      │  (proc-mac) │     │provider │
        │           └─────────────┘     └─────────┘
        │
        ▼
┌───────────────┐    ┌───────────────┐    ┌─────────────┐
│ conversation  │    │   context     │    │  telemetry  │
│   manager     │    │   manager     │    │ (cloudwatch)│
└───────────────┘    └───────────────┘    └─────────────┘
        │                  │                     │
        └──────────────────┼─────────────────────┘
                           ▼
                    ┌─────────────┐
                    │    types    │
                    │   (core)    │
                    └──────┬──────┘
                           │
                           ▼
                    ┌─────────────┐     ┌─────────────┐
                    │    error    │     │performance  │
                    └─────────────┘     │ (optional)  │
                                        └─────────────┘
```

---

## External Dependencies Summary

| Category | Crates |
|----------|--------|
| **AWS SDK** | `aws_sdk_bedrockruntime`, `aws_config`, `aws_sdk_cloudwatchlogs` |
| **Async** | `tokio`, `async_trait`, `futures`, `tokio_stream`, `tokio_util` |
| **Serialization** | `serde`, `serde_json` |
| **Error Handling** | `thiserror` |
| **Observability** | `tracing`, `tracing_subscriber`, `opentelemetry` |
| **Utilities** | `uuid`, `chrono`, `fastrand`, `base64` |
| **Proc Macros** | `proc-macro2`, `syn`, `quote` |

---

*Generated: 2026-01-11*
