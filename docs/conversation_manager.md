# Conversation Manager

*Tool-aware conversation management with context overflow recovery*

## Overview

The conversation manager enables you to maintain conversation coherence while respecting size limits and tool interaction integrity. You'll get sliding window management, tool-aware pruning, and context overflow recovery that preserves meaningful conversation flow without breaking tool sequences.

## Key Components

- 📚 [ConversationManager Trait](../src/conversation_manager/mod.rs) - Abstract interface for conversation management strategies
- 📚 [SlidingWindowManager](../src/conversation_manager/mod.rs) - Window-based conversation limiting with tool awareness
- 📚 [NullConversationManager](../src/conversation_manager/mod.rs) - No-op implementation for testing scenarios
- 📚 [ConversationManagerFactory](../src/conversation_manager/mod.rs) - Factory pattern for creating different manager types

## What You Get

*Tool-Aware Management*
- Never separates tool use from tool result messages
- Finds safe conversation boundaries for trimming
- Identifies and cleans orphaned tool messages
- Preserves tool sequence relationships during reduction

*Intelligent Conversation Limiting*
- Dual-limit strategy supporting both message count and token limits
- Priority-based reduction through context manager integration
- Configurable window sizes (default: 40 messages following Python reference)
- Conversation continuity that maintains meaningful flow

*Context Overflow Recovery*
- Aggressive trimming when model limits exceeded
- Fallback strategies for multiple management approaches
- Error context preservation in management decisions
- Recovery verification to ensure overflow resolution

*Production Integration*
- Event loop coordination with management after each cycle
- Tool execution timing coordination
- Error recovery with context overflow exception handling
- State consistency that maintains conversation integrity

## Quick Start

Configure conversation management:

```rust
use stood::conversation_manager::{SlidingWindowManager, ConversationConfig};
use stood::context_manager::ContextConfig;

// Production configuration with context integration
let context_config = ContextConfig {
    max_tokens: 100_000,
    buffer_percentage: 0.8,
    ..Default::default()
};

let config = ConversationConfig {
    max_messages: 50,
    enable_context_management: true,
    context_config: Some(context_config),
    enable_tool_aware_pruning: true,
    auto_clean_dangling: true,
    ..Default::default()
};

let manager = SlidingWindowManager::with_config(config);
```

Apply management after agent cycles:

```rust
// Regular conversation management
let result = manager.apply_management(&mut conversation).await?;

if result.changes_made {
    info!("Conversation managed: {}", result.description);
    info!("  Messages removed: {}", result.messages_removed);
    info!("  Dangling cleaned: {}", result.dangling_cleaned);
}
```

Handle context overflow scenarios:

```rust
// Context overflow recovery
match agent.process_turn(&conversation).await {
    Err(StoodError::QuotaExceeded { message }) if message.contains("context") => {
        let result = manager.reduce_context(
            &mut conversation, 
            Some(&message)
        ).await?;
        
        info!("Context reduced: {}", result.description);
        
        // Retry with reduced context
        agent.process_turn(&conversation).await
    }
    result => result,
}
```

## Management Strategies

*Sliding Window* (default)
```rust
// Default configuration - 40 messages following Python reference
let manager = SlidingWindowManager::new();

// Custom window size
let manager = SlidingWindowManager::with_window_size(30);
```

*Context-Aware Management*
```rust
let config = ConversationConfig {
    max_messages: 60,                    // High message limit
    enable_context_management: true,     // Token-based limits take precedence
    ..Default::default()
};
```

*Null Manager* (testing)
```rust
let manager = NullConversationManager::new();
// No management applied - useful for debugging and testing
```

## Tool Sequence Preservation

The system ensures tool interactions remain intact:

```rust
// Tool use/result pairs are never separated
let messages = vec![
    Message::assistant_with_tool_use("call_123", "calculator", json!({"expr": "2+2"})),
    Message::user_with_tool_result("call_123", "4"),
];

// Management preserves both messages together
let result = manager.apply_management(&mut messages).await?;
```

Orphaned tool messages are cleaned automatically:

```rust
// These will be removed during management
let orphaned_messages = vec![
    Message::user_with_tool_result("missing_123", "orphaned result"), // No tool use
    Message::assistant_with_tool_use("orphan_456", "calculator", json!({})), // No result
];
```

## Management Results

Track what happened during management:

```rust
let result = manager.apply_management(&mut conversation).await?;

println!("Management Summary:");
println!("  Changes made: {}", result.changes_made);
println!("  Messages removed: {}", result.messages_removed);  
println!("  Dangling cleaned: {}", result.dangling_cleaned);
println!("  Description: {}", result.description);
```

## Safe Trimming Algorithm

The system finds safe points to trim conversations:

1. *Tool Relationship Analysis* - Maps tool use to tool result relationships
2. *Boundary Detection* - Identifies conversation points safe for trimming
3. *Coherence Preservation* - Ensures no orphaned tool messages created
4. *Context Integrity* - Maintains conversation flow and meaning

```rust
// Internal algorithm ensures safe trimming
let trim_index = manager.find_safe_trim_index(&messages, target_size);
// Returns index that won't break tool sequences
```

## Factory Pattern Usage

Create managers with the factory:

```rust
use stood::conversation_manager::ConversationManagerFactory;

// Production configuration
let manager = ConversationManagerFactory::sliding_window_with_config(
    ConversationConfig {
        max_messages: 60,
        enable_context_management: true,
        ..Default::default()
    }
);

// Testing configuration
let test_manager = ConversationManagerFactory::null();

// Simple configuration
let simple_manager = ConversationManagerFactory::sliding_window_with_size(25);
```

## Integration with Agent Cycles

Embed management in your agent event loop:

```rust
pub async fn process_agent_cycle(
    agent: &Agent,
    conversation: &mut Messages,
    manager: &dyn ConversationManager,
) -> Result<()> {
    // 1. Process user input and tools
    let response = agent.process_turn(conversation).await?;
    conversation.push(response);
    
    // 2. Apply conversation management
    let mgmt_result = manager.apply_management(conversation).await?;
    if mgmt_result.changes_made {
        debug!("Conversation managed: {}", mgmt_result.description);
    }
    
    Ok(())
}
```

## Context Integration

When context management is enabled:

```rust
// Context manager handles token-based limits
// Conversation manager provides message-count fallback
let result = manager.apply_management(&mut messages).await?;

if result.description.contains("context-aware") {
    // Context manager was used for token-based reduction
    info!("Token-based management applied");
} else {
    // Message-count limits were used
    info!("Message-count management applied");
}
```

## Error Scenarios

Handle different error conditions:

*Context Overflow*
```rust
let result = manager.reduce_context(&mut messages, Some("Context too large")).await?;
// Applies aggressive trimming (75% of window size)
```

*Management Failures*
```rust
match manager.apply_management(&mut messages).await {
    Ok(result) => info!("Management successful: {}", result.description),
    Err(error) => {
        error!("Management failed: {}", error);
        // Fallback to simple truncation
        messages.truncate_to_last(20);
    }
}
```

## Performance Characteristics

*Time Complexity*
- O(n) for conversation analysis and trimming
- Efficient tool relationship mapping
- Single-pass orphan detection and cleanup

*Memory Usage*
- Immediate message removal without copying
- Minimal overhead for relationship tracking
- Constant memory usage regardless of conversation size

*Management Overhead*
- Lazy evaluation - only when limits approached
- Batch operations handle multiple concerns in single pass
- Non-blocking operation between agent cycles

## Python Reference Alignment

Follows Python reference implementation patterns:

- Default 40 message limit matches `SlidingWindowConversationManager`
- Tool awareness uses same preservation logic
- `NullConversationManager` throws exceptions like Python version
- Management timing applied after event loop cycles

## Related Documentation

**architecture** - Conversation management architecture and design patterns
**patterns** - Long-running conversation patterns and strategies
**troubleshooting** - Conversation debugging and overflow recovery
**performance** - Conversation optimization and efficiency techniques
