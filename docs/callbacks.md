# Callback System

The Stood agent library provides a comprehensive callback system for monitoring and controlling agent execution in real-time. Callbacks enable you to observe internal events, handle streaming content, monitor performance, and implement custom business logic.

## Overview

Callbacks in Stood are event-driven handlers that respond to various stages of agent execution:

```rust
use stood::agent::Agent;
use stood::agent::callbacks::PrintingConfig;

// Basic printing callbacks
let agent = Agent::builder()
    .with_printing_callbacks()
    .build()
    .await?;

// Verbose callbacks with reasoning
let agent = Agent::builder()
    .with_verbose_callbacks()
    .build()
    .await?;
```

## Available Callback Configurations

### 1. `with_printing_callbacks()`

**Purpose**: Basic real-time console output for development and debugging.

**How it works**: Provides immediate visual feedback during agent execution with clean, formatted output. Displays streaming content, tool executions, and execution summaries without overwhelming detail.

**Usage**:
```rust
let agent = Agent::builder()
    .with_printing_callbacks()
    .build()
    .await?;
```

**Output Example**:
```
🔧 Executing tool: calculator
✅ Tool calculator completed in 132ms
📊 Execution completed successfully!
```

### 2. `with_verbose_callbacks()`

**Purpose**: Enhanced printing with reasoning traces and detailed execution context.

**How it works**: Shows LLM thinking process and decision-making for debugging complex workflows. Includes all standard printing output plus internal reasoning, evaluation decisions, and detailed context information.

**Usage**:
```rust
let agent = Agent::builder()
    .with_verbose_callbacks()
    .build()
    .await?;
```

**Additional Output**:
```
🤔 Evaluation (self_reflection) 🔄 -> CONTINUE (took 1.2s)
💭 I need to verify the calculation is correct...
   Additional content: The result looks accurate based on the formula
```

### 3. `with_performance_callbacks(tracing::Level::INFO)`

**Purpose**: Structured logging of performance metrics using the tracing framework.

**How it works**: Captures timing, token usage, and system performance data for monitoring and optimization. Integrates with standard Rust logging infrastructure and can be sent to observability platforms like Prometheus, Jaeger, or custom log aggregators.

**Usage**:
```rust
use tracing::Level;

let agent = Agent::builder()
    .with_performance_callbacks(Level::INFO)
    .build()
    .await?;
```

**Log Output**:
```
2025-01-15T10:30:45.123Z INFO [stood::agent] tool=calculator duration=132ms Tool execution completed
2025-01-15T10:30:45.456Z INFO [stood::agent] duration=5.2s cycles=1 tools=1 Agent execution completed
```

### 4. `with_callback_handler(custom_handler)`

**Purpose**: Integration point for implementing your own custom callback logic.

**How it works**: Allows complete control over event handling for specialized use cases like custom UIs, analytics pipelines, or business logic integration. You implement the `CallbackHandler` trait to define custom behavior for each event type.

**Usage**:
```rust
use stood::agent::callbacks::{CallbackHandler, CallbackEvent, CallbackError};
use async_trait::async_trait;

struct CustomHandler {
    // Your custom state
}

#[async_trait]
impl CallbackHandler for CustomHandler {
    async fn handle_event(&self, event: CallbackEvent) -> Result<(), CallbackError> {
        match event {
            CallbackEvent::ToolStart { tool_name, .. } => {
                // Custom logic for tool start
                self.log_to_analytics(&tool_name).await?;
            }
            CallbackEvent::ContentDelta { delta, .. } => {
                // Custom UI updates
                self.update_ui(&delta).await?;
            }
            _ => {}
        }
        Ok(())
    }
}

let agent = Agent::builder()
    .with_callback_handler(CustomHandler::new())
    .build()
    .await?;
```

### 5. `with_composite_callbacks(vec![config1, config2])`

**Purpose**: Combines multiple callback handlers to run simultaneously during execution.

**How it works**: Enables scenarios like real-time UI updates plus background performance logging. Events are dispatched to all registered handlers in order, allowing you to compose different callback behaviors without conflicts.

**Usage**:
```rust
use stood::agent::callbacks::{CallbackHandlerConfig, PrintingConfig};
use tracing::Level;

let composite_config = vec![
    CallbackHandlerConfig::Printing(PrintingConfig::default()),
    CallbackHandlerConfig::Performance(Level::INFO),
    CallbackHandlerConfig::Custom(Arc::new(CustomHandler::new())),
];

let agent = Agent::builder()
    .with_composite_callbacks(composite_config)
    .build()
    .await?;
```

## Callback Events

The callback system responds to these key events:

### Agent Lifecycle
- `EventLoopStart` - Agent execution begins
- `EventLoopComplete` - Agent execution completes
- `CycleStart` - New reasoning cycle begins
- `CycleComplete` - Reasoning cycle completes

### Model Interactions
- `ModelStart` - LLM invocation begins
- `ModelComplete` - LLM response received
- `ContentDelta` - Streaming content chunk received

### Tool Execution
- `ToolStart` - Tool execution begins
- `ToolComplete` - Tool execution completes
- `ParallelStart` - Parallel tool execution begins
- `ParallelProgress` - Parallel execution progress update

### Evaluation Events
- `EvaluationStart` - Agent evaluation begins
- `EvaluationComplete` - Agent evaluation completes with decision

## Advanced Configuration

### Custom Printing Configuration

```rust
use stood::agent::callbacks::PrintingConfig;

let config = PrintingConfig {
    show_reasoning: true,      // Show LLM reasoning
    show_tools: true,          // Show tool executions
    show_performance: true,    // Show performance metrics
    stream_output: true,       // Enable streaming output
};

let agent = Agent::builder()
    .with_printing_callbacks_config(config)
    .build()
    .await?;
```

### Batched Callbacks

For high-throughput scenarios, you can enable callback batching:

```rust
let agent = Agent::builder()
    .with_batched_printing_callbacks()
    .build()
    .await?;
```

## Performance Considerations

- **Printing Callbacks**: Minimal overhead, suitable for development
- **Verbose Callbacks**: Higher overhead due to detailed logging
- **Performance Callbacks**: Low overhead, designed for production use
- **Custom Handlers**: Overhead depends on your implementation
- **Composite Callbacks**: Additive overhead of all configured handlers

## Integration Examples

### Web UI Integration

```rust
struct WebUIHandler {
    websocket: Arc<Mutex<WebSocket>>,
}

#[async_trait]
impl CallbackHandler for WebUIHandler {
    async fn handle_event(&self, event: CallbackEvent) -> Result<(), CallbackError> {
        let message = serde_json::to_string(&event)?;
        self.websocket.lock().await.send(message).await?;
        Ok(())
    }
}
```

### Metrics Collection

```rust
struct MetricsHandler {
    metrics_client: PrometheusClient,
}

#[async_trait]
impl CallbackHandler for MetricsHandler {
    async fn handle_event(&self, event: CallbackEvent) -> Result<(), CallbackError> {
        match event {
            CallbackEvent::ToolComplete { tool_name, duration, .. } => {
                self.metrics_client.record_tool_duration(&tool_name, duration).await;
            }
            _ => {}
        }
        Ok(())
    }
}
```

## Best Practices

1. **Development**: Use `with_printing_callbacks()` for basic feedback
2. **Debugging**: Use `with_verbose_callbacks()` for detailed troubleshooting
3. **Production**: Use `with_performance_callbacks()` for monitoring
4. **Custom UIs**: Implement `CallbackHandler` for specialized interfaces
5. **Multiple Needs**: Use `with_composite_callbacks()` to combine approaches

## See Also

- [Examples](examples.md) - Practical callback examples
- [Streaming](streaming.md) - Stream integration with callback patterns
- [Patterns](patterns.md) - Advanced callback implementation patterns