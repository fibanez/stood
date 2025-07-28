# Dynamic Model Registration Implementation Plan

## **Problem Statement**

The current Stood agent library requires hardcoding every model in `src/agent/mod.rs:402` via the `create_model_from_config()` function. This approach doesn't scale as:

- AWS Bedrock supports 100+ models  
- OpenRouter supports 400+ models
- New models require source code updates and recompilation

**Goal**: Implement runtime model registration using string-based identification patterns that support multi-provider access with robust fallback mechanisms.

## **Solution Architecture**

### **1. String-Based Model Identification Pattern**

**Format**: `provider:model_family/model_name/context_window/capabilities`

```text
Examples:
✅ "bedrock:anthropic/claude-3.5-sonnet/200k/text+vision+tools+thinking"
✅ "anthropic:claude-3.5-sonnet/200k/text+vision+tools+thinking"  
✅ "openrouter:anthropic/claude-3.5-sonnet/200k/text+vision+tools"
✅ "bedrock:amazon/nova-pro/300k/text+vision+tools"
✅ "lmstudio:google/gemma-3-27b/8k/text+tools"

Fallback patterns (capabilities optional):
✅ "bedrock:anthropic/claude-3.5-sonnet/200k" → assumes "text" capability
✅ "bedrock:anthropic/claude-3.5-sonnet" → infers context from discovery
```

**Key Benefits**:
- **Multi-Provider Access**: Same model accessible through different providers
- **Provider Differentiation**: `bedrock:anthropic/claude` vs `anthropic:claude` vs `openrouter:anthropic/claude`
- **Robust Fallbacks**: Missing capabilities default to text model

### **2. Core Components**

#### **A. ModelIdentifier Parsing**

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ModelIdentifier {
    pub provider: String,           // "bedrock", "anthropic", "openrouter"
    pub model_family: String,       // "anthropic", "amazon", "google"  
    pub model_name: String,         // "claude-3.5-sonnet", "nova-pro"
    pub context_window: Option<String>, // "200k", "1m", None
    pub capabilities: Vec<String>,  // ["text", "vision", "tools", "thinking"]
}

impl ModelIdentifier {
    pub fn parse(identifier: &str) -> Result<Self, ParseError> {
        // Parse "bedrock:anthropic/claude-3.5-sonnet/200k/text+vision+tools"
        // Supports 2-4 components with fallbacks to text capability
    }
}
```

#### **B. DynamicModel Runtime Struct**

```rust
#[derive(Debug, Clone)]
pub struct DynamicModel {
    pub identifier: String,        // Original string identifier
    pub api_model_id: String,      // Provider-specific API model ID
    pub provider: ProviderType,
    pub display_name: String,
    pub context_window: usize,
    pub max_output_tokens: usize,
    pub capabilities: ModelCapabilities,
    pub default_temperature: f32,
    pub family: ModelFamily,
}

impl LlmModel for DynamicModel {
    fn model_id(&self) -> &str { &self.api_model_id }
    fn provider(&self) -> ProviderType { self.provider }
    // ... implement all LlmModel methods using stored metadata
}
```

#### **C. ModelRegistry System**

```rust
pub struct ModelRegistry {
    dynamic_models: RwLock<HashMap<String, DynamicModel>>,
    provider_discovery: HashMap<ProviderType, Arc<dyn ModelDiscovery>>,
}

pub static MODEL_REGISTRY: Lazy<ModelRegistry> = Lazy::new(|| {
    ModelRegistry::new()
});

impl ModelRegistry {
    pub async fn resolve_model(&self, identifier: &str) -> Result<Box<dyn LlmModel>, ModelError> {
        // 1. Try exact match in registry cache
        // 2. Parse identifier with fallbacks  
        // 3. Get provider and attempt discovery
        // 4. Try provider-specific resolution
        // 5. Auto-discovery fallback
        // 6. Error with suggested alternatives
    }
}
```

### **3. Enhanced LlmProvider Trait**

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    // Existing methods...
    
    /// Discover available models for this provider
    async fn discover_models(&self) -> Result<Vec<ModelMetadata>, LlmError>;
    
    /// Resolve a model identifier to actual API model ID  
    fn resolve_model_id(&self, model_identifier: &ModelIdentifier) -> Option<String>;
    
    /// Get model capabilities without API call
    fn get_model_capabilities(&self, model_id: &str) -> Option<ModelCapabilities>;
}

#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub identifier: String,
    pub api_id: String,
    pub display_name: String,
    pub context_window: usize,
    pub max_output_tokens: usize,
    pub capabilities: ModelCapabilities,
    pub family: ModelFamily,
}
```

### **4. Provider-Specific Implementation Examples**

#### **Bedrock Provider Model Translation**

```rust
impl BedrockProvider {
    fn resolve_model_id(&self, identifier: &ModelIdentifier) -> Option<String> {
        match (&identifier.model_family[..], &identifier.model_name[..]) {
            ("anthropic", "claude-3.5-sonnet") => {
                Some("us.anthropic.claude-3-5-sonnet-20241022-v2:0".to_string())
                // ✅ Cross-region inference model ID with region built-in
            },
            ("anthropic", "claude-3.5-haiku") => {
                Some("us.anthropic.claude-3-5-haiku-20241022-v1:0".to_string())
            },
            ("amazon", "nova-pro") => {
                Some("us.amazon.nova-pro-v1:0".to_string())
            },
            // ✅ Region handling: AWS SDK + cross-inference IDs handle regions
            _ => None,
        }
    }
    
    async fn discover_models(&self) -> Result<Vec<ModelMetadata>, LlmError> {
        // Call Bedrock ListFoundationModels API
        let response = self.client.list_foundation_models().send().await?;
        // Transform to standardized ModelMetadata format
    }
}
```

#### **Multi-Provider Support**

```rust
// Same Claude model, different providers:

// 1. Via AWS Bedrock 
"bedrock:anthropic/claude-3.5-sonnet/200k/text+vision+tools"
→ API: "us.anthropic.claude-3-5-sonnet-20241022-v2:0"
→ Endpoint: AWS Bedrock

// 2. Via Anthropic Direct API
"anthropic:claude-3.5-sonnet/200k/text+vision+tools" 
→ API: "claude-3-5-sonnet-20241022"
→ Endpoint: api.anthropic.com

// 3. Via OpenRouter
"openrouter:anthropic/claude-3.5-sonnet/200k/text+vision+tools"
→ API: "anthropic/claude-3.5-sonnet"  
→ Endpoint: openrouter.ai
```

### **5. Enhanced Agent Builder**

```rust
impl AgentBuilder {
    /// Primary method: string-based model selection with provider prefix
    pub fn model_from_string(mut self, model_identifier: &str) -> Self {
        self.model_identifier = Some(model_identifier.to_string());
        self
    }
    
    /// Convenience methods for common providers
    pub fn bedrock_model(self, model: &str) -> Self {
        self.model_from_string(&format!("bedrock:{}", model))
    }
    
    pub fn anthropic_model(self, model: &str) -> Self {
        self.model_from_string(&format!("anthropic:{}", model))
    }
    
    pub fn openrouter_model(self, model: &str) -> Self {
        self.model_from_string(&format!("openrouter:{}", model))
    }
    
    /// Backward compatibility - existing static models still work
    pub fn model(mut self, model: impl LlmModel + 'static) -> Self {
        self.static_model = Some(Box::new(model));
        self
    }
}
```

### **6. Robust Fallback Mechanism**

```rust
impl ModelRegistry {
    fn infer_capabilities(&self, parsed: &ModelIdentifier, provider: &dyn LlmProvider) -> Option<ModelCapabilities> {
        // Provider-specific capability inference
        match parsed.provider.as_str() {
            "bedrock" => self.infer_bedrock_capabilities(parsed),
            "anthropic" => self.infer_anthropic_capabilities(parsed),
            "openrouter" => self.infer_openrouter_capabilities(parsed),
            _ => Some(ModelCapabilities::text_only()), // ✅ Safe fallback
        }
    }
    
    fn parse_context_window(&self, context_str: &Option<String>) -> Option<usize> {
        context_str.as_ref().and_then(|s| {
            match s.to_lowercase().as_str() {
                s if s.ends_with("k") => s.trim_end_matches("k").parse::<usize>().ok().map(|n| n * 1024),
                s if s.ends_with("m") => s.trim_end_matches("m").parse::<usize>().ok().map(|n| n * 1024 * 1024),
                s => s.parse().ok(),
            }
        })
    }
}
```

## **Implementation Plan**

### **Phase 1: Core Infrastructure (Week 1-2)**
1. ✅ Create `src/llm/model_registry.rs` with ModelRegistry struct
2. ✅ Implement ModelIdentifier parsing with fallback logic  
3. ✅ Create DynamicModel struct implementing LlmModel trait
4. ✅ Add model discovery methods to LlmProvider trait
5. ✅ Unit tests for parsing and fallback mechanisms

### **Phase 2: Provider Integration (Week 3-4)**  
1. ✅ Implement Bedrock provider model discovery using ListFoundationModels API
2. ✅ Add model resolution logic for Bedrock (static mapping initially)
3. ✅ Update Agent builder with dynamic model resolution
4. ✅ Ensure backward compatibility with existing static model usage
5. ✅ Integration tests with real Bedrock API calls

### **Phase 3: Multi-Provider & Advanced Features (Week 5-6)**
1. ✅ Add OpenRouter provider implementation with 400+ model support  
2. ✅ Add Anthropic Direct API provider implementation
3. ✅ Implement model caching and persistence
4. ✅ Add auto-discovery on startup option
5. ✅ Comprehensive error handling with suggested alternatives
6. ✅ Performance optimization and benchmarking

### **Phase 4: Documentation & Migration (Week 7)**
1. ✅ Update CLAUDE.md with new model selection patterns
2. ✅ Create migration guide for existing applications  
3. ✅ Add examples showing all supported patterns
4. ✅ Performance comparison documentation
5. ✅ Deprecation warnings for old patterns (optional)

## **Backward Compatibility Strategy**

```rust
// Phase 1: Dual support - existing code continues working
Agent::builder()
    .model(Bedrock::Claude35Sonnet)  // ✅ Static models still work
    .build().await?;

Agent::builder()
    .model_from_string("bedrock:anthropic/claude-3.5-sonnet/200k/text+vision+tools")  // ✅ New dynamic models
    .build().await?;

// Phase 2: Enhanced string patterns  
Agent::builder()
    .bedrock_model("anthropic/claude-3.5-sonnet/200k") // ✅ Convenience methods
    .build().await?;

// Phase 3: Auto-discovery with caching
Agent::builder()
    .auto_discover_models(true)     // ✅ Populate registry at startup
    .model_from_string("bedrock:anthropic/claude-4")  // ✅ Future models work automatically
    .build().await?;
```

## **Usage Examples**

```rust
// Basic usage with fallbacks
let agent = Agent::builder()
    .model_from_string("bedrock:anthropic/claude-3.5-sonnet")  // Infers context & capabilities
    .build().await?;

// Multi-provider flexibility
let bedrock_agent = Agent::builder()
    .model_from_string("bedrock:anthropic/claude-3.5-sonnet/200k/text+vision+tools")
    .build().await?;

let openrouter_agent = Agent::builder()  
    .model_from_string("openrouter:anthropic/claude-3.5-sonnet/200k/text+vision+tools")
    .build().await?;

// Auto-discovery for new models
let future_agent = Agent::builder()
    .model_from_string("bedrock:anthropic/claude-4/500k/text+vision+tools+reasoning")  // Will work when Claude 4 releases
    .build().await?;
```

## **Files to Create/Modify**

### **New Files**
- `src/llm/model_registry.rs` - Core registry implementation
- `src/llm/model_identifier.rs` - Parsing logic  
- `src/llm/dynamic_model.rs` - Runtime model struct
- `tests/integration/dynamic_models.rs` - Integration tests

### **Modified Files**  
- `src/llm/traits.rs` - Add discovery methods to LlmProvider
- `src/llm/providers/bedrock.rs` - Add model discovery implementation
- `src/agent/mod.rs` - Replace create_model_from_config with dynamic resolution
- `src/agent/builder.rs` - Add string-based model selection methods
- `src/types/agent.rs` - Add model_identifier field to AgentConfig

## **Success Criteria**

1. **Scalability**: Support 100+ Bedrock models and 400+ OpenRouter models without code changes
2. **Multi-Provider**: Same model accessible through different providers with clear differentiation  
3. **Backward Compatibility**: All existing static model usage continues working unchanged
4. **Fallback Resilience**: Graceful handling of missing capabilities, context windows, and unknown models
5. **Performance**: Model resolution cached with sub-millisecond lookup times
6. **Error Handling**: Clear error messages with suggested alternatives for invalid model identifiers

This implementation future-proofs the Stood library by enabling automatic support for new models across all providers while maintaining the robust, type-safe architecture that makes Stood production-ready.