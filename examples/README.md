# Stood Examples

Requirements: AWS credentials configured (environment variables, AWS profile, or IAM role) with Bedrock access.

Examples are organized by complexity level, from basic concepts to advanced systems.

## Basic Concepts (001-007)

### 001_tool_macro ✅
Basic tool creation using the #[tool] macro.
```bash
cargo run --example 001_tool_macro
```

### 002_tool_decorator_registry ✅
Tool decorator pattern with registry system.
```bash
cargo run --example 002_tool_decorator_registry
```

### 003_interactive_chat_simple ✅
Safe interactive chat REPL that works reliably.
```bash
cargo run --example 003_interactive_chat_simple
```

### 004_streaming_simple ✅
Simple streaming response handling.
```bash
cargo run --example 004_streaming_simple
```

### 005_callbacks_basic ✅
Basic callback patterns for event handling.
```bash
cargo run --example 005_callbacks_basic
```

### 006_callback_system_demo ✅
Comprehensive callback system with aligned streaming output.
```bash
cargo run --example 006_callback_system_demo
```

### 007_debug_logging ✅
Debug logging for detailed conversation analysis.
```bash
cargo run --example 007_debug_logging
```

## Intermediate Concepts (008-015)

### 008_streaming_custom_callbacks ✅
Custom callbacks for streaming responses.
```bash
cargo run --example 008_streaming_custom_callbacks
```

### 009_logging_demo ✅
Comprehensive logging setup and configuration patterns.
```bash
cargo run --example 009_logging_demo
```

### 010_streaming_with_tools ✅
Streaming responses with tool integration.
```bash
cargo run --example 010_streaming_with_tools
```

### 011_basic_agent ✅
Basic agent setup with multiple provider support.
```bash
cargo run --example 011_basic_agent
```

### 012_batching_optimization_demo ✅
I/O performance optimization through batching techniques that reduce expensive file system calls.
```bash
cargo run --example 012_batching_optimization_demo
```

### 013_mcp_integration ✅
Simple MCP server integration with agent.
```bash
cargo run --example 013_mcp_integration
```

### 014_mcp_configuration_examples ✅
Various MCP configuration patterns.
```bash
cargo run --example 014_mcp_configuration_examples
```

### 015_authorization_chat_wrapper ✅
Authorization patterns for chat applications with tool approval callbacks.
```bash
cargo run --example 015_authorization_chat_wrapper
```

## Advanced Concepts (016-024)

### 016_context_management ✅
Context window management and reduction techniques.
```bash
cargo run --example 016_context_management
```

### 017_parallel_execution ✅
Parallel tool execution patterns demonstrating max_parallel_tools configuration.
```bash
cargo run --example 017_parallel_execution
```

### 018_task_evaluation ✅
Task evaluation strategy for autonomous multi-cycle execution.
```bash
cargo run --example 018_task_evaluation
```

### 019_agent_based_evaluation ✅
Agent-based evaluation using a separate evaluator agent for quality assessment.
```bash
cargo run --example 019_agent_based_evaluation
```

### 020_multi_perspective ✅
Multi-perspective evaluation with weighted scoring from multiple viewpoints.
```bash
cargo run --example 020_multi_perspective
```

### 021_agentic_chat ✅
Full interactive chat application with LLM-driven tool selection.
```bash
cargo run --example 021_agentic_chat
```

## Specialized Modules (022-024)

### 022_aws_doc_mcp ✅
AWS documentation access via MCP.
```bash
cd examples/022_aws_doc_mcp && cargo run
```

### 023_telemetry ⚠️
Telemetry examples (some require full OTEL implementation).
```bash
cd examples/023_telemetry
cargo run --example simple_telemetry_test
cargo run --example smart_telemetry_test
cargo run --example metrics_test
```

### 024_enterprise_prompt_builder ✅
Enterprise prompt builder patterns.
```bash
cargo run --example 024_enterprise_prompt_builder
```

## Complexity Overview

- **Basic (001-007)**: Tool creation, simple streaming, basic agent patterns
- **Intermediate (008-015)**: Callbacks, MCP integration, logging, authorization
- **Advanced (016-021)**: Evaluation strategies, performance optimization, context management
- **Specialized (022-024)**: MCP integrations, telemetry, enterprise patterns