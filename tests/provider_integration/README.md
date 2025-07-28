# Provider Integration Testing

## Test Commands

### Run All Tests
```bash
cargo run --bin verify
```
Run all tests across all providers and models

### Test by Suite
```bash
cargo run --bin verify -- core
```
Test core functionality (basic chat, multi-turn, health checks, agent builder)

```bash
cargo run --bin verify -- tools
```
Test tool integration (tool registry, builtin tools, custom tools, parallel execution)

```bash
cargo run --bin verify -- streaming
```
Test streaming functionality (basic streaming, streaming with tools)

```bash
cargo run --bin verify -- token_counting
```
Test token counting accuracy (streaming, non-streaming, with tools, consistency)

### Test by Provider
```bash
cargo run --bin verify -- --provider bedrock
```
Test AWS Bedrock provider only (Claude 3.5 Haiku, Amazon Nova Micro)

```bash
cargo run --bin verify -- --provider lm_studio
```
Test LM Studio provider only (Gemma 3 27B/12B, Tessa Rust 7B)

### Test by Model
```bash
cargo run --bin verify -- --model tessa-rust-t1-7b
```
Test tessa-rust-t1-7b model across all test suites

```bash
cargo run --bin verify -- --model google/gemma-3-27b
```
Test google/gemma-3-27b model across all test suites

```bash
cargo run --bin verify -- --model claude-3-5-haiku
```
Test Claude 3.5 Haiku model across all test suites

### Test Specific Functionality
```bash
cargo run --bin verify -- --test basic_chat
```
Test basic chat functionality across all providers

```bash
cargo run --bin verify -- --test builtin_calculator
```
Test calculator tool across all providers

```bash
cargo run --bin verify -- --test streaming_with_tools
```
Test streaming with tools across all providers

### Combined Filtering
```bash
cargo run --bin verify -- core --provider lm_studio
```
Test core functionality on LM Studio provider only

```bash
cargo run --bin verify -- tools --model tessa-rust-t1-7b
```
Test tools functionality with tessa model only

```bash
cargo run --bin verify -- --test builtin_calculator --provider lm_studio --model tessa-rust-t1-7b
```
Test calculator tool with specific provider and model

### Debug Mode
```bash
cargo run --bin verify -- --debug
```
Enable detailed debug output showing test generation and execution details

```bash
cargo run --bin verify -- core --provider lm_studio --model tessa-rust-t1-7b --debug
```
Debug mode with specific filters for detailed troubleshooting

## Environment Setup

**LM Studio**: Install LM Studio, load supported models, enable API server on `http://localhost:1234`

**AWS Bedrock**: Set AWS credentials and ensure Bedrock access in your AWS account