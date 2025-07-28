# API Documentation

This section provides detailed API documentation for the core Stood agent library components.

## Agent

### Agent::builder()

Creates a new AgentBuilder for configuring and constructing Agent instances. The builder provides a fluent interface for setting up agents with custom models, parameters, and configurations.

```rust
use stood::agent::Agent;
use stood::llm::models::Bedrock;

let agent = Agent::builder()
    .model(Bedrock::Claude35Haiku)
    .temperature(0.7)
    .system_prompt("You are a helpful assistant")
    .build()
    .await?;
```

## AgentBuilder Options

All AgentBuilder methods for configuring agent behavior and capabilities:

### Core Configuration

- **`model(M)`** - Set the LLM model (Bedrock::Claude35Haiku, LMStudio::Gemma3_12B, etc.)
- **`temperature(f32)`** - Response randomness (0.0-1.0, default: 0.7)
- **`max_tokens(u32)`** - Maximum response length (default: 4096)
- **`system_prompt(String)`** - System prompt for agent behavior
- **`name(String)`** - Agent name for identification
- **`with_id(String)`** - Custom agent ID (auto-generated UUID if not provided)
- 
📖 **Example:** [011_basic_agent.rs](../examples/011_basic_agent.rs) - Demonstrates core agent configuration with different models and parameters

### Tools Configuration

- **`tool(Box<dyn Tool>)`** - Add a single custom tool
- **`tools(Vec<Box<dyn Tool>>)`** - Add multiple custom tools
- **`with_builtin_tools()`** - Add calculator, file I/O, HTTP, time, and environment tools
- **`with_think_tool(String)`** - Add structured problem-solving tool with custom prompt

📖 **Example:** [001_tool_macro.rs](../examples/001_tool_macro.rs) - Shows how to create and register custom tools with the #[tool] macro

### MCP Integration

- **`with_mcp_client(MCPClient, Option<String>)`** - Add tools from MCP server with namespace
- **`with_mcp_clients(Vec<(MCPClient, Option<String>)>)`** - Add tools from multiple MCP servers

📖 **Example:** [013_mcp_integration.rs](../examples/013_mcp_integration.rs) - Demonstrates connecting to MCP servers and using external tools

### Retry Configuration

- **`with_retry_config(RetryConfig)`** - Custom retry behavior for LM Studio
- **`with_conservative_retry()`** - Enable 2-attempt retry (LM Studio)
- **`with_aggressive_retry()`** - Enable 5-attempt retry (LM Studio)  
- **`without_retry()`** - Disable retry behavior entirely
- 
📖 **Example:** [011_basic_agent.rs](../examples/011_basic_agent.rs) - Shows retry configuration for LM Studio provider resilience

### Telemetry & Observability

- **`with_telemetry(TelemetryConfig)`** - Enable telemetry with custom configuration
- **`with_telemetry_from_env()`** - Enable telemetry from environment variables
- **`with_metrics()`** - Enable comprehensive metrics collection
- **`with_metrics_config(TelemetryConfig)`** - Custom metrics configuration
- 
📖 **Example:** [023_telemetry/](../examples/023_telemetry/) - Comprehensive telemetry integration with OpenTelemetry

### Callbacks & Logging

- **`with_printing_callbacks()`** - Enable real-time execution logging
- **`with_printing_callbacks_config(PrintingConfig)`** - Custom printing configuration
- **`with_verbose_callbacks()`** - Enable verbose development logging
- **`with_performance_callbacks(tracing::Level)`** - Performance logging at specified level
- **`with_callback_handler(H)`** - Custom callback handler implementation
- **`with_batched_printing_callbacks()`** - Batched printing for better performance
- **`with_batched_callbacks(CallbackConfig, BatchConfig)`** - Custom batched callbacks
- **`with_composite_callbacks(Vec<CallbackConfig>)`** - Multiple callback handlers
- 
📖 **Example:** [005_callbacks_basic.rs](../examples/005_callbacks_basic.rs) - Basic callback patterns for real-time execution monitoring

### Execution Configuration

- **`with_streaming(bool)`** - Enable/disable streaming responses
- **`with_timeout(Duration)`** - Set execution timeout
- **`with_log_level(LogLevel)`** - Debug output level (Off, Info, Debug, Trace)
- **`with_execution_config(ExecutionConfig)`** - Direct execution settings
- **`with_event_loop_config(EventLoopConfig)`** - EventLoop behavior settings
- 
📖 **Example:** [004_streaming_simple.rs](../examples/004_streaming_simple.rs) - Demonstrates streaming responses and execution configuration

### Parallel Execution

- **`max_parallel_tools(usize)`** - Maximum concurrent tool execution (1 = sequential)
- **`max_parallel_tools_auto()`** - Use CPU count for optimal parallelism
- **`sequential_execution()`** - Force sequential tool execution (alias for max_parallel_tools(1))
- 
📖 **Example:** [017_parallel_execution.rs](../examples/017_parallel_execution.rs) - Shows parallel tool execution patterns and performance optimization

### Evaluation Strategies

- **`with_task_evaluation(String)`** - Enable task completion evaluation with custom prompt
- **`with_multi_perspective_evaluation(Vec<PerspectiveConfig>)`** - Multi-perspective evaluation (see [020_multi_perspective.rs](../examples/020_multi_perspective.rs))
- **`with_agent_based_evaluation(Agent)`** - Separate evaluator agent for task assessment (see [019_agent_based_evaluation.rs](../examples/019_agent_based_evaluation.rs))
- **`with_high_tool_limit(u32)`** - Increase maximum tool iterations (default: 7)
- 
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
