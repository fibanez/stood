# Context Manager

*Intelligent context window management for long-running conversations*

## Overview

The context manager enables you to prevent context overflow errors and optimize conversation size automatically. You'll get token counting, proactive overflow prevention, and priority-based message retention that keeps your conversations running smoothly without hitting model limits.

## Key Components

- 📚 [ContextManager](../src/context_manager/mod.rs) - Context usage analysis with token estimation and breakdown
- 📚 [ContextConfig](../src/context_manager/mod.rs) - Configuration for token limits and management strategies  
- 📚 [ContextUsage](../src/context_manager/mod.rs) - Multi-metric tracking with threshold monitoring
- 📚 [MessagePriority](../src/context_manager/mod.rs) - Five-tier priority system for smart message retention

## What You Get

*Intelligent Analysis*
- Token estimation using character-to-token ratios
- Content type breakdown for targeted optimization
- Usage percentage calculation against model limits
- Threshold monitoring for approaching and exceeding safe limits

*Proactive Management*
- Early warning system that detects approaching limits
- Automatic reduction when safe thresholds exceeded
- Configurable thresholds (90% of safe limit triggers warnings)
- Non-disruptive operation between agent cycles

*Priority-Based Retention*
- System message protection with Critical priority
- Recent conversation bias (last 20% gets higher priority)
- Tool interaction preservation for functional context
- Gradual degradation that removes lowest priority messages first

*Model-Aware Configuration*
- Default settings for Claude 3.5 Sonnet (200k tokens)
- Conservative token estimation (4.0 chars/token) for safety
- 85% utilization threshold to prevent overflow
- Minimum message preservation for conversation continuity

## Quick Start

Configure context management:

```rust
use stood::context_manager::{ContextManager, ContextConfig};

// Production configuration
let config = ContextConfig {
    max_tokens: 200_000,         // Claude 3.5 Sonnet limit
    buffer_percentage: 0.80,     // 80% utilization threshold
    chars_per_token: 3.5,        // Optimized estimation ratio
    enable_proactive_prevention: true,
    enable_priority_retention: true,
    min_messages: 5,             // Preserve more context
};

let manager = ContextManager::with_config(config);
```

Analyze conversation usage:

```rust
// Check current context usage
let usage = manager.analyze_usage(&messages);

if usage.exceeds_safe_limit {
    warn!("Context at {:.1}% capacity", usage.usage_percentage);
}

if usage.approaching_limit {
    info!("Context approaching limit, management may be needed soon");
}
```

Apply proactive management:

```rust
// Check if management is needed
if manager.needs_management(&messages) {
    let result = manager.manage_context(&mut messages)?;
    
    if result.changes_made {
        info!("Context managed: removed {} messages, saved {} chars", 
              result.messages_removed, result.characters_saved);
    }
}
```

## Priority System

Messages are assigned priorities based on role, recency, and tool interactions:

*Critical (4)*
- System messages and initial context
- Never removed during management

*High (3)*  
- Recent user messages (last 20% of conversation)
- Messages with tool interactions in recent conversation

*Medium (2)*
- Assistant responses with tool usage
- Recent messages without tools

*Normal (1)*
- General conversation messages
- Older messages without special significance

*Low (0)*
- Old messages that can be safely removed
- Messages identified as least important

## Content Analysis

The system provides detailed breakdown of context usage:

```rust
let usage = manager.analyze_usage(&messages);

println!("Context Breakdown:");
println!("  Text content: {} chars", usage.content_breakdown.text_chars);
println!("  Tool use: {} chars", usage.content_breakdown.tool_use_chars);
println!("  Tool results: {} chars", usage.content_breakdown.tool_result_chars);
println!("  Thinking: {} chars", usage.content_breakdown.thinking_chars);
println!("  Tool interactions: {}", usage.content_breakdown.tool_interactions);
```

## Agent Integration

Integrate with agent event loops:

```rust
// Before each agent cycle
let usage = context_manager.analyze_usage(&conversation.messages);
if usage.approaching_limit {
    warn!("Context approaching limit: {:.1}%", usage.usage_percentage);
}

// Proactive management
if context_manager.needs_management(&conversation.messages) {
    let result = context_manager.manage_context(&mut conversation.messages)?;
    info!("Context managed: removed {} messages", result.messages_removed);
}

// Continue with agent processing
let response = agent.process_turn(&conversation).await?;
```

## Management Strategies

*Simple Sliding Window*
```rust
let config = ContextConfig {
    enable_priority_retention: false,
    ..Default::default()
};
// Removes messages from beginning (FIFO)
```

*Priority-Based Retention*
```rust
let config = ContextConfig {
    enable_priority_retention: true,
    ..Default::default()
};
// Removes lowest priority messages first, preserves important context
```

## Token Estimation

The system uses conservative character-to-token ratios:

```rust
// Estimate tokens for a single message
let tokens = manager.estimate_message_tokens(&message);

// Get maximum safe token count
let safe_limit = manager.max_safe_tokens();
// Returns: max_tokens * buffer_percentage
```

## Management Results

Track what happened during management:

```rust
let result = manager.manage_context(&mut messages)?;

if result.changes_made {
    println!("Management Results:");
    println!("  Messages removed: {}", result.messages_removed);
    println!("  Characters saved: {}", result.characters_saved);
    println!("  Before: {} tokens", result.before_usage.estimated_tokens);
    println!("  After: {} tokens", result.after_usage.estimated_tokens);
    
    // Priority breakdown
    for (priority, count) in result.removed_by_priority {
        println!("  Removed {} {:?} priority messages", count, priority);
    }
}
```

## Error Recovery Integration

Coordinate with error recovery for context overflow:

```rust
use stood::error_recovery::ErrorClassifier;

// During agent processing
match agent.process_turn(&conversation).await {
    Err(error) if ErrorClassifier::is_context_overflow(&error) => {
        // Use context manager for recovery
        let result = context_manager.manage_context(&mut conversation)?;
        info!("Context overflow recovered: {}", result.description);
        
        // Retry with reduced context
        agent.process_turn(&conversation).await
    }
    result => result,
}
```

## Performance Monitoring

Track context utilization over time:

```rust
// Collect metrics
let usage = context_manager.analyze_usage(&messages);

metrics.record_gauge("context.usage_percentage", usage.usage_percentage);
metrics.record_gauge("context.estimated_tokens", usage.estimated_tokens as f64);
metrics.record_gauge("context.message_count", usage.message_count as f64);

// Alert on approaching limits
if usage.approaching_limit {
    alerts.send_warning("Context approaching limit", &usage);
}
```

## Common Patterns

*Conversation Continuity*
- Preserve important messages during reduction
- Maintain conversation flow despite message removal
- Keep tool use/result pairs together
- Protect system context and recent interactions

*Memory Optimization*
- Coordinate with memory management systems
- Track context utilization efficiency
- Monitor reduction effectiveness over time
- Optimize character-to-token estimation ratios

## Related Documentation

**architecture** - Context management architecture and design patterns
**patterns** - Long-running conversation patterns and strategies
**troubleshooting** - Context overflow debugging and resolution
**performance** - Context optimization and efficiency techniques
