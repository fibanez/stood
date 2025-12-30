# Stood Agent Library - Documentation Index

Welcome to the Stood agent library documentation. This knowledge base provides comprehensive guidance for building AI agents with multi-provider LLM support.

## Quick Navigation

### Getting Started
- **architecture** - System design and component overview
- **patterns** - Common usage patterns and best practices
- **examples** - Code examples and tutorials
- **tools** - Tool development approaches and best practices
- **mcp** - Model Context Protocol integration guide
- **telemetry** - Logging, metrics, and observability
- **conversation_manager** - Tool-aware message history and conversation pruning
- **context_manager** - Token counting and proactive context window management

### Development
- **troubleshooting** - Common issues and solutions
- **performance** - Performance optimization guide

### Reference
- **antipatterns** - What NOT to do and why
- **migration** - Version migration guides

## Library Overview

Stood is a multi-provider agent framework that provides:

* *Multi-Provider Support* - AWS Bedrock, LM Studio, Anthropic, OpenAI, and more
* *Type-Safe Tools* - Compile-time validation of tool parameters
* *Agentic Execution* - Multi-step reasoning with automatic tool orchestration
* *Production Ready* - Comprehensive error handling and observability

## Architecture Summary

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│     Agent       │────│ ProviderRegistry │────│   LLM APIs      │
│                 │    │                  │    │  (Bedrock,      │
├─────────────────┤    │ • Bedrock        │    │  Anthropic,     │
│ Conversation    │    │ • LMStudio       │    │  OpenAI,        │
│ Manager         │    │ • Anthropic      │    │  LMStudio,...)  │
├─────────────────┤    │ • OpenAI         │    └─────────────────┘
│   EventLoop     │    └──────────────────┘
│ (5-Phase        │
│  Agentic)       │    ┌──────────────────┐
└─────────────────┘────│   ToolRegistry   │
                       │                  │
                       └──────────┬───────┘
                                  │
                                  ▼
                     ┌─────────────────────────┐
                     │     Tool Ecosystem      │
                     │ ┌─────────┐ ┌─────────┐ │
                     │ │Built-in │ │ Custom  │ │
                     │ │ Tools   │ │#[tool]  │ │
                     │ └─────────┘ └─────────┘ │
                     │ ┌─────────┐             │
                     │ │   MCP   │             │
                     │ │ Tools   │             │
                     │ └─────────┘             │
                     └─────────────────────────┘
```

## Key Components

### Agent Module
Core agent orchestration with multi-provider LLM support and 5-phase agentic execution:
- `Agent` - Main agent struct with single `execute()` method for all tasks
- `AgentBuilder` - Fluent configuration with provider selection and tool integration
- `ConversationManager` - Message history and tool-aware conversation pruning
- `ContextManager` - Token counting and proactive context window management
- `EventLoop` - 5-phase agentic execution (Reasoning, Tool Selection, Tool Execution, Reflection, Response Generation)
- `AgentResult` - Unified execution results with performance metrics and tool usage details

📚 [View Agent Module API Documentation](../src/agent/mod.rs)

### Tools Module  
Unified tool system with compile-time validation and parallel execution:
- `Tool` trait - Primary tool interface for built-in, custom, and MCP tools
- `ToolRegistry` - Thread-safe tool management with Arc-based sharing
- `ToolExecutor` - Parallel tool execution with intelligent strategy selection
- `#[tool]` macro - Automatic tool generation with schema validation
- `ToolResult` - Standardized execution results with success/error handling

📚 [View Tools Module API Documentation](../src/tools/mod.rs)

### LLM Module
Multi-provider LLM integration with unified interface and enterprise-grade reliability:
- `ProviderRegistry` - Central registry for all LLM provider configurations
- `LlmProvider` trait - Unified interface for Bedrock, LM Studio, Anthropic, OpenAI, Ollama
- `LlmModel` enum - Type-safe model selection (Claude 3.5, Nova, Gemma, GPT-4, etc.)
- `ChatConfig` - Provider-agnostic configuration with streaming and tool support
- `ProviderCapabilities` - Feature detection and compatibility checking

📚 [View LLM Module API Documentation](../src/llm/mod.rs)

### MCP Module
Model Context Protocol integration with simplified agent integration:
- `MCPClient` - Full-featured client with session management and error recovery
- `TransportFactory` - WebSocket and stdio transport with automatic lifecycle management
- `Agent::with_mcp_client()` - One-line integration matching Python's simplicity
- Tool namespace prefixing to prevent conflicts with multiple MCP servers
- Automatic tool discovery, schema validation, and execution

📚 [View MCP Module API Documentation](../src/mcp/mod.rs)


### Telemetry Module
Observability for AI agent performance monitoring:
- `TelemetryConfig` - Configuration for telemetry and tracing
- `EventLoopMetrics` - Agent performance and token usage tracking
- `LoggingConfig` and `PerformanceTracer` - File logging and timing
- GenAI semantic conventions for OpenTelemetry compatibility

📚 [View Telemetry Module API Documentation](../src/telemetry/mod.rs)

## Context Management: Understanding the Distinction

Two complementary systems work together to manage conversation size and quality:

### Context Manager
**Purpose**: Proactive token-level management and overflow prevention
- **Token Counting**: Estimates token usage with character-to-token ratios
- **Threshold Monitoring**: Warns when approaching model limits (90% of safe limit)  
- **Proactive Reduction**: Automatically trims messages before overflow occurs
- **Priority-Based**: Uses 5-tier priority system to preserve important messages
- **Model-Aware**: Configured for specific model token limits (Claude 3.5 Sonnet: 200k tokens)

📚 [Complete Context Manager Guide](context_manager.md)

### Conversation Manager  
**Purpose**: Tool-aware message history management and conversation coherence
- **Tool Sequence Preservation**: Never separates tool use from tool result messages
- **Sliding Window**: Maintains conversation within message count limits (default: 40 messages)
- **Safe Boundaries**: Finds appropriate points to trim without breaking tool interactions
- **Conversation Flow**: Maintains meaningful dialogue continuity
- **Dual Strategy**: Works with both message count and context manager token limits

📚 [Complete Conversation Manager Guide](conversation_manager.md)

### How They Work Together
1. **Context Manager** monitors token usage and identifies when reduction is needed
2. **Conversation Manager** performs the actual message trimming while preserving tool sequences
3. **Integration**: Context manager provides intelligence, conversation manager executes safely