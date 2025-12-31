# Stood Agent Library - Documentation Index

Welcome to the Stood agent library documentation. This knowledge base provides comprehensive guidance for building AI agents with AWS Bedrock integration.

## Quick Navigation

### Getting Started
- [API Documentation](api.md) - Core API reference for Agent and tools
- [Architecture](architecture.md) - System design and component overview
- [Examples](examples.md) - Code examples and tutorials
- [Tools](tools.md) - Tool development approaches and best practices
- [Tool Middleware](tools-middleware.md) - Intercept and modify tool execution
- [MCP](mcp.md) - Model Context Protocol integration guide

### Advanced Features
- [Telemetry](telemetry.md) - CloudWatch Gen AI Observability integration
- [Streaming](streaming.md) - Real-time response handling and patterns
- [Callbacks](callbacks.md) - Real-time monitoring and event handling system
- [Conversation Manager](conversation_manager.md) - Message history and context management
- [Context Manager](context_manager.md) - Context window optimization

### Reference
- [CloudWatch GenAI Implementation Guide](CLOUDWATCH_GENAI_IMPLEMENTATION_GUIDE.md) - Detailed CloudWatch setup
- [CloudWatch GenAI Observability Knowledge](CLOUDWATCH_GENAI_OBSERVABILITY_KNOWLEDGE.md) - Background knowledge

## Library Overview

Stood is an AWS Bedrock-focused agent framework that provides:

* *Native AWS Integration* - Optimized for Claude 4.5 and Nova models
* *Type-Safe Tools* - Compile-time validation of tool parameters
* *Agentic Execution* - Multi-step reasoning with automatic tool orchestration
* *Tool Middleware* - Intercept and control tool execution
* *Production Ready* - Comprehensive error handling and CloudWatch observability

## Architecture Summary

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│     Agent       │────│  BedrockClient   │────│  AWS Bedrock    │
│                 │    │                  │    │     API         │
├─────────────────┤    └──────────────────┘    └─────────────────┘
│ Conversation    │
│ Manager         │    ┌──────────────────┐
├─────────────────┤────│   ToolRegistry   │
│   EventLoop     │    │                  │
│ (Agentic)       │    └──────────────────┘
└─────────────────┘              │
                                 ▼
                     ┌─────────────────────────┐
                     │     Tool Ecosystem      │
                     │ ┌─────────┐ ┌─────────┐ │
                     │ │Built-in │ │ Custom  │ │
                     │ │ Tools   │ │ Tools   │ │
                     │ └─────────┘ └─────────┘ │
                     │ ┌─────────┐ ┌─────────┐ │
                     │ │   MCP   │ │Middleware│ │
                     │ │ Tools   │ │         │ │
                     │ └─────────┘ └─────────┘ │
                     └─────────────────────────┘
```

## Key Components

### Agent Module
Core agent implementation with conversation management:
- `Agent` - Main agent struct with Bedrock integration
- `AgentBuilder` - Fluent configuration interface
- `ConversationManager` - Message history and context
- `EventLoop` - Agentic execution orchestration

📚 [View Agent Module API Documentation](../src/agent/mod.rs)

### Tools Module
Unified tool system with compile-time validation:
- `Tool` trait - Primary tool interface
- `ToolRegistry` - Tool management and execution
- `#[tool]` macro - Automatic tool generation
- `ToolMiddleware` - Intercept tool execution
- MCP integration for external tools

📚 [View Tools Module API Documentation](../src/tools/mod.rs)

### Bedrock Provider
AWS Bedrock client with enterprise features:
- Native Claude 4.5 and Nova model support
- Model aliases for automatic upgrades (SonnetLatest, HaikuLatest, OpusLatest)
- Streaming responses and token management
- Comprehensive retry logic and error handling
- Performance optimization and connection pooling

📚 [View Bedrock Provider Documentation](../src/llm/providers/bedrock.rs)

### MCP Module
Model Context Protocol integration for external tools:
- MCP client and server implementations
- WebSocket and stdio transport support
- Automatic tool discovery and execution
- Comprehensive error handling and recovery

📚 [View MCP Module API Documentation](../src/mcp/mod.rs)

### Performance Module
Production-grade performance optimization for high-throughput deployments:
- AWS Bedrock connection pooling for 60-80% performance improvement
- Request batching and adaptive concurrency control
- Intelligent memory optimization and resource management
- Comprehensive metrics collection and monitoring

📚 [View Performance Module API Documentation](../src/performance/mod.rs)

### Telemetry Module
CloudWatch Gen AI Observability for AI agent monitoring:
- CloudWatch integration with OpenTelemetry semantic conventions
- Smart truncation for large prompts/responses (auto-handles 1MB limit)
- Batch splitting for high-volume exports
- File logging with `LoggingConfig` and `PerformanceTracer`
- Metrics collection with `EventLoopMetrics`, `CycleMetrics`, `TokenUsage`

📚 [View Telemetry Module API Documentation](../src/telemetry/mod.rs)

## Recent Changes

### Version 0.1.1

- **CloudWatch Gen AI Observability** - Production-ready integration with AWS CloudWatch GenAI dashboards
- **Smart Truncation** - Automatic handling of large prompts/responses to stay within CloudWatch limits
- **Model Aliases** - `SonnetLatest`, `HaikuLatest`, `OpusLatest` for automatic model upgrades
- **Nova 2 Models** - Support for NovaPremier, Nova2Lite, Nova2Pro with 1M context windows
- **Tool Middleware** - Intercept and control tool execution with `ToolMiddleware` trait
- **Cancellation Support** - External cancellation for long-running operations
- **AWS Credentials** - Programmatic credential configuration via `with_credentials()`
