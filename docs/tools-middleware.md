# Tool Middleware

Intercept and modify tool execution with the middleware system.

## Overview

Tool middleware allows you to intercept tool calls before and after execution. Use this for:

- **Approval workflows** - Require user confirmation before executing sensitive operations
- **Logging and auditing** - Track all tool executions for compliance
- **Parameter validation** - Enforce additional constraints on tool inputs
- **Rate limiting** - Control how often certain tools can be called
- **Result modification** - Transform or enhance tool outputs

## Architecture

```
Tool Request → before_tool() → Tool Execution → after_tool() → Result
                    ↓                                  ↓
             Can: Modify params           Can: Modify result
                  Abort/Skip                   Inject context
```

## Quick Start

```rust
use stood::agent::Agent;
use stood::tools::middleware::{ToolMiddleware, ToolMiddlewareAction, AfterToolAction, ToolContext};
use stood::tools::ToolResult;
use stood::llm::models::Bedrock;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug)]
struct LoggingMiddleware;

#[async_trait]
impl ToolMiddleware for LoggingMiddleware {
    async fn before_tool(
        &self,
        tool_name: &str,
        params: &Value,
        ctx: &ToolContext,
    ) -> ToolMiddlewareAction {
        println!("[{}] Executing tool: {}", ctx.agent_id, tool_name);
        ToolMiddlewareAction::Continue
    }

    async fn after_tool(
        &self,
        tool_name: &str,
        result: &ToolResult,
        ctx: &ToolContext,
    ) -> AfterToolAction {
        println!("[{}] Tool {} completed: success={}",
                 ctx.agent_id, tool_name, result.success);
        AfterToolAction::PassThrough
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut agent = Agent::builder()
        .model(Bedrock::ClaudeHaiku45)
        .with_builtin_tools()
        .with_middleware(Arc::new(LoggingMiddleware))
        .build()
        .await?;

    agent.execute("What time is it?").await?;
    Ok(())
}
```

## ToolMiddleware Trait

```rust
#[async_trait]
pub trait ToolMiddleware: Send + Sync + std::fmt::Debug {
    /// Called before tool execution
    async fn before_tool(
        &self,
        tool_name: &str,
        params: &Value,
        ctx: &ToolContext,
    ) -> ToolMiddlewareAction;

    /// Called after tool execution
    async fn after_tool(
        &self,
        tool_name: &str,
        result: &ToolResult,
        ctx: &ToolContext,
    ) -> AfterToolAction;
}
```

## Before Tool Actions

The `before_tool` method returns a `ToolMiddlewareAction`:

| Action | Description |
|--------|-------------|
| `Continue` | Proceed with original parameters |
| `ModifyParams(Value)` | Continue with modified parameters |
| `Abort(ToolResult)` | Stop execution with synthetic result |
| `Skip` | Skip this tool call entirely |

### Modifying Parameters

```rust
async fn before_tool(
    &self,
    tool_name: &str,
    params: &Value,
    ctx: &ToolContext,
) -> ToolMiddlewareAction {
    if tool_name == "file_write" {
        // Add a prefix to all file paths
        let mut modified = params.clone();
        if let Some(path) = modified.get_mut("path") {
            let new_path = format!("/sandbox/{}", path.as_str().unwrap_or(""));
            *path = Value::String(new_path);
        }
        return ToolMiddlewareAction::ModifyParams(modified);
    }
    ToolMiddlewareAction::Continue
}
```

### Aborting Execution

```rust
async fn before_tool(
    &self,
    tool_name: &str,
    params: &Value,
    ctx: &ToolContext,
) -> ToolMiddlewareAction {
    // Block dangerous operations
    if tool_name == "shell_exec" {
        return ToolMiddlewareAction::Abort(
            ToolResult::error("Shell execution is disabled for security")
        );
    }
    ToolMiddlewareAction::Continue
}
```

## After Tool Actions

The `after_tool` method returns an `AfterToolAction`:

| Action | Description |
|--------|-------------|
| `PassThrough` | Return original result unchanged |
| `ModifyResult(ToolResult)` | Return modified result |
| `InjectContext(String)` | Add context message after result |

### Modifying Results

```rust
async fn after_tool(
    &self,
    tool_name: &str,
    result: &ToolResult,
    ctx: &ToolContext,
) -> AfterToolAction {
    // Redact sensitive information
    if tool_name == "database_query" {
        let mut modified = result.clone();
        modified.output = redact_pii(&result.output);
        return AfterToolAction::ModifyResult(modified);
    }
    AfterToolAction::PassThrough
}
```

### Injecting Context

```rust
async fn after_tool(
    &self,
    tool_name: &str,
    result: &ToolResult,
    ctx: &ToolContext,
) -> AfterToolAction {
    // Add usage hints after certain tools
    if tool_name == "file_read" && result.success {
        return AfterToolAction::InjectContext(
            "Note: File content loaded. You can now analyze or process it.".to_string()
        );
    }
    AfterToolAction::PassThrough
}
```

## ToolContext

The `ToolContext` provides information about the current execution environment:

```rust
pub struct ToolContext {
    /// Unique identifier for the agent
    pub agent_id: String,
    /// Name of the agent (if set)
    pub agent_name: Option<String>,
    /// Type of agent
    pub agent_type: String,
    /// When this tool execution started
    pub execution_start: Instant,
    /// Number of tools executed in this turn
    pub tool_count_this_turn: usize,
    /// Total conversation message count
    pub message_count: usize,
}
```

## Use Cases

### Approval Middleware

Require user approval for certain operations:

```rust
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
        let needs_approval = matches!(
            tool_name,
            "file_write" | "file_delete" | "http_request"
        );

        if needs_approval {
            println!("Tool '{}' requires approval.", tool_name);
            println!("Parameters: {}", serde_json::to_string_pretty(params).unwrap());
            print!("Approve? [y/n]: ");

            // Read user input...
            let approved = read_user_input() == "y";

            if !approved {
                return ToolMiddlewareAction::Abort(
                    ToolResult::error("User denied tool execution")
                );
            }
        }
        ToolMiddlewareAction::Continue
    }

    async fn after_tool(
        &self,
        _tool_name: &str,
        _result: &ToolResult,
        _ctx: &ToolContext,
    ) -> AfterToolAction {
        AfterToolAction::PassThrough
    }
}
```

### Rate Limiting Middleware

Limit how often tools can be called:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
struct RateLimitMiddleware {
    call_count: AtomicUsize,
    max_calls: usize,
}

#[async_trait]
impl ToolMiddleware for RateLimitMiddleware {
    async fn before_tool(
        &self,
        tool_name: &str,
        params: &Value,
        ctx: &ToolContext,
    ) -> ToolMiddlewareAction {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);

        if count >= self.max_calls {
            return ToolMiddlewareAction::Abort(
                ToolResult::error(&format!(
                    "Rate limit exceeded: {} calls maximum",
                    self.max_calls
                ))
            );
        }
        ToolMiddlewareAction::Continue
    }

    async fn after_tool(
        &self,
        _tool_name: &str,
        _result: &ToolResult,
        _ctx: &ToolContext,
    ) -> AfterToolAction {
        AfterToolAction::PassThrough
    }
}
```

### Audit Logging Middleware

Log all tool executions for compliance:

```rust
use chrono::Utc;

#[derive(Debug)]
struct AuditMiddleware {
    log_file: std::sync::Mutex<std::fs::File>,
}

#[async_trait]
impl ToolMiddleware for AuditMiddleware {
    async fn before_tool(
        &self,
        tool_name: &str,
        params: &Value,
        ctx: &ToolContext,
    ) -> ToolMiddlewareAction {
        let entry = serde_json::json!({
            "timestamp": Utc::now().to_rfc3339(),
            "agent_id": ctx.agent_id,
            "tool": tool_name,
            "params": params,
            "event": "before_execution"
        });

        if let Ok(mut file) = self.log_file.lock() {
            writeln!(file, "{}", entry).ok();
        }

        ToolMiddlewareAction::Continue
    }

    async fn after_tool(
        &self,
        tool_name: &str,
        result: &ToolResult,
        ctx: &ToolContext,
    ) -> AfterToolAction {
        let entry = serde_json::json!({
            "timestamp": Utc::now().to_rfc3339(),
            "agent_id": ctx.agent_id,
            "tool": tool_name,
            "success": result.success,
            "event": "after_execution"
        });

        if let Ok(mut file) = self.log_file.lock() {
            writeln!(file, "{}", entry).ok();
        }

        AfterToolAction::PassThrough
    }
}
```

## Examples

📖 **Example:** [027_tool_approval_middleware.rs](../examples/027_tool_approval_middleware.rs) - Interactive tool approval with user confirmation

## Related Documentation

- [Tools](tools.md) - Tool development guide
- [API](api.md) - AgentBuilder middleware configuration
- [Source Code](../src/tools/middleware.rs) - Middleware implementation
