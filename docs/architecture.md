# Stood Architecture Guide

This page provides detailed architectural insights into the Stood agent library, covering design decisions, component interactions, and system patterns.

## System Overview

Stood is designed as a *multi-provider agent framework* with four primary architectural principles:

1. *Multi-Provider Integration* - Support for AWS Bedrock, LM Studio, Anthropic, OpenAI, and other providers
2. *Library-First Design* - Embeddable in applications rather than standalone service  
3. *Type Safety Throughout* - Leverage Rust's type system to prevent runtime errors
4. *Intelligent Execution* - Automatic parallel/sequential tool execution with real-time monitoring

## Core Component Architecture

### Agent Layer
```
┌─────────────────────────────────────────────────────────┐
│                     Agent Layer                         │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │    Agent     │  │ AgentBuilder │  │ AgentConfig  │   │
│  │              │  │              │  │              │   │
│  │ - execute()  │  │ - model()    │  │ - temperature│   │
│  │              │  │ - temp()     │  │ - max_tokens │   │
│  │              │  │ - build()    │  │ - system_    │   │
│  │              │  │              │  │   prompt     │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Conversation Management
```
┌─────────────────────────────────────────────────────────┐
│                Conversation Layer                       │
├─────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────┐  │
│  │            ConversationManager                    │  │
│  │                                                   │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐│  │
│  │  │   Messages  │  │   System    │  │   Provider  ││  │
│  │  │             │  │   Prompt    │  │  Formatter  ││  │
│  │  │ - User      │  │             │  │             ││  │
│  │  │ - Assistant │  │ - Role      │  │ - JSON      ││  │
│  │  │ - Tool Use  │  │ - Context   │  │ - Validation││  │
│  │  │ - Tool Res  │  │             │  │             ││  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘│  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Tool System Architecture
```
┌─────────────────────────────────────────────────────────┐
│                    Tool System                          │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────┐   │
│  │                ToolRegistry                      │   │
│  │         Arc<RwLock<HashMap<String, Tool>>>       │   │
│  │                                                  │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │  │  Built-in   │  │   Custom    │  │     MCP     │ │
│  │  │    Tools    │  │    Tools    │  │    Tools    │ │
│  │  │             │  │             │  │             │ │
│  │  │ - File ops  │  │ - #[tool]   │  │ - External  │ │
│  │  │ - HTTP      │  │ - Manual    │  │ - Protocol  │ │
│  │  │ - Math      │  │   impl      │  │ - Servers   │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘ │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### MCP Integration Architecture {#mcp}
```
┌─────────────────────────────────────────────────────────┐
│               MCP Integration Layer                     │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────┐   │
│  │                 MCPClient                        │   │
│  │                                                  │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │  │  Session    │  │  Transport  │  │  Protocol   │ │
│  │  │  Manager    │  │   Layer     │  │   Handler   │ │
│  │  │             │  │             │  │             │ │
│  │  │ - Handshake │  │ - WebSocket │  │ - JSON-RPC  │ │
│  │  │ - Tools     │  │ - Stdio     │  │ - Message   │ │
│  │  │ - Timeout   │  │ - Lifecycle │  │   Routing   │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘ │
│  └──────────────────────────────────────────────────┘   │
│                           │                             │
│  ┌────────────────────────▼──────────────────────────┐   │
│  │              External MCP Servers                 │   │
│  │                                                   │   │
│  │ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐   │   │
│  │ │   Python    │ │   Node.js   │ │   Docker    │   │   │
│  │ │   Servers   │ │   Servers   │ │ Containers  │   │   │
│  │ └─────────────┘ └─────────────┘ └─────────────┘   │   │
│  └───────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```


## Data Flow Patterns

### Multi-Provider LLM Flow
```
User Input → Agent.execute() → ConversationManager.add_user_message()
    ↓
EventLoop.execute() → ProviderRegistry.get_provider()
    ↓
LlmProvider (Bedrock | LMStudio | Anthropic | OpenAI) → Model API Call
    ↓
Streaming Response → ConversationManager.add_assistant_message()
    ↓
AgentResult ← Agent.execute() ← User
```

### Agentic Execution Flow (5-Phase System)
```
User Input → Agent.execute() → EventLoop.new()
    ↓
Phase 1: Reasoning Phase → LlmProvider.chat() → Model analyzes current state
    ↓
Phase 2: Tool Selection Phase → Model selects tools for parallel execution
    ↓
Phase 3: Tool Execution Phase → ToolRegistry.execute_tools() → Parallel execution
    ↓
Phase 4: Reflection Phase → Model reflects on results and determines completion
    ↓
Phase 5: Response Generation Phase → Generate final response (if complete)
    ↓
(Optional) Task Evaluation Phase 
    ↓
Continue loop OR Return AgentResult ← Agent.execute() ← User
```

### Tool Registration and Execution
```
# [tool] macro → Tool impl → ToolRegistry.register_tool()
    ↓
Agent requests tools → ToolRegistry.get_tool_schemas()
    ↓
Model selects tool → ToolRegistry.execute_tool() → Tool.execute()
    ↓
ToolResult → Agent → Model → Final Response
```

### Task Evaluation Flow
```
Agent completes potential response → EvaluationStrategy::TaskEvaluation
    ↓
Agent evaluates: "Has the user's request been fully satisfied?"
    ↓
If satisfied: Return AgentResult with final response
    ↓
If not satisfied: Continue EventLoop with additional context
    ↓
Model identifies gaps → Tool selection → Tool execution → Re-evaluation
    ↓
Iterative improvement until user intent is fully addressed
```

### MCP Tool Discovery and Execution Flow
```
Agent.builder().with_mcp_client() → MCPClient.connect() → Transport.connect()
    ↓
MCP Handshake → Server Capabilities → Client.list_tools()
    ↓
Tool Schemas → ToolRegistry.register_mcp_tools() → Namespace prefixing
    ↓
Agent.execute() → Model requests tool → MCPAgentTool.execute()
    ↓
MCPClient.call_tool() → MCP Server → Tool Execution → JSON-RPC Response
    ↓
ToolResult → EventLoop → Model → AgentResult
```

## Concurrency Model

### Thread Safety
All core components are thread-safe and support concurrent access:

```rust
// Safe to share agents across threads
let agent = Arc::new(Mutex::new(agent));

// Safe to share registries across agents
let registry = Arc::new(ToolRegistry::new());

// Safe concurrent tool execution
let results = futures::future::join_all(
    tool_calls.iter().map(|call| registry.execute_tool(call))
).await;
```

### Async Patterns
- *Agent Operations*: All major operations are async
- *Tool Execution*: Async trait enables non-blocking I/O
- *LLM Providers*: Native async with provider-specific connection pooling
- *Event Loop*: Async orchestration of multi-step workflows with 5-phase execution
- *MCP Integration*: Async transport layers for WebSocket and stdio connections

## Error Handling Strategy

### Error Categories
1. *Input Validation* - Invalid parameters, malformed requests
2. *Configuration* - Missing credentials, invalid settings
3. *Model Errors* - API failures, quota limits, model unavailability
4. *Tool Errors* - Tool execution failures, timeout errors
5. *Conversation* - Context management, formatting errors

### Recovery Strategies
- *Provider Failover* - Switch between LLM providers when one becomes unavailable
- *Automatic Retry* - Exponential backoff with provider-specific retry configs
- *Graceful Degradation* - Continue with partial results when tools fail
- *Context Preservation* - Maintain conversation state across EventLoop failures
- *MCP Reconnection* - Automatic reconnection to MCP servers with transport recovery
- *Error Propagation* - Clear error messages with actionable guidance and provider context

