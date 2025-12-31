# API Documentation

This section provides detailed API documentation for the core Stood agent library components.

## Agent

### Agent::builder()

Creates a new AgentBuilder for configuring and constructing Agent instances. The builder provides a fluent interface for setting up agents with custom models, parameters, and configurations.

```rust
use stood::agent::Agent;
use stood::llm::models::Bedrock;

let agent = Agent::builder()
    .model(Bedrock::ClaudeHaiku45)
    .temperature(0.7)
    .system_prompt("You are a helpful assistant")
    .build()
    .await?;
```

## Available Models

### AWS Bedrock Models

#### Claude 4.5 Models (Recommended)

| Model | Type | Description |
|-------|--------|-------------|
| `Bedrock::ClaudeSonnet45` | Struct | Balanced performance model for complex agents and coding |
| `Bedrock::ClaudeHaiku45` | Struct | Fastest model with near-frontier intelligence |
| `Bedrock::ClaudeOpus45` | Struct | Maximum intelligence for demanding tasks |

#### Model Aliases (Auto-Upgrade)

Use these aliases to automatically get the latest model version when upgrading the Stood library:

| Alias | Points To | Use Case |
|-------|-----------|----------|
| `Bedrock::SonnetLatest` | `ClaudeSonnet45` | Production applications wanting latest Sonnet |
| `Bedrock::HaikuLatest` | `ClaudeHaiku45` | Production applications wanting latest Haiku |
| `Bedrock::OpusLatest` | `ClaudeOpus45` | Production applications wanting latest Opus |

```rust
use stood::llm::models::Bedrock;

// Using specific version (won't change with library updates)
let agent = Agent::builder()
    .model(Bedrock::ClaudeHaiku45)
    .build().await?;

// Using alias (automatically upgrades with library)
let agent = Agent::builder()
    .model(Bedrock::HaikuLatest)
    .build().await?;
```

#### Amazon Nova Models

| Model | Context Window | Description |
|-------|----------------|-------------|
| `Bedrock::NovaMicro` | Standard | Lightweight, cost-effective |
| `Bedrock::NovaLite` | Standard | Fast inference |
| `Bedrock::NovaPro` | Standard | Balanced capability |
| `Bedrock::NovaPremier` | 300K | Highest capability, vision and video support |
| `Bedrock::Nova2Lite` | 1M | Extended thinking, fast reasoning |
| `Bedrock::Nova2Pro` | 1M | Extended thinking, most intelligent |

#### Legacy Claude Models (Deprecated)

These models are deprecated and will be removed in a future release:

| Model | Replacement |
|-------|-------------|
| `Bedrock::Claude35Sonnet` | `Bedrock::ClaudeSonnet45` |
| `Bedrock::Claude35Haiku` | `Bedrock::ClaudeHaiku45` |
| `Bedrock::ClaudeHaiku3` | `Bedrock::ClaudeHaiku45` |
| `Bedrock::ClaudeOpus3` | `Bedrock::ClaudeOpus45` |

## AgentBuilder Options

All AgentBuilder methods for configuring agent behavior and capabilities:

### Core Configuration

- **`model(M)`** - Set the LLM model (Bedrock::ClaudeHaiku45, LMStudio::Gemma3_12B, etc.)
- **`temperature(f32)`** - Response randomness (0.0-1.0, default: 0.7)
- **`max_tokens(u32)`** - Maximum response length (default: 4096)
- **`system_prompt(String)`** - System prompt for agent behavior
- **`name(String)`** - Agent name for identification
- **`with_id(String)`** - Custom agent ID (auto-generated UUID if not provided)

📖 **Example:** [011_basic_agent.rs](../examples/011_basic_agent.rs) - Demonstrates core agent configuration with different models and parameters

### AWS Credentials

Configure AWS credentials programmatically for the Bedrock provider:

- **`with_credentials(access_key, secret_key, session_token, region)`** - Set credentials directly

```rust
use stood::agent::Agent;
use stood::llm::models::Bedrock;

// With programmatic credentials
let agent = Agent::builder()
    .model(Bedrock::ClaudeHaiku45)
    .with_credentials(
        "AKIAIOSFODNN7EXAMPLE",
        "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        Some("session-token"),  // Optional session token
        "us-east-1",
    )
    .build()
    .await?;
```

For telemetry, you can also configure credentials via `TelemetryConfig`:

```rust
use stood::telemetry::{TelemetryConfig, AwsCredentialSource};

let config = TelemetryConfig::cloudwatch_with_credentials(
    "us-east-1",
    AwsCredentialSource::Profile("my-profile".to_string())
);
```

### Tools Configuration

- **`tool(Box<dyn Tool>)`** - Add a single custom tool
- **`tools(Vec<Box<dyn Tool>>)`** - Add multiple custom tools
- **`with_builtin_tools()`** - Add calculator, file I/O, HTTP, time, and environment tools
- **`with_think_tool(String)`** - Add structured problem-solving tool with custom prompt
- **`with_middleware(Arc<dyn ToolMiddleware>)`** - Add middleware for tool execution interception

📖 **Example:** [001_tool_macro.rs](../examples/001_tool_macro.rs) - Shows how to create and register custom tools with the #[tool] macro

### Tool Middleware

Intercept and modify tool execution with middleware:

```rust
use stood::tools::middleware::{ToolMiddleware, ToolMiddlewareAction, AfterToolAction, ToolContext};
use stood::tools::ToolResult;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug)]
struct ApprovalMiddleware;

#[async_trait]
impl ToolMiddleware for ApprovalMiddleware {
    async fn before_tool(
        &self,
        tool_name: &str,
        params: &Value,
        ctx: &ToolContext,
    ) -> ToolMiddlewareAction {
        // Require approval for file operations
        if tool_name.contains("file") {
            println!("Approve execution of {}? [y/n]", tool_name);
            // ... get user input ...
            return ToolMiddlewareAction::Abort(ToolResult::error("User denied"));
        }
        ToolMiddlewareAction::Continue
    }

    async fn after_tool(
        &self,
        tool_name: &str,
        result: &ToolResult,
        ctx: &ToolContext,
    ) -> AfterToolAction {
        AfterToolAction::PassThrough
    }
}

let agent = Agent::builder()
    .model(Bedrock::ClaudeHaiku45)
    .with_middleware(Arc::new(ApprovalMiddleware))
    .build()
    .await?;
```

📖 **Example:** [027_tool_approval_middleware.rs](../examples/027_tool_approval_middleware.rs) - Interactive tool approval middleware

### MCP Integration

- **`with_mcp_client(MCPClient, Option<String>)`** - Add tools from MCP server with namespace
- **`with_mcp_clients(Vec<(MCPClient, Option<String>)>)`** - Add tools from multiple MCP servers

📖 **Example:** [013_mcp_integration.rs](../examples/013_mcp_integration.rs) - Demonstrates connecting to MCP servers and using external tools

### Retry Configuration

- **`with_retry_config(RetryConfig)`** - Custom retry behavior for LM Studio
- **`with_conservative_retry()`** - Enable 2-attempt retry (LM Studio)
- **`with_aggressive_retry()`** - Enable 5-attempt retry (LM Studio)
- **`without_retry()`** - Disable retry behavior entirely

📖 **Example:** [011_basic_agent.rs](../examples/011_basic_agent.rs) - Shows retry configuration for LM Studio provider resilience

### Telemetry & Observability

- **`with_telemetry(TelemetryConfig)`** - Enable telemetry with custom configuration
- **`with_telemetry_from_env()`** - Enable telemetry from environment variables
- **`with_metrics()`** - Enable comprehensive metrics collection
- **`with_metrics_config(TelemetryConfig)`** - Custom metrics configuration

📖 **Example:** [025_cloudwatch_observability.rs](../examples/025_cloudwatch_observability.rs) - CloudWatch Gen AI integration

### Callbacks & Logging

- **`with_printing_callbacks()`** - Enable real-time execution logging
- **`with_printing_callbacks_config(PrintingConfig)`** - Custom printing configuration
- **`with_verbose_callbacks()`** - Enable verbose development logging
- **`with_performance_callbacks(tracing::Level)`** - Performance logging at specified level
- **`with_callback_handler(H)`** - Custom callback handler implementation
- **`with_batched_printing_callbacks()`** - Batched printing for better performance
- **`with_batched_callbacks(CallbackConfig, BatchConfig)`** - Custom batched callbacks
- **`with_composite_callbacks(Vec<CallbackConfig>)`** - Multiple callback handlers

📖 **Example:** [005_callbacks_basic.rs](../examples/005_callbacks_basic.rs) - Basic callback patterns for real-time execution monitoring

### Execution Configuration

- **`with_streaming(bool)`** - Enable/disable streaming responses
- **`with_timeout(Duration)`** - Set execution timeout
- **`with_log_level(LogLevel)`** - Debug output level (Off, Info, Debug, Trace)
- **`with_execution_config(ExecutionConfig)`** - Direct execution settings
- **`with_event_loop_config(EventLoopConfig)`** - EventLoop behavior settings
- **`with_cancellation()`** - Enable external cancellation support with internal token creation

📖 **Example:** [004_streaming_simple.rs](../examples/004_streaming_simple.rs) - Demonstrates streaming responses and execution configuration

### Parallel Execution

- **`max_parallel_tools(usize)`** - Maximum concurrent tool execution (1 = sequential)
- **`max_parallel_tools_auto()`** - Use CPU count for optimal parallelism
- **`sequential_execution()`** - Force sequential tool execution (alias for max_parallel_tools(1))

📖 **Example:** [017_parallel_execution.rs](../examples/017_parallel_execution.rs) - Shows parallel tool execution patterns and performance optimization

### Evaluation Strategies

- **`with_task_evaluation(String)`** - Enable task completion evaluation with custom prompt
- **`with_multi_perspective_evaluation(Vec<PerspectiveConfig>)`** - Multi-perspective evaluation (see [020_multi_perspective.rs](../examples/020_multi_perspective.rs))
- **`with_agent_based_evaluation(Agent)`** - Separate evaluator agent for task assessment (see [019_agent_based_evaluation.rs](../examples/019_agent_based_evaluation.rs))
- **`with_high_tool_limit(u32)`** - Increase maximum tool iterations (default: 7)

📖 **Example:** [018_task_evaluation.rs](../examples/018_task_evaluation.rs) - Task evaluation strategy for autonomous multi-cycle execution

### Builder Completion
- **`build()`** - Build the configured Agent instance

## Agent Instance Methods

Methods available on Agent instances for interaction and state management:

📖 **Example:** [021_agentic_chat.rs](../examples/021_agentic_chat.rs) - Full interactive chat application demonstrating agent instance usage

### Execution
- **`execute(String)`** - Primary execution method with 5-phase agentic processing

### Conversation Management
- **`add_user_message(String)`** - Add user message to conversation history
- **`add_assistant_message(String)`** - Add assistant message to conversation history
- **`clear_history()`** - Clear all conversation history
- **`conversation_history()`** - Get read-only access to message history

### Agent Information
- **`agent_id()`** - Get agent identifier
- **`agent_name()`** - Get agent name (if set)
- **`config()`** - Get agent configuration
- **`provider()`** - Get LLM provider instance
- **`model()`** - Get model instance
- **`supports_agentic_execution()`** - Check if agent supports agentic workflows
- **`cancellation_token()`** - Get cancellation token for external termination (if configured with `with_cancellation()`)

### State Access
- **`conversation()`** - Get read-only conversation manager
- **`conversation_mut()`** - Get mutable conversation manager
- **`tool_registry()`** - Get tool registry for inspection
- **`get_performance_summary()`** - Get performance metrics and operational summary
- **`create_context(String)`** - Create AgentContext for parent-child tracking

## agent.execute(prompt)

Execute a task using the unified agent interface with 5-phase agentic execution. This is the primary method for all agent interactions, automatically handling tool selection, execution, and response generation.

```rust
let result = agent.execute("Calculate the square root of 144 and save it to a file").await?;
println!("Response: {}", result.response);
println!("Tools used: {}", result.tools_called.len());
println!("Duration: {:?}", result.duration);
```

**Arguments:**
- `prompt: String` - Your task, question, or instruction for the agent

**Returns:**
- `AgentResult` containing response text, execution metrics, tools used, and performance data
- `result.response` - The final answer or result text
- `result.tools_called` - Vector of tool names that were executed
- `result.duration` - Total execution time
- `result.success` - Whether execution completed successfully
- `result.execution` - Detailed execution metrics (cycles, token usage, etc.)
- `result.used_tools` - Boolean indicating if any tools were used

## Cancellation

The agent supports external cancellation for long-running operations:

```rust
use stood::agent::Agent;
use stood::llm::models::Bedrock;
use tokio::time::{sleep, Duration};

// Build agent with cancellation support
let mut agent = Agent::builder()
    .model(Bedrock::ClaudeHaiku45)
    .with_cancellation()
    .build()
    .await?;

// Get cancellation token for external control
let token = agent.cancellation_token().expect("Cancellation enabled");

// Spawn task to cancel after timeout
let cancel_token = token.clone();
tokio::spawn(async move {
    sleep(Duration::from_secs(30)).await;
    cancel_token.cancel();
});

// Execute - will be cancelled if timeout expires
match agent.execute("Long running task...").await {
    Ok(result) => println!("Completed: {}", result.response),
    Err(e) if e.to_string().contains("cancelled") => println!("Task was cancelled"),
    Err(e) => return Err(e.into()),
}
```

## See Also

- [Tools](tools.md) - Tool development and middleware
- [Telemetry](telemetry.md) - CloudWatch Gen AI Observability
- [Streaming](streaming.md) - Real-time response handling
- [Source Code](../src/agent/mod.rs) - Agent module implementation
