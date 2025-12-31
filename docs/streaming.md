[#](#) Streaming System

*Real-time processing of model responses with tool execution during streaming*

## Overview

The streaming system enables you to process LLM provider responses in real-time, providing immediate user feedback and progressive tool execution. You'll get token-by-token delivery, incremental message building, and seamless integration with the agent framework.

## Key Components

- 📚 [StreamEvent](../src/streaming/mod.rs) - Typed event system for provider streaming responses
- 📚 [StreamingMessage](../src/streaming/mod.rs) - Incremental message building from stream chunks
- 📚 [StreamProcessor](../src/streaming/mod.rs) - Provider stream conversion to typed events
- 📚 [StreamCallback](../src/streaming/mod.rs) - Real-time event handling interface for applications

## What You Get

*Real-Time Responsiveness*
- Token-by-token delivery for immediate user feedback
- Progressive tool execution as tool calls are identified
- Background processing with non-blocking event handling
- Buffer management to prevent memory overflow

*Content Type Support*
- Text responses with incremental building
- Tool invocation JSON with fragmented assembly
- Model reasoning content with signature verification
- Content redaction support for safety compliance

*Error Resilience*
- Stream interruption recovery with partial message preservation
- Malformed event handling with graceful degradation
- Timeout management for hung or slow streams
- Content validation for security and correctness

*Performance Optimization*
- Minimal memory allocation during stream processing
- Efficient JSON parsing for tool use content
- Channel-based architecture for high throughput
- Configurable buffering for memory/latency tradeoffs

## Quick Start

Configure streaming behavior:

```rust
use stood::streaming::{StreamConfig, StreamProcessor};
use std::time::Duration;

// Production configuration
let config = StreamConfig {
    enabled: true,
    buffer_size: 200,                    // Higher capacity for production
    timeout: Duration::from_secs(60),    // Extended timeout
    enable_tool_streaming: true,         // Real-time tool execution
};

let processor = StreamProcessor::new(config);
```

Process AWS SDK streams:

```rust
use stood::streaming::StreamingMessage;
use stood::types::MessageRole;

// Process streaming response
let event_stream = processor.process_aws_stream(aws_stream).await?;
let mut streaming_message = StreamingMessage::new(MessageRole::Assistant);

while let Some(event) = event_stream.recv().await {
    streaming_message.process_event(event)?;
    
    if streaming_message.is_complete() {
        break;
    }
}

let final_message = streaming_message.current_message();
```

## Stream Events

The system handles multiple event types:

*Message Lifecycle Events*
- MessageStart - Conversation initiation
- ContentBlockStart - Content type detection (text, tool use, reasoning)
- ContentBlockDelta - Incremental content updates
- ContentBlockStop - Content finalization
- MessageStop - Response completion

*Metadata Events*
- Metadata - Usage and performance data
- TokenUsage - Input/output token tracking
- Metrics - Latency and performance information

*Error Events*
- InternalServerException - Service errors
- ModelStreamErrorException - Model-specific errors
- ThrottlingException - Rate limiting
- ValidationException - Input validation errors

## Real-Time UI Integration

Implement callbacks for live UI updates:

```rust
use stood::streaming::StreamCallback;

struct UICallback;

impl StreamCallback for UICallback {
    fn on_event(&self, event: &StreamEvent) {
        match event {
            StreamEvent::ContentBlockDelta(delta_event) => {
                // Update UI with incremental content
                update_ui_with_delta(&delta_event.delta);
            }
            StreamEvent::MessageStop(_) => {
                // Finalize UI state
                finalize_message_ui();
            }
            StreamEvent::Metadata(metadata) => {
                // Update performance indicators
                update_performance_ui(&metadata.metrics);
            }
            _ => {}
        }
    }

    fn on_complete(&self, message: &Message, usage: &Usage, metrics: &Metrics) {
        info!("Streaming complete: {} tokens in {:.2}s", 
              usage.total_tokens, metrics.latency_ms as f64 / 1000.0);
    }
}
```

## Tool Execution During Streaming

Execute tools as they become available:

```rust
// Monitor for tool use events
match event {
    StreamEvent::ContentBlockStart(start_event) => {
        if let ContentBlockStart::ToolUse { name, .. } = start_event.start {
            // Prepare for tool execution
            tool_executor.prepare_tool(&name);
        }
    }
    StreamEvent::ContentBlockStop(_) => {
        // Execute tool with complete input
        if let Some(tool_call) = streaming_message.get_last_tool_call() {
            let result = tool_executor.execute(tool_call).await?;
            // Continue conversation with tool result
        }
    }
}
```

## Content Types

*Text Content*
```rust
StreamEvent::ContentBlockDelta(ContentBlockDeltaEvent {
    delta: ContentBlockDelta::Text(ContentBlockDeltaText {
        text: "Hello".to_string(),
    }),
    ..
})
```

*Tool Use Content*  
```rust
StreamEvent::ContentBlockDelta(ContentBlockDeltaEvent {
    delta: ContentBlockDelta::ToolUse(ContentBlockDeltaToolUse {
        input: r#"{"expression": "2 + 2"}"#.to_string(),
    }),
    ..
})
```

*Reasoning Content*
```rust
StreamEvent::ContentBlockDelta(ContentBlockDeltaEvent {
    delta: ContentBlockDelta::ReasoningContent(ReasoningContentBlockDelta {
        text: Some("Let me think about this...".to_string()),
        signature: Some("verification_token".to_string()),
        redacted_content: None,
    }),
    ..
})
```

## Message Assembly

The StreamingMessage handles incremental building:

```rust
let mut streaming_message = StreamingMessage::new(MessageRole::Assistant);

// Events are processed in order
streaming_message.process_event(message_start_event)?;
streaming_message.process_event(content_block_start_event)?;
streaming_message.process_event(content_delta_event)?;
streaming_message.process_event(content_block_stop_event)?;
streaming_message.process_event(message_stop_event)?;

// Check completion
if streaming_message.is_complete() {
    let final_message = streaming_message.current_message().clone();
}
```

## Safety and Compliance

*Content Redaction*
- Support for sensitive information redaction
- Automatic content replacement when redaction events received
- Signature verification for reasoning content authenticity

*Error Masking*
- Production-safe error messages
- Detailed debug information in development
- Audit logging for compliance requirements

## Performance Characteristics

*Memory Usage*
- Constant memory usage for streaming regardless of response length
- Efficient string concatenation for text content
- JSON parsing only when tool content is complete

*Latency*
- Sub-100ms event processing for typical responses
- Non-blocking channel operations
- Configurable buffer sizes for throughput tuning

*Throughput*
- Handles thousands of concurrent streams
- Channel-based architecture scales with available cores
- Memory pooling for high-frequency allocations

## Agent Integration

*Event Loop Coordination*
- Streaming response processing during agent cycles
- Tool execution triggering during streaming
- Conversation management with incremental updates
- Error recovery coordination with retry systems

*State Management*
- Preserve partial messages during interruptions
- Coordinate with conversation management
- Maintain tool execution context across streams

## Cancellation Support

The streaming system integrates with Stood's cancellation mechanism for stopping long-running operations:

```rust
use stood::agent::Agent;
use stood::llm::models::Bedrock;
use tokio::time::{sleep, Duration};

// Build agent with cancellation and streaming
let mut agent = Agent::builder()
    .model(Bedrock::ClaudeHaiku45)
    .with_streaming(true)
    .with_cancellation()
    .build()
    .await?;

// Get cancellation token
let token = agent.cancellation_token().expect("Cancellation enabled");

// Cancel from another task
let cancel_token = token.clone();
tokio::spawn(async move {
    sleep(Duration::from_secs(30)).await;
    cancel_token.cancel();
});

// Execute with cancellation support
match agent.execute("Generate a long report...").await {
    Ok(result) => println!("Completed: {}", result.response),
    Err(e) if e.to_string().contains("cancelled") => {
        println!("Streaming was cancelled");
    }
    Err(e) => return Err(e.into()),
}
```

### Cancellation Behavior

When cancellation is triggered during streaming:
- The current stream is immediately terminated
- Partial content received is preserved
- Pending tool executions receive synthetic "cancelled" results
- The agent returns a cancellation error

## GUI Integration Example

For building streaming UIs with real-time updates, see the enterprise prompt builder example:

```bash
cargo run --example 024_enterprise_prompt_builder
```

This example demonstrates:
- Real-time content streaming to egui panels
- Tool execution status indicators
- Cancellation button integration
- Token usage display during streaming

## Related Documentation

- [API](api.md) - Agent builder cancellation configuration
- [Callbacks](callbacks.md) - Event handling for streaming updates
- [Architecture](architecture.md) - Real-time processing architecture
- [Source Code](../src/streaming/mod.rs) - Streaming module implementation
