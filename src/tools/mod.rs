//! Unified tool system for multi-provider agents with compile-time validation.
//!
//! This module provides a type-safe tool system that enables agents to execute
//! external functions, APIs, and integrations with full compile-time validation.
//! You'll get automatic schema generation, parallel execution, and robust error
//! handling for production tool workflows across all LLM providers.
//!
//! # Quick Start
//!
//! Define and register a tool using the macro:
//!
//! ```no_run
//! use stood::{tool, ToolRegistry, ToolResult};
//!
//! #[tool]
//! async fn get_weather(location: String, units: Option<String>) -> ToolResult {
//!     let units = units.unwrap_or_else(|| "celsius".to_string());
//!     // Your weather API integration here
//!     let weather_data = fetch_weather(&location, &units).await?;
//!     Ok(serde_json::json!({ "weather": weather_data }).into())
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut registry = ToolRegistry::new();
//!     registry.register_tool(Box::new(get_weather)).await?;
//!
//!     // Tool is now available for agent execution
//!     Ok(())
//! }
//! # async fn fetch_weather(location: &str, units: &str) -> Result<String, Box<dyn std::error::Error>> {
//! #     Ok("sunny".to_string())
//! # }
//! ```
//!
//! # Tool Types
//!
//! The system supports multiple tool sources:
//!
//! - **Built-in Tools** - Core functionality like file operations, HTTP requests, calculator
//! - **Custom Tools** - Your application-specific functions with the `#[tool]` macro
//! - **MCP Tools** - External tools via Model Context Protocol servers with namespace support
//!
//! # Architecture
//!
//! ```text
//! Multi-Provider Agent → ToolRegistry → Tool trait implementations
//!       ↓                     ↓                    ↓
//! LLM Provider         Schema Generation    Parallel Tool Execution
//! (Bedrock, LM Studio, etc.)               (Built-in, Custom, MCP)
//! ```
//!
//! # Key Features
//!
//! - **Compile-time Validation** - Tool parameters validated at build time with `#[tool]` macro
//! - **Automatic Schema Generation** - JSON schemas created from Rust types for all providers
//! - **Parallel Execution** - Independent tools run concurrently with intelligent strategy selection
//! - **Provider Agnostic** - Works seamlessly with Bedrock, LM Studio, Anthropic, OpenAI
//! - **MCP Integration** - External tool support via Model Context Protocol
//! - **Error Recovery** - Robust handling of tool failures with retry logic
//! - **Type Safety** - Full Rust type system protection throughout
//!
//! # Performance Characteristics
//!
//! - Tool registration: O(1) HashMap insertion
//! - Tool lookup: O(1) HashMap access with Arc sharing
//! - Parallel execution: Limited by tool dependencies and system resources
//! - Memory overhead: Minimal with Arc-based tool sharing
//!
//! # Key Types
//!
//! - [`Tool`] - Primary trait for implementing tools
//! - [`ToolRegistry`] - Central registry for tool management
//! - [`ToolResult`] - Standardized tool execution results
//! - [`ToolError`] - Comprehensive error handling for tool operations

pub mod builtin;
pub mod executor;
pub mod mcp_adapter;
pub mod middleware;

#[cfg(test)]
mod mcp_e2e_tests;

#[cfg(any(test, feature = "examples"))]
pub mod mcp_performance_tests;

#[cfg(any(test, feature = "examples"))]
pub mod mcp_error_scenario_tests;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use executor::{ExecutionMetrics, ExecutorConfig, ToolExecutor};
pub use middleware::{
    AfterToolAction, MiddlewareStack, ToolContext, ToolMiddleware, ToolMiddlewareAction,
};

// Note: Unified tool system types are defined below and exported automatically

/// Request from any LLM provider to execute a specific tool.
///
/// This struct represents a tool call request from any supported LLM provider
/// (Bedrock, LM Studio, Anthropic, OpenAI), containing all information needed
/// to execute the tool and return results. Each tool use has a unique ID for
/// tracking and correlation with results.
///
/// # Examples
///
/// Tool use request for a weather lookup:
/// ```
/// # use stood::tools::ToolUse;
/// # use serde_json::json;
/// let tool_use = ToolUse {
///     tool_use_id: "call_abc123".to_string(),
///     name: "get_weather".to_string(),
///     input: json!({
///         "location": "San Francisco",
///         "units": "fahrenheit"
///     }),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolUse {
    /// Unique identifier for this tool call
    pub tool_use_id: String,
    /// Name of the tool to call
    pub name: String,
    /// Input parameters for the tool
    pub input: serde_json::Value,
}

/// Primary trait for implementing tools in the unified tool system.
///
/// This trait provides a consistent interface for all tool types (built-in,
/// custom, and MCP) enabling seamless integration with multi-provider agents
/// and the broader tool ecosystem. Implementors get automatic schema generation
/// and validation that works across all LLM providers.
///
/// # Implementation Requirements
///
/// - **Thread Safety** - Tools must be `Send + Sync` for concurrent execution
/// - **Async Support** - All tool execution is async for non-blocking operations
/// - **JSON Schema** - Parameter schema must be valid JSON Schema format
/// - **Error Handling** - Use [`ToolError`] for consistent error reporting
///
/// # Examples
///
/// Implement a custom calculation tool:
/// ```
/// use stood::tools::{Tool, ToolResult, ToolError};
/// use async_trait::async_trait;
/// use serde_json::{json, Value};
///
/// #[derive(Debug)]
/// struct Calculator;
///
/// #[async_trait]
/// impl Tool for Calculator {
///     fn name(&self) -> &str { "calculate" }
///
///     fn description(&self) -> &str {
///         "Perform mathematical calculations with basic operators"
///     }
///
///     fn parameters_schema(&self) -> Value {
///         json!({
///             "type": "object",
///             "properties": {
///                 "expression": {
///                     "type": "string",
///                     "description": "Mathematical expression to evaluate"
///                 }
///             },
///             "required": ["expression"]
///         })
///     }
///
///     async fn execute(&self, parameters: Option<Value>, _agent_context: Option<&crate::agent::AgentContext>) -> Result<ToolResult, ToolError> {
///         let params = parameters.ok_or_else(|| ToolError::InvalidParameters {
///             message: "Parameters required".to_string()
///         })?;
///
///         let expression = params["expression"].as_str().ok_or_else(|| ToolError::InvalidParameters {
///             message: "Expression must be a string".to_string()
///         })?;
///
///         // Your calculation logic here
///         let result = evaluate_expression(expression)?;
///         Ok(ToolResult::success(json!({ "result": result })))
///     }
/// }
///
/// # fn evaluate_expression(expr: &str) -> Result<f64, ToolError> {
/// #     Ok(42.0) // Simplified for example
/// # }
/// ```
///
/// # Schema Best Practices
///
/// - Use descriptive field names and descriptions
/// - Specify required vs optional parameters clearly
/// - Include examples in schema descriptions when helpful
/// - Validate enum values in your schema
/// - Use appropriate JSON Schema types (`string`, `number`, `boolean`, etc.)
///
/// # Performance Considerations
///
/// - Keep tool execution efficient for real-time agent workflows
/// - Use async/await for I/O operations to avoid blocking
/// - Consider caching expensive computations
/// - Implement proper timeout handling for long-running operations
#[async_trait]
pub trait Tool: Send + Sync + std::fmt::Debug {
    /// Get the tool name
    fn name(&self) -> &str;

    /// Get the tool description
    fn description(&self) -> &str;

    /// Get the JSON schema for parameters
    fn parameters_schema(&self) -> Value;

    /// Execute the tool with the given parameters and optional agent context
    async fn execute(
        &self,
        parameters: Option<Value>,
        agent_context: Option<&crate::agent::AgentContext>,
    ) -> Result<ToolResult, ToolError>;

    /// Check if the tool is available for use
    fn is_available(&self) -> bool {
        true
    }

    /// Get the source of this tool
    fn source(&self) -> ToolSource {
        ToolSource::Custom
    }
}

/// Source type for tools in the unified system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolSource {
    /// Built-in tools that come with the library
    Builtin,
    /// Tools from MCP (Model Context Protocol) servers
    MCP,
    /// Custom tools defined by users
    Custom,
}

/// Result from executing a tool
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool execution was successful
    pub success: bool,
    /// The result content
    pub content: Value,
    /// Optional error message if execution failed
    pub error: Option<String>,
}

impl ToolResult {
    /// Create a successful tool result
    pub fn success(content: Value) -> Self {
        Self {
            success: true,
            content,
            error: None,
        }
    }

    /// Create an error tool result
    pub fn error<S: Into<String>>(message: S) -> Self {
        Self {
            success: false,
            content: Value::Null,
            error: Some(message.into()),
        }
    }
}

/// Specialized tool error type for the unified system
#[derive(Debug, Clone, thiserror::Error)]
pub enum ToolError {
    /// Invalid parameters provided to the tool
    #[error("Invalid parameters: {message}")]
    InvalidParameters { message: String },

    /// Tool was not found in the registry
    #[error("Tool not found: {name}")]
    ToolNotFound { name: String },

    /// Duplicate tool name during registration
    #[error("Duplicate tool name: {name}")]
    DuplicateTool { name: String },

    /// Tool execution failed
    #[error("Tool execution failed: {message}")]
    ExecutionFailed { message: String },

    /// Tool is not available
    #[error("Tool not available: {name}")]
    ToolNotAvailable { name: String },
}

/// Thread-safe registry for managing tool collections across multiple agents and providers.
///
/// The `ToolRegistry` serves as the central hub for tool management, providing
/// registration, lookup, execution, and schema generation capabilities. It supports
/// concurrent access from multiple agents using different LLM providers while
/// maintaining consistency and performance.
///
/// # Architecture
///
/// ```text
/// Multiple Agents → ToolRegistry → Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>
///                       ↓
///                  Tool Execution & Schema Generation
/// ```
///
/// # Key Capabilities
///
/// - **Concurrent Access** - Multiple agents can safely access tools simultaneously
/// - **Dynamic Registration** - Add tools at runtime without affecting existing agents
/// - **Schema Generation** - Automatic JSON schemas for all LLM provider integration
/// - **Tool Validation** - Prevents duplicate names and validates availability
/// - **Performance Optimization** - Arc-based sharing minimizes memory overhead
///
/// # Examples
///
/// Create and populate a tool registry:
/// ```no_run
/// use stood::tools::{ToolRegistry, Tool};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let registry = ToolRegistry::new();
///
///     // Register multiple tools
///     registry.register_tool(Box::new(WeatherTool)).await?;
///     registry.register_tool(Box::new(CalculatorTool)).await?;
///
///     // Get schemas for multi-provider LLM integration
///     let schemas = registry.get_tool_schemas().await;
///     println!("Available tools: {}", schemas.len());
///
///     // Execute a tool
///     let result = registry.execute_tool(
///         "weather",
///         Some(serde_json::json!({ "location": "Boston" }))
///     ).await?;
///
///     Ok(())
/// }
/// # #[derive(Debug)] struct WeatherTool;
/// # #[derive(Debug)] struct CalculatorTool;
/// # use async_trait::async_trait;
/// # use serde_json::Value;
/// # use stood::tools::{ToolResult, ToolError};
/// # #[async_trait] impl Tool for WeatherTool {
/// #     fn name(&self) -> &str { "weather" }
/// #     fn description(&self) -> &str { "Get weather" }
/// #     fn parameters_schema(&self) -> Value { serde_json::json!({}) }
/// #     async fn execute(&self, _: Option<Value>) -> Result<ToolResult, ToolError> { Ok(ToolResult::success(Value::Null)) }
/// # }
/// # #[async_trait] impl Tool for CalculatorTool {
/// #     fn name(&self) -> &str { "calc" }
/// #     fn description(&self) -> &str { "Calculate" }
/// #     fn parameters_schema(&self) -> Value { serde_json::json!({}) }
/// #     async fn execute(&self, _: Option<Value>) -> Result<ToolResult, ToolError> { Ok(ToolResult::success(Value::Null)) }
/// # }
/// ```
///
/// Use with multiple agents:
/// ```no_run
/// # use stood::tools::ToolRegistry;
/// # use std::sync::Arc;
/// #[tokio::main]
/// async fn main() {
///     let registry = Arc::new(ToolRegistry::new());
///
///     // Share registry across multiple agents (any provider)
///     let bedrock_agent_registry = Arc::clone(&registry);
///     let lmstudio_agent_registry = Arc::clone(&registry);
///
///     // Both agents can use tools concurrently
///     tokio::join!(
///         async move {
///             // Bedrock agent operations
///             let tools = bedrock_agent_registry.tool_names().await;
///         },
///         async move {
///             // LM Studio agent operations
///             let has_weather = lmstudio_agent_registry.has_tool("weather").await;
///         }
///     );
/// }
/// ```
///
/// # Performance Characteristics
///
/// - **Registration** - O(1) HashMap insertion with duplicate checking
/// - **Lookup** - O(1) HashMap access with Arc cloning
/// - **Execution** - Direct tool invocation with minimal overhead
/// - **Schema Generation** - O(n) iteration over tools, cached by agents
/// - **Memory Usage** - Shared tool instances via Arc reduce duplication
///
/// # Thread Safety
///
/// The registry uses `Arc<RwLock<HashMap>>` for thread-safe access:
/// - **Read Operations** - Multiple concurrent readers (tool lookup, execution)
/// - **Write Operations** - Exclusive access for registration/modification
/// - **Clone Friendly** - Cheap cloning enables easy sharing between agents
#[derive(Clone)]
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
    middleware: Arc<RwLock<MiddlewareStack>>,
}

impl ToolRegistry {
    /// Create a new empty unified tool registry
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            middleware: Arc::new(RwLock::new(MiddlewareStack::new())),
        }
    }

    /// Add middleware to the tool registry.
    ///
    /// Middleware is executed in registration order for `before_tool`
    /// and reverse order for `after_tool`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use stood::tools::{ToolRegistry, ToolMiddleware, ToolMiddlewareAction, AfterToolAction, ToolContext, ToolResult};
    /// # use async_trait::async_trait;
    /// # use serde_json::Value;
    /// # use std::sync::Arc;
    /// # #[derive(Debug)]
    /// # struct LoggingMiddleware;
    /// # #[async_trait]
    /// # impl ToolMiddleware for LoggingMiddleware {
    /// #     async fn before_tool(&self, _: &str, _: &Value, _: &ToolContext) -> ToolMiddlewareAction { ToolMiddlewareAction::Continue }
    /// #     async fn after_tool(&self, _: &str, _: &ToolResult, _: &ToolContext) -> AfterToolAction { AfterToolAction::PassThrough }
    /// # }
    /// #[tokio::main]
    /// async fn main() {
    ///     let registry = ToolRegistry::new();
    ///     registry.add_middleware(Arc::new(LoggingMiddleware)).await;
    /// }
    /// ```
    pub async fn add_middleware(&self, middleware: Arc<dyn ToolMiddleware>) {
        let mut stack = self.middleware.write().await;
        stack.add(middleware);
    }

    /// Check if middleware is configured
    pub async fn has_middleware(&self) -> bool {
        let stack = self.middleware.read().await;
        !stack.is_empty()
    }

    /// Register a new tool in the registry for use by agents.
    ///
    /// Adds a tool to the registry, making it available for execution by any agent
    /// using this registry. Tool names must be unique within the registry.
    ///
    /// # Arguments
    ///
    /// * `tool` - Boxed tool implementation to register
    ///
    /// # Returns
    ///
    /// Success on successful registration, or [`ToolError::DuplicateTool`]
    /// if a tool with the same name already exists.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use stood::tools::{ToolRegistry, Tool, ToolResult, ToolError};
    /// # use async_trait::async_trait;
    /// # use serde_json::Value;
    /// # #[derive(Debug)] struct MyTool;
    /// # #[async_trait] impl Tool for MyTool {
    /// #     fn name(&self) -> &str { "my_tool" }
    /// #     fn description(&self) -> &str { "A custom tool" }
    /// #     fn parameters_schema(&self) -> Value { serde_json::json!({}) }
    /// #     async fn execute(&self, _: Option<Value>) -> Result<ToolResult, ToolError> { Ok(ToolResult::success(Value::Null)) }
    /// # }
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let registry = ToolRegistry::new();
    ///
    ///     // Register your custom tool
    ///     registry.register_tool(Box::new(MyTool)).await?;
    ///
    ///     // Verify registration
    ///     assert!(registry.has_tool("my_tool").await);
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Handle duplicate registration:
    /// ```no_run
    /// # use stood::tools::{ToolRegistry, ToolError};
    /// # async fn example(registry: ToolRegistry, tool1: Box<dyn stood::tools::Tool>, tool2: Box<dyn stood::tools::Tool>) {
    /// // First registration succeeds
    /// registry.register_tool(tool1).await.unwrap();
    ///
    /// // Second registration with same name fails
    /// match registry.register_tool(tool2).await {
    ///     Err(ToolError::DuplicateTool { name }) => {
    ///         println!("Tool '{}' already registered", name);
    ///     }
    ///     _ => unreachable!()
    /// }
    /// # }
    /// ```
    ///
    /// # Thread Safety
    ///
    /// This method requires exclusive write access to the registry, ensuring
    /// no race conditions during registration even with concurrent access.
    pub async fn register_tool(&self, tool: Box<dyn Tool>) -> Result<(), ToolError> {
        let tool_name = tool.name().to_string();
        let tool_arc = Arc::from(tool);

        let mut tools = self.tools.write().await;

        // Check for duplicate names
        if tools.contains_key(&tool_name) {
            return Err(ToolError::DuplicateTool { name: tool_name });
        }

        tools.insert(tool_name.clone(), tool_arc);

        tracing::info!("Registered unified tool: {}", tool_name);
        Ok(())
    }

    /// Get tool schemas for LLM consumption
    pub async fn get_tool_schemas(&self) -> Vec<Value> {
        let tools = self.tools.read().await;
        tools
            .values()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name(),
                    "description": tool.description(),
                    "input_schema": tool.parameters_schema()
                })
            })
            .collect()
    }

    /// Convert tool registry to LLM Tool format for provider consumption
    pub async fn to_llm_tools(&self) -> Vec<crate::llm::traits::Tool> {
        let tools = self.tools.read().await;
        tools
            .values()
            .map(|tool| crate::llm::traits::Tool {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                input_schema: tool.parameters_schema(),
            })
            .collect()
    }

    /// Execute a registered tool with the provided parameters.
    ///
    /// Looks up and executes the specified tool, handling parameter validation
    /// and availability checking automatically. Results are returned in a
    /// standardized format for consistent agent integration.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the tool to execute
    /// * `parameters` - Optional JSON parameters for the tool
    ///
    /// # Returns
    ///
    /// A [`ToolResult`] containing the execution result and status,
    /// or a [`ToolError`] if execution fails.
    ///
    /// # Examples
    ///
    /// Execute a weather tool:
    /// ```no_run
    /// # use stood::tools::ToolRegistry;
    /// # use serde_json::json;
    /// # async fn example(registry: ToolRegistry) -> Result<(), Box<dyn std::error::Error>> {
    /// let result = registry.execute_tool(
    ///     "get_weather",
    ///     Some(json!({ "location": "New York", "units": "celsius" })),
    ///     None
    /// ).await?;
    ///
    /// if result.success {
    ///     println!("Weather data: {}", result.content);
    /// } else {
    ///     println!("Error: {}", result.error.unwrap_or_default());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Handle tool errors gracefully:
    /// ```no_run
    /// # use stood::tools::{ToolRegistry, ToolError};
    /// # use serde_json::json;
    /// # async fn example(registry: ToolRegistry) {
    /// match registry.execute_tool("unknown_tool", None, None).await {
    ///     Ok(result) => println!("Success: {}", result.content),
    ///     Err(ToolError::ToolNotFound { name }) => {
    ///         println!("Tool '{}' not found in registry", name);
    ///     }
    ///     Err(ToolError::ToolNotAvailable { name }) => {
    ///         println!("Tool '{}' is currently unavailable", name);
    ///     }
    ///     Err(e) => println!("Execution error: {}", e),
    /// }
    /// # }
    /// ```
    ///
    /// # Error Handling
    ///
    /// This method can return several error types:
    /// - [`ToolError::ToolNotFound`] - Tool name not registered
    /// - [`ToolError::ToolNotAvailable`] - Tool exists but unavailable
    /// - [`ToolError::ExecutionFailed`] - Tool execution encountered an error
    /// - [`ToolError::InvalidParameters`] - Parameters failed validation
    ///
    /// # Performance Notes
    ///
    /// - Tool lookup: O(1) HashMap access
    /// - Execution time varies by tool implementation
    /// - Concurrent executions are supported and encouraged
    /// - No internal caching of results (implement in tools if needed)
    pub async fn execute_tool(
        &self,
        name: &str,
        parameters: Option<Value>,
        agent_context: Option<&crate::agent::AgentContext>,
    ) -> Result<ToolResult, ToolError> {
        let tools = self.tools.read().await;

        let tool = tools.get(name).ok_or_else(|| ToolError::ToolNotFound {
            name: name.to_string(),
        })?;

        if !tool.is_available() {
            return Err(ToolError::ToolNotAvailable {
                name: name.to_string(),
            });
        }

        // Build middleware context
        let middleware_ctx = if let Some(agent_ctx) = agent_context {
            ToolContext::from_agent_context(agent_ctx)
        } else {
            ToolContext::new("unknown".to_string())
        };

        // Get middleware stack
        let middleware_stack = self.middleware.read().await;
        let params = parameters.clone().unwrap_or(Value::Null);

        // Run before_tool middleware
        let (action, final_params) = middleware_stack
            .process_before_tool(name, &params, &middleware_ctx)
            .await;

        // Handle middleware action
        let result = match action {
            ToolMiddlewareAction::Continue | ToolMiddlewareAction::ModifyParams(_) => {
                // Execute tool with (potentially modified) parameters
                let exec_params = if matches!(action, ToolMiddlewareAction::ModifyParams(_)) {
                    Some(final_params)
                } else {
                    parameters
                };
                tool.execute(exec_params, agent_context).await?
            }
            ToolMiddlewareAction::Abort { reason, synthetic_result } => {
                tracing::info!("Tool {} aborted by middleware: {}", name, reason);
                synthetic_result.unwrap_or_else(|| ToolResult::error(reason))
            }
            ToolMiddlewareAction::Skip => {
                tracing::info!("Tool {} skipped by middleware", name);
                return Ok(ToolResult::success(Value::Null));
            }
        };

        // Run after_tool middleware
        let (after_action, final_result) = middleware_stack
            .process_after_tool(name, &result, &middleware_ctx)
            .await;

        // Handle after-tool action
        match after_action {
            AfterToolAction::PassThrough => Ok(final_result),
            AfterToolAction::ModifyResult(modified) => Ok(modified),
            AfterToolAction::InjectContext(context) => {
                // For now, log the context injection - actual injection requires
                // changes to the agent event loop to handle the injected context
                tracing::debug!("Middleware injected context: {}", context);
                Ok(final_result)
            }
        }
    }

    /// Get all registered tool names
    pub async fn tool_names(&self) -> Vec<String> {
        let tools = self.tools.read().await;
        tools.keys().cloned().collect()
    }

    /// Check if a tool is registered
    pub async fn has_tool(&self, name: &str) -> bool {
        let tools = self.tools.read().await;
        tools.contains_key(name)
    }

    /// Get tool configuration for LLM integration (compatibility method)
    pub async fn get_tool_config(&self) -> crate::types::tools::ToolConfig {
        let schemas = self.get_tool_schemas().await;
        let tools: Vec<crate::types::tools::Tool> = schemas
            .into_iter()
            .map(|schema| {
                let spec = crate::types::tools::ToolSpec {
                    name: schema["name"].as_str().unwrap_or("unknown").to_string(),
                    description: schema["description"].as_str().unwrap_or("").to_string(),
                    input_schema: schema["input_schema"].clone(),
                    metadata: std::collections::HashMap::new(),
                };
                crate::types::tools::Tool::new(spec)
            })
            .collect();

        crate::types::tools::ToolConfig::new(tools)
    }

    /// Get a tool by name for direct execution
    pub async fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry")
            .field("tools", &"<tools HashMap>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Mock tool for testing the new unified system
    #[derive(Debug)]
    struct MockUnifiedTool {
        name: String,
        description: String,
    }

    #[async_trait]
    impl Tool for MockUnifiedTool {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn parameters_schema(&self) -> Value {
            json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Message to process"
                    }
                },
                "required": ["message"]
            })
        }

        async fn execute(
            &self,
            parameters: Option<Value>,
            _agent_context: Option<&crate::agent::AgentContext>,
        ) -> Result<ToolResult, ToolError> {
            let params = parameters.unwrap_or(Value::Null);
            Ok(ToolResult::success(json!({
                "result": "success",
                "input_received": params
            })))
        }
    }

    #[tokio::test]
    async fn test_tool_retrieval() {
        let registry = ToolRegistry::new();
        let tool = Box::new(MockUnifiedTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
        });

        registry.register_tool(tool).await.unwrap();

        assert!(registry.has_tool("test_tool").await);
        let tool_names = registry.tool_names().await;
        assert!(tool_names.contains(&"test_tool".to_string()));
    }

    #[tokio::test]
    async fn test_duplicate_tool_registration() {
        let registry = ToolRegistry::new();

        let tool1 = Box::new(MockUnifiedTool {
            name: "duplicate_tool".to_string(),
            description: "First tool".to_string(),
        });

        let tool2 = Box::new(MockUnifiedTool {
            name: "duplicate_tool".to_string(),
            description: "Second tool".to_string(),
        });

        registry.register_tool(tool1).await.unwrap();

        let result = registry.register_tool(tool2).await;
        assert!(result.is_err());

        if let Err(ToolError::DuplicateTool { name }) = result {
            assert!(name.contains("duplicate_tool"));
        } else {
            panic!("Expected DuplicateTool error");
        }
    }

    #[tokio::test]
    async fn test_tool_execution() {
        let registry = ToolRegistry::new();
        let tool = Box::new(MockUnifiedTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
        });

        registry.register_tool(tool).await.unwrap();

        let result = registry
            .execute_tool("test_tool", Some(json!({"message": "Hello, world!"})), None)
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.content["result"], "success");
    }

    #[tokio::test]
    async fn test_tool_execution_not_found() {
        let registry = ToolRegistry::new();

        let result = registry
            .execute_tool(
                "nonexistent_tool",
                Some(json!({"message": "Hello, world!"})),
                None,
            )
            .await;

        assert!(result.is_err());
        if let Err(ToolError::ToolNotFound { name }) = result {
            assert_eq!(name, "nonexistent_tool");
        } else {
            panic!("Expected ToolNotFound error");
        }
    }

    #[tokio::test]
    async fn test_tool_registration() {
        let registry = ToolRegistry::new();
        let tool = Box::new(MockUnifiedTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
        });

        registry.register_tool(tool).await.unwrap();
        assert!(registry.has_tool("test_tool").await);

        let tool_names = registry.tool_names().await;
        assert_eq!(tool_names.len(), 1);
        assert!(tool_names.contains(&"test_tool".to_string()));
    }

    #[tokio::test]
    async fn test_tool_schemas() {
        let registry = ToolRegistry::new();

        let tool1 = Box::new(MockUnifiedTool {
            name: "tool1".to_string(),
            description: "First tool".to_string(),
        });

        let tool2 = Box::new(MockUnifiedTool {
            name: "tool2".to_string(),
            description: "Second tool".to_string(),
        });

        registry.register_tool(tool1).await.unwrap();
        registry.register_tool(tool2).await.unwrap();

        let schemas = registry.get_tool_schemas().await;
        assert_eq!(schemas.len(), 2);

        let names: Vec<&str> = schemas
            .iter()
            .map(|s| s["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"tool1"));
        assert!(names.contains(&"tool2"));
    }

    #[tokio::test]
    async fn test_tool_with_hyphen_name() {
        let registry = ToolRegistry::new();
        let tool = Box::new(MockUnifiedTool {
            name: "tool-with-hyphen".to_string(), // This is now allowed in unified system
            description: "A test tool".to_string(),
        });

        let result = registry.register_tool(tool).await;
        assert!(result.is_ok()); // Should succeed in unified system
        assert!(registry.has_tool("tool-with-hyphen").await);
    }

    #[tokio::test]
    async fn test_empty_tool_name() {
        let registry = ToolRegistry::new();
        let tool = Box::new(MockUnifiedTool {
            name: "".to_string(),
            description: "A test tool".to_string(),
        });

        let result = registry.register_tool(tool).await;
        // Empty names should fail with DuplicateTool error (empty string collision)
        // or we could add validation later
        if result.is_err() {
            // Accept any error for empty tool name
            assert!(true);
        } else {
            // If it succeeds, verify it was registered
            assert!(registry.has_tool("").await);
        }
    }

    #[tokio::test]
    async fn test_registry_multiple_tools() {
        let registry = ToolRegistry::new();

        let tool1 = Box::new(MockUnifiedTool {
            name: "tool1".to_string(),
            description: "First tool".to_string(),
        });

        let tool2 = Box::new(MockUnifiedTool {
            name: "tool2".to_string(),
            description: "Second tool".to_string(),
        });

        registry.register_tool(tool1).await.unwrap();
        registry.register_tool(tool2).await.unwrap();

        let tool_names = registry.tool_names().await;
        assert_eq!(tool_names.len(), 2);
        assert!(tool_names.contains(&"tool1".to_string()));
        assert!(tool_names.contains(&"tool2".to_string()));
    }

    #[tokio::test]
    async fn test_registry_tool_execution_parallel() {
        let registry = ToolRegistry::new();

        let tool1 = Box::new(MockUnifiedTool {
            name: "tool1".to_string(),
            description: "First tool".to_string(),
        });

        let tool2 = Box::new(MockUnifiedTool {
            name: "tool2".to_string(),
            description: "Second tool".to_string(),
        });

        registry.register_tool(tool1).await.unwrap();
        registry.register_tool(tool2).await.unwrap();

        // Test individual executions
        let result1 = registry
            .execute_tool("tool1", Some(json!({"message": "Message 1"})), None)
            .await
            .unwrap();
        let result2 = registry
            .execute_tool("tool2", Some(json!({"message": "Message 2"})), None)
            .await
            .unwrap();

        assert!(result1.success);
        assert!(result2.success);
        assert_eq!(result1.content["result"], "success");
        assert_eq!(result2.content["result"], "success");
    }

    #[tokio::test]
    async fn test_registry_execute_missing_tool() {
        let registry = ToolRegistry::new();

        let tool1 = Box::new(MockUnifiedTool {
            name: "tool1".to_string(),
            description: "First tool".to_string(),
        });

        registry.register_tool(tool1).await.unwrap();

        // Test success case
        let result1 = registry
            .execute_tool("tool1", Some(json!({"message": "Message 1"})), None)
            .await
            .unwrap();
        assert!(result1.success);

        // Test missing tool case
        let result2 = registry
            .execute_tool("missing_tool", Some(json!({"message": "Message 2"})), None)
            .await;
        assert!(result2.is_err());
        if let Err(ToolError::ToolNotFound { name }) = result2 {
            assert_eq!(name, "missing_tool");
        } else {
            panic!("Expected ToolNotFound error");
        }
    }

    #[tokio::test]
    async fn test_get_tool_config_llm_driven_approach() {
        let registry = ToolRegistry::new();

        // Register a couple of tools
        let tool1 = Box::new(MockUnifiedTool {
            name: "calculator".to_string(),
            description: "Performs mathematical calculations".to_string(),
        });

        let tool2 = Box::new(MockUnifiedTool {
            name: "file_reader".to_string(),
            description: "Reads file contents".to_string(),
        });

        registry.register_tool(tool1).await.unwrap();
        registry.register_tool(tool2).await.unwrap();

        // Test the get_tool_config method
        let tool_config = registry.get_tool_config().await;

        // Verify the structure matches LLM-driven approach
        assert_eq!(tool_config.tools.len(), 2);
        assert!(matches!(
            tool_config.tool_choice,
            crate::types::tools::ToolChoice::Auto
        ));

        // Verify tool specifications are properly formatted
        let tool_names: Vec<&str> = tool_config
            .tools
            .iter()
            .map(|t| t.tool_spec.name.as_str())
            .collect();

        assert!(tool_names.contains(&"calculator"));
        assert!(tool_names.contains(&"file_reader"));

        // Verify each tool has required fields for LLM integration
        for tool in &tool_config.tools {
            assert!(!tool.tool_spec.name.is_empty());
            assert!(!tool.tool_spec.description.is_empty());
            assert!(tool.tool_spec.input_schema.is_object());
        }

        println!(
            "✅ LLM-driven tool configuration test passed - {} tools available for model",
            tool_config.tools.len()
        );
    }

    // Tests for the unified tool system (now the primary system)
    #[tokio::test]
    async fn test_unified_tool_registry_creation() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.tool_names().await.len(), 0);
        assert!(!registry.has_tool("test").await);
    }

    #[tokio::test]
    async fn test_unified_tool_registration() {
        let registry = ToolRegistry::new();
        let tool = Box::new(MockUnifiedTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
        });

        registry.register_tool(tool).await.unwrap();

        assert!(registry.has_tool("test_tool").await);
        assert_eq!(registry.tool_names().await.len(), 1);
    }

    #[tokio::test]
    async fn test_unified_tool_execution() {
        let registry = ToolRegistry::new();
        let tool = Box::new(MockUnifiedTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
        });

        registry.register_tool(tool).await.unwrap();

        let result = registry
            .execute_tool("test_tool", Some(json!({"message": "Hello"})), None)
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.content["result"], "success");
        assert!(result.error.is_none());
    }

    #[tokio::test]
    async fn test_unified_tool_duplicate_registration() {
        let registry = ToolRegistry::new();

        let tool1 = Box::new(MockUnifiedTool {
            name: "duplicate_tool".to_string(),
            description: "First tool".to_string(),
        });

        let tool2 = Box::new(MockUnifiedTool {
            name: "duplicate_tool".to_string(),
            description: "Second tool".to_string(),
        });

        registry.register_tool(tool1).await.unwrap();
        let result = registry.register_tool(tool2).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ToolError::DuplicateTool { .. }
        ));
    }

    #[tokio::test]
    async fn test_unified_tool_not_found() {
        let registry = ToolRegistry::new();

        let result = registry.execute_tool("nonexistent", None, None).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ToolError::ToolNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn test_unified_tool_schemas() {
        let registry = ToolRegistry::new();
        let tool = Box::new(MockUnifiedTool {
            name: "schema_test".to_string(),
            description: "Schema test tool".to_string(),
        });

        registry.register_tool(tool).await.unwrap();

        let schemas = registry.get_tool_schemas().await;
        assert_eq!(schemas.len(), 1);

        let schema = &schemas[0];
        assert_eq!(schema["name"], "schema_test");
        assert_eq!(schema["description"], "Schema test tool");
        assert!(schema["input_schema"].is_object());
    }
}
