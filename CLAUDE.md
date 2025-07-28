# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the Stood agent library - a Rust implementation of an AI agent framework with multi providers support. The project consists of:

1. **Rust Library** (`src/`) - The main Stood agent library implementation
1. **Provider Integration** (`src/tests/provider_integration`) - Test Suite to verify LLM provider functionality - use it to verify new provider functionality
1. **Documenation** (`src/docs`) - Project documetnation - developer driven
1. **Examples** (`src/examles`) - Fully functional examples to be used by developers and code assistant to quickly understand library functionality
1. **rewls** (`src/rewls`) - project rules for maintaining architectural and coding practices - executed via an MCP to verify rule violations before committing to git
1. **Stood Macros** (`src/stood-macros`) - macros that are part of stood, but have to be maintained in their folder

## Commands

### Rust Development
```bash
# Build the library
cargo build

# Run tests (includes integration tests that make real AWS Bedrock API calls)
cargo test

# Check for compilation errors without building
cargo check

# Format code
cargo fmt

# Run clippy linter
cargo clippy

# Build documentation
cargo doc --open

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Testing

### Integration Tests
Integration tests run automatically with `cargo test` and make real provider API calls. They require:
- Valid credentials (verified with test above)
- Internet connectivity

**IMPORTANT: Do not modify integration tests to require environment variables or flags. They should run by default with `cargo test`.**

## Architecture

### High-Level Design
The Stood library is designed with three core components:

1. **Provider Client** - Direct provider integration for Claude/Nova and eventually many other models
2. **Tools Component** - Rust function integration with compile-time validation
3. **Agent Component** - Orchestrates the agentic loop between Bedrock and Tools
4. **MCP Component** - MCP support
5. **Telemetry Component** - Telemetr support using OpenTelemetry

### Key Design Principles
- **Library-First**: Designed to be embedded in other Rust applications  
- **Performance Optimized**: Leverages Rust's zero-cost abstractions
- **Type Safety**: Strong typing throughout to prevent runtime errors
- **Model Compatibility**: Defaults to Claude Haiku 3 for broad AWS account compatibility


### Live Examples
Review examples for working versions of the API 
