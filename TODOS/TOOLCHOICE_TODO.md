# Tool Choice Implementation Plan

## Overview

Tool choice functionality allows developers to control how LLMs select and use tools, providing fine-grained control over tool execution behavior. This feature is standard across major LLM providers but uses different naming conventions and parameter values.

## Current State Analysis

### ✅ **What's Already Implemented**
- `ToolChoice` enum in `src/types/tools.rs` with variants:
  - `Auto` - LLM decides whether to use tools (current default)
  - `Any` - LLM must use at least one tool  
  - `Tool { name: String }` - LLM must use specific named tool
- `ToolConfig::with_choice()` method for tool configuration
- Basic infrastructure in place

### ❌ **What's Missing**
- `ToolChoice::None` variant to prevent any tool use
- Agent builder API integration (`.tool_choice()` method)
- Provider-specific implementation across Bedrock, Anthropic, OpenAI, LM Studio
- Proper mapping between Stood's ToolChoice and provider-specific parameters
- Documentation and examples
- Test coverage for forced tool selection

## Provider Compatibility Matrix

| Provider | Auto | Force Any Tool | Force Specific Tool | Prevent Tools |
|----------|------|----------------|---------------------|---------------|
| **Claude/Anthropic** | `"auto"` | `"any"` | `{"type": "tool", "name": "tool_name"}` | `"none"` |
| **OpenAI/ChatGPT** | `"auto"` | `"required"` | `{"type": "function", "function": {"name": "func_name"}}` | `"none"` |
| **AWS Bedrock** | Default behavior | `toolChoice` param | `toolChoice` with specific tool | Not specified |
| **LM Studio** | Depends on model | Varies by implementation | Varies by implementation | Varies |

## Recommendation: Keep "Auto" as Default

**Rationale:**
- Maintains backward compatibility with existing code
- Provides most intuitive behavior for new users
- Matches industry standard defaults (Claude, OpenAI both default to "auto")
- Allows gradual adoption of forced tool selection for specific use cases

## Implementation Plan

### **Milestone 1: Complete Core Types** 
**Target: Week 1**

#### Tasks:
- [ ] Add `ToolChoice::None` variant to enum
- [ ] Update `ToolChoice` serialization for provider compatibility
- [ ] Add provider-specific mapping functions
- [ ] Create `ToolChoiceStrategy` trait for provider abstraction

```rust
// Target API:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolChoice {
    Auto,                    // LLM decides (default)
    Any,                     // Must use some tool  
    Tool { name: String },   // Must use specific tool
    None,                    // Prevent tool use
}
```

### **Milestone 2: Agent Builder Integration**
**Target: Week 2**

#### Tasks:
- [ ] Add `.tool_choice()` method to `AgentBuilder`
- [ ] Add `.force_tool_use()` convenience method
- [ ] Add `.prevent_tool_use()` convenience method
- [ ] Integrate tool choice with tool configuration pipeline
- [ ] Update builder validation logic

```rust
// Target API:
let agent = Agent::builder()
    .model(Bedrock::Claude35Haiku)
    .tools(my_tools)
    .tool_choice(ToolChoice::Any)           // Force some tool
    .build().await?;

// Convenience methods:
let agent = Agent::builder()
    .force_tool_use("calculator")           // Force specific tool
    .prevent_tool_use()                     // Block all tools
    .build().await?;
```

### **Milestone 3: Provider Implementation**
**Target: Week 3-4**

#### Tasks:
- [ ] **Bedrock Provider**: Map ToolChoice to AWS Bedrock `toolChoice` parameter
- [ ] **Anthropic Provider**: Map ToolChoice to Claude API `tool_choice` parameter  
- [ ] **OpenAI Provider**: Map ToolChoice to OpenAI `tool_choice` parameter
- [ ] **LM Studio Provider**: Implement best-effort mapping based on model capabilities
- [ ] Create provider-specific test suites

#### Provider Mapping Logic:
```rust
// Bedrock mapping:
ToolChoice::Auto -> No toolChoice parameter (default)
ToolChoice::Any -> toolChoice: {"auto": {}} or force first available
ToolChoice::Tool { name } -> toolChoice: {"tool": {"name": name}}
ToolChoice::None -> Exclude tools from request entirely

// Anthropic mapping:
ToolChoice::Auto -> tool_choice: "auto"
ToolChoice::Any -> tool_choice: "any" 
ToolChoice::Tool { name } -> tool_choice: {"type": "tool", "name": name}
ToolChoice::None -> tool_choice: "none"

// OpenAI mapping:
ToolChoice::Auto -> tool_choice: "auto"
ToolChoice::Any -> tool_choice: "required"
ToolChoice::Tool { name } -> tool_choice: {"type": "function", "function": {"name": name}}
ToolChoice::None -> tool_choice: "none"
```

### **Milestone 4: Advanced Features**
**Target: Week 5**

#### Tasks:
- [ ] **Conditional Tool Choice**: Based on conversation context
- [ ] **Tool Choice Policies**: Reusable configurations for common patterns
- [ ] **Tool Choice Analytics**: Track tool selection patterns
- [ ] **Token-Efficient Mode**: Integration with AWS Bedrock 2025 features

```rust
// Advanced API:
let agent = Agent::builder()
    .tool_choice_policy(ToolChoicePolicy::Development) // Force tools for testing
    .conditional_tool_choice(|context| {
        if context.is_debugging() { ToolChoice::Any } 
        else { ToolChoice::Auto }
    })
    .build().await?;
```

### **Milestone 5: Documentation & Testing**
**Target: Week 6**

#### Tasks:
- [ ] Create comprehensive examples for each tool choice mode
- [ ] Add integration tests across all providers
- [ ] Document provider-specific behavior differences
- [ ] Create migration guide for existing users
- [ ] Add tool choice examples to example suite

## Implementation Considerations

### **Cross-Provider Challenges**
1. **Feature Parity**: Not all providers support all tool choice modes
2. **Parameter Format**: Each provider uses different JSON structures
3. **Error Handling**: Different error responses for invalid tool choice
4. **Model Support**: Some models don't support forced tool selection

### **Backward Compatibility Strategy**
- Keep existing `ToolConfig::new()` behavior (Auto choice)
- Add new builder methods as opt-in features
- Provide clear migration path in documentation
- Maintain existing tool execution semantics

### **Testing Strategy**
- Unit tests for each ToolChoice variant
- Integration tests with real provider APIs
- Mock provider tests for edge cases
- Performance tests for tool selection overhead
- Cross-provider compatibility validation

## Risk Mitigation

### **High Risk Items**
1. **Provider API Changes**: AWS/Anthropic/OpenAI may change tool choice parameters
2. **Model Compatibility**: Not all models support forced tool selection
3. **Performance Impact**: Additional API complexity may slow requests

### **Mitigation Strategies**
- Abstract provider differences behind common interface
- Graceful degradation for unsupported models
- Comprehensive error handling and user feedback
- Feature flags for experimental functionality

## Success Metrics

- [ ] All four ToolChoice variants work across primary providers (Bedrock, Anthropic, OpenAI)
- [ ] 100% backward compatibility with existing tool usage
- [ ] Comprehensive test coverage (>95%)
- [ ] Complete documentation with examples
- [ ] Zero breaking changes to existing Agent API
- [ ] Performance impact <5% for tool selection overhead

## Future Enhancements

- **Multi-Tool Policies**: Complex rules for tool selection
- **Tool Choice Learning**: AI-driven optimization of tool selection
- **Provider-Specific Optimizations**: Leverage unique provider features
- **Tool Choice Analytics Dashboard**: Visual tool usage patterns
- **Dynamic Tool Choice**: Runtime adjustment based on context