# Examples - Code Examples and Tutorials

This page provides comprehensive walkthroughs of the Stood agent library examples, progressing from basic concepts to advanced integrations and production-ready implementations.

## Basic Examples (001-007)

### Example 001: Tool Macro
*Learn the fundamentals of creating custom tools with the `#[tool]` macro*

📖 [View Full Example](../examples/001_tool_macro.rs)

This example demonstrates the core tool creation system using Rust's procedural macro system for rapid tool development.

**Key Concepts Covered:**
- `#[tool]` macro usage and syntax
- Tool parameter validation
- Error handling in tools
- Basic agent integration

### Example 002: Tool Decorator Registry
*Master tool organization and registry patterns*

📖 [View Full Example](../examples/002_tool_decorator_registry.rs)

This example showcases advanced tool management using decorator patterns and registry systems.

**Key Concepts Covered:**
- Tool decorator patterns
- Registry-based tool management
- Tool lifecycle management
- Metadata and documentation

### Example 003: Interactive Chat Simple
*Build basic conversational interfaces*

📖 [View Full Example](../examples/003_interactive_chat_simple.rs)

This example demonstrates creating simple interactive chat interfaces with the agent system.

**Key Concepts Covered:**
- Interactive REPL implementation
- Basic conversation flow
- User input handling
- Agent response formatting

### Example 004: Streaming Simple
*Implement real-time streaming responses*

📖 [View Full Example](../examples/004_streaming_simple.rs)

This example shows how to implement streaming responses for real-time user experiences.

**Key Concepts Covered:**
- Streaming response patterns
- Real-time output handling
- Async response processing
- User experience optimization

### Example 005: Callbacks Basic
*Handle events and callbacks in agent execution*

📖 [View Full Example](../examples/005_callbacks_basic.rs)

This example demonstrates basic callback patterns for monitoring and responding to agent events.

**Key Concepts Covered:**
- Callback registration and handling
- Event-driven programming
- Agent lifecycle monitoring
- Custom event responses

### Example 006: Callback System Demo
*Advanced callback system integration*

📖 [View Full Example](../examples/006_callback_system_demo.rs)

This example showcases comprehensive callback system integration with advanced patterns.

**Key Concepts Covered:**
- Advanced callback patterns
- System integration
- Performance monitoring
- Error handling in callbacks

### Example 007: Debug Logging
*Configure logging and debugging for development*

📖 [View Full Example](../examples/007_debug_logging.rs)

This example demonstrates logging configuration and debugging techniques for agent development.

**Key Concepts Covered:**
- Logging configuration
- Debug output formatting
- Troubleshooting techniques
- Development best practices

## Intermediate Examples (008-015)

### Example 008: Streaming Custom Callbacks
*Advanced streaming with custom callback handlers*

📖 [View Full Example](../examples/008_streaming_custom_callbacks.rs)

This example demonstrates advanced streaming patterns with custom callback handling for complex real-time applications.

**Key Concepts Covered:**
- Custom streaming callbacks
- Advanced event handling
- Real-time processing patterns
- Performance optimization

### Example 009: Logging Demo
*Comprehensive logging setup and configuration*

📖 [View Full Example](../examples/009_logging_demo.rs)

This example showcases production-ready logging configuration and patterns for agent applications.

**Key Concepts Covered:**
- Production logging setup
- Log level configuration
- Structured logging patterns
- Log aggregation

### Example 010: Streaming with Tools
*Combine streaming responses with tool execution*

📖 [View Full Example](../examples/010_streaming_with_tools.rs)

This example demonstrates combining streaming responses with tool execution for complex workflows.

**Key Concepts Covered:**
- Tool integration with streaming
- Complex workflow patterns
- Performance optimization
- User experience design

### Example 011: Basic Agent
*Multi-provider agent configuration*

📖 [View Full Example](../examples/011_basic_agent.rs)

This example shows basic agent setup with support for multiple model providers.

**Key Concepts Covered:**
- Multi-provider configuration
- Agent builder patterns
- Model selection
- Basic configuration

### Example 012: Batching Optimization Demo
*Performance optimization through batching techniques*

📖 [View Full Example](../examples/012_batching_optimization_demo.rs)

This example demonstrates I/O performance optimization through batching techniques that reduce expensive operations.

**Key Concepts Covered:**
- Batching optimization patterns
- I/O performance improvement
- Resource management
- Efficiency techniques

### Example 013: MCP Integration
*Simple MCP server integration*

📖 [View Full Example](../examples/013_mcp_integration.rs)

This example demonstrates Model Context Protocol integration with automatic tool discovery and validation.

**Key Concepts Covered:**
- MCP client setup and configuration
- External tool discovery and registration
- Tool result validation
- Simple integration patterns

### Example 014: MCP Configuration Examples
*Advanced MCP configuration patterns*

📖 [View Full Example](../examples/014_mcp_configuration_examples.rs)

This example showcases various MCP configuration patterns for different deployment scenarios.

**Key Concepts Covered:**
- Multiple MCP server configurations
- Environment-based configuration
- Network configuration patterns
- Error handling and fallback strategies

### Example 015: Authorization Chat Wrapper
*Security and authorization patterns*

📖 [View Full Example](../examples/015_authorization_chat_wrapper.rs)

This example demonstrates authorization patterns and security controls for chat applications.

**Key Concepts Covered:**
- Authorization mechanisms
- Security patterns
- Access control
- Tool approval workflows

## Advanced Examples (016-021)

### Example 016: Context Management
*Context window management and optimization*

📖 [View Full Example](../examples/016_context_management.rs)

This example demonstrates advanced context management techniques for handling large conversations.

**Key Concepts Covered:**
- Context window optimization
- Memory management
- Conversation summarization
- Performance considerations

### Example 017: Parallel Execution
*Concurrency and parallel processing patterns*

📖 [View Full Example](../examples/017_parallel_execution.rs)

This example showcases parallel tool execution and concurrency patterns for performance optimization.

**Key Concepts Covered:**
- Parallel tool execution
- Concurrency management
- Performance optimization
- Resource coordination

### Example 018: Task Evaluation
*Default multi-cycle behavior with task evaluation*

📖 [View Full Example](../examples/018_task_evaluation.rs)

This example demonstrates the default task evaluation strategy that enables multi-cycle agent execution based on user intent satisfaction.

**Key Concepts Covered:**
- Task evaluation strategy (default behavior)
- User intent satisfaction assessment
- Multi-cycle autonomous execution
- Quality-driven continuation decisions

### Example 019: Agent-Based Evaluation
*Separate evaluator agents for quality assessment*

📖 [View Full Example](../examples/019_agent_based_evaluation.rs)

This example shows how to use separate evaluator agents for independent quality assessment and continuation decisions.

**Key Concepts Covered:**
- Multi-agent evaluation architecture
- Independent quality assessment
- Evaluator agent configuration
- Specialized evaluation logic

### Example 020: Multi-Perspective Evaluation
*Weighted multi-perspective evaluation system*

📖 [View Full Example](../examples/020_multi_perspective.rs)

This example showcases multi-perspective evaluation with weighted scoring from multiple viewpoints for comprehensive assessment.

**Key Concepts Covered:**
- Multi-perspective analysis
- Weighted evaluation scoring
- Comprehensive quality assessment
- Perspective configuration and management

### Example 021: Agentic Chat
*Complete interactive chat system*

📖 [View Full Example](../examples/021_agentic_chat.rs)

This example demonstrates a full interactive chat application with LLM-driven tool selection and autonomous multi-cycle operation.

**Key Concepts Covered:**
- Complete interactive chat system
- Autonomous multi-cycle operation
- LLM-driven tool orchestration
- Real-time user interaction patterns

## Expert Examples (022-023)

### Example 022: AWS Documentation MCP
*Production MCP integration with Docker deployment*

📖 [View Full Example](../examples/022_aws_doc_mcp/)

This example demonstrates real-world MCP integration using Docker-based AWS Documentation MCP server with enterprise deployment patterns.

**Key Concepts Covered:**
- Production MCP integration
- Docker-based MCP servers
- Real-world external tool usage
- Enterprise deployment patterns

### Example 023: Telemetry
*Production observability with OpenTelemetry stack*

📖 [View Full Example](../examples/023_telemetry/)

This comprehensive example showcases production-grade observability with complete OpenTelemetry, Prometheus, and Grafana integration.

**Key Concepts Covered:**
- Complete observability stack
- Production telemetry patterns
- Real-time performance monitoring
- Enterprise observability best practices

## Running the Examples

### Prerequisites
All examples require:
- Rust 1.70+ with Cargo
- AWS credentials configured for Bedrock access
- Internet connectivity for AWS API calls

### Basic Examples (001-011)
```bash
# From the project root directory
cargo run --example 001_tool_macro
cargo run --example 011_basic_agent
cargo run --example 013_mcp_integration
```

### Telemetry Demo (023)
```bash
# Navigate to telemetry demo
cd examples/023_telemetry

# Quick start (automated setup)
./demo-overview.sh setup
./run-demo.sh

# Or step by step
./setup-telemetry.sh        # Deploy monitoring stack
./troubleshoot.sh diagnose  # Verify system health
./run-demo.sh               # Run telemetry demo
```


