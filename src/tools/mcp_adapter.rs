//! MCP Tool Adapter Implementation
//!
//! This module provides the bridge between MCP tools and the Stood tool system.
//! It enables agents to seamlessly use tools from any MCP server by adapting
//! the MCP protocol to the native tool interface.

use crate::error::StoodError;
use crate::mcp::client::MCPClient;
use crate::mcp::types::{Content, Tool as MCPTool};
use crate::tools::{Tool, ToolError, ToolResult};
use crate::types::content::ToolResultContent;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Adapter that wraps an MCP tool to work with the native tool system
pub struct MCPAgentTool {
    /// The MCP tool definition
    mcp_tool: MCPTool,
    /// Reference to the MCP client for executing the tool
    mcp_client: Arc<RwLock<MCPClient>>,
    /// Optional namespace prefix for the tool name
    namespace: Option<String>,
    /// Cached prefixed name for efficient access
    prefixed_name: String,
}

impl MCPAgentTool {
    /// Create a new MCP tool adapter
    pub fn new(
        mcp_tool: MCPTool,
        mcp_client: Arc<RwLock<MCPClient>>,
        namespace: Option<String>,
    ) -> Self {
        let prefixed_name = match &namespace {
            Some(ns) => format!("{}{}", ns, mcp_tool.name),
            None => mcp_tool.name.clone(),
        };

        Self {
            mcp_tool,
            mcp_client,
            namespace,
            prefixed_name,
        }
    }

    /// Get the prefixed tool name with namespace
    pub fn prefixed_name(&self) -> &str {
        &self.prefixed_name
    }

    /// Convert MCP Content to ToolResultContent
    fn convert_mcp_content(content: &[Content]) -> ToolResultContent {
        if content.is_empty() {
            return ToolResultContent::text("No content returned from MCP tool");
        }

        if content.len() == 1 {
            // Single content item
            match &content[0] {
                Content::Text(text_content) => ToolResultContent::text(text_content.text.clone()),
                Content::Image(image_content) => {
                    // For now, store image data as base64 text since we don't have base64 decoder
                    // TODO: Add proper base64 decoding when needed
                    ToolResultContent::text(format!(
                        "Image data ({}): {}",
                        image_content.mime_type,
                        if image_content.data.len() > 100 {
                            format!("{}...", &image_content.data[..100])
                        } else {
                            image_content.data.clone()
                        }
                    ))
                }
                Content::Resource(resource_content) => {
                    // Convert resource to text representation
                    let resource_text = format!(
                        "Resource: {} ({})",
                        resource_content.uri,
                        resource_content.mime_type.as_deref().unwrap_or("unknown")
                    );
                    ToolResultContent::text(resource_text)
                }
            }
        } else {
            // Multiple content items - create a multiple content block
            let blocks: Vec<ToolResultContent> = content
                .iter()
                .map(|c| Self::convert_mcp_content(std::slice::from_ref(c)))
                .collect();
            ToolResultContent::Multiple { blocks }
        }
    }

    /// Convert StoodError to MCP-aware error with context
    #[allow(dead_code)]
    fn convert_error_with_context(&self, error: StoodError, context: &str) -> StoodError {
        match error {
            StoodError::ToolError { message, .. } => StoodError::tool_error(format!(
                "MCP Tool '{}' {}: {}",
                self.prefixed_name, context, message
            )),
            StoodError::ConfigurationError { message, .. } => {
                StoodError::configuration_error(format!(
                    "MCP Tool '{}' configuration error: {}",
                    self.prefixed_name, message
                ))
            }
            other => other,
        }
    }
}

impl std::fmt::Debug for MCPAgentTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MCPAgentTool")
            .field("name", &self.prefixed_name)
            .field("description", &self.mcp_tool.description)
            .field("mcp_client", &"<MCPClient>")
            .field("namespace", &self.namespace)
            .finish()
    }
}

// Tool trait implementation for unified system
#[async_trait]
impl Tool for MCPAgentTool {
    fn name(&self) -> &str {
        &self.prefixed_name
    }

    fn description(&self) -> &str {
        &self.mcp_tool.description
    }

    fn parameters_schema(&self) -> Value {
        self.mcp_tool.input_schema.clone()
    }

    async fn execute(
        &self,
        parameters: Option<Value>,
        _agent_context: Option<&crate::agent::AgentContext>,
    ) -> Result<ToolResult, ToolError> {
        // Handle the case where no parameters are provided
        let mut input = match parameters {
            Some(Value::Null) | None => {
                // No parameters provided - use empty object
                Value::Object(serde_json::Map::new())
            }
            Some(value) => value,
        };

        // Debug logging to see what we're getting
        tracing::debug!(
            "🔧 MCP TOOL '{}' received parameters: {:?}",
            self.prefixed_name,
            input
        );
        tracing::debug!(
            "🔧 MCP TOOL '{}' parameter type: {}",
            self.prefixed_name,
            match &input {
                serde_json::Value::Null => "null",
                serde_json::Value::Bool(_) => "boolean",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::String(_) => "string",
                serde_json::Value::Array(_) => "array",
                serde_json::Value::Object(_) => "object",
            }
        );

        // Basic validation - ensure it's an object, but be more forgiving
        if !input.is_object() {
            // If it's a string, try to parse it as JSON
            if let Value::String(s) = &input {
                tracing::debug!("🔧 Attempting to parse string parameter as JSON: {}", s);
                match serde_json::from_str::<Value>(s) {
                    Ok(parsed) if parsed.is_object() => {
                        tracing::debug!("✅ Successfully parsed string as JSON object");
                        input = parsed;
                    }
                    Ok(parsed) => {
                        tracing::warn!("⚠️ Parsed JSON but not an object: {:?}", parsed);
                        return Err(ToolError::InvalidParameters {
                            message: format!(
                                "MCP Tool '{}' string parameter parsed to {}, not object: {:?}",
                                self.prefixed_name,
                                match parsed {
                                    serde_json::Value::Null => "null",
                                    serde_json::Value::Bool(_) => "boolean",
                                    serde_json::Value::Number(_) => "number",
                                    serde_json::Value::String(_) => "string",
                                    serde_json::Value::Array(_) => "array",
                                    serde_json::Value::Object(_) => "object",
                                },
                                parsed
                            ),
                        });
                    }
                    Err(e) => {
                        tracing::warn!("❌ Failed to parse string as JSON: {}", e);
                        return Err(ToolError::InvalidParameters {
                            message: format!(
                                "MCP Tool '{}' input must be a JSON object, got unparseable string: {} (error: {})",
                                self.prefixed_name, s, e
                            ),
                        });
                    }
                }
            } else {
                return Err(ToolError::InvalidParameters {
                    message: format!(
                        "MCP Tool '{}' input must be a JSON object, got {} instead: {:?}",
                        self.prefixed_name,
                        match &input {
                            serde_json::Value::Null => "null",
                            serde_json::Value::Bool(_) => "boolean",
                            serde_json::Value::Number(_) => "number",
                            serde_json::Value::String(_) => "string",
                            serde_json::Value::Array(_) => "array",
                            serde_json::Value::Object(_) => "object",
                        },
                        input
                    ),
                });
            }
        }

        // Log MCP tool invocation
        tracing::info!(
            "🔧 MCP TOOL INVOCATION: Calling '{}' (original: '{}')",
            self.prefixed_name,
            self.mcp_tool.name
        );
        tracing::debug!(
            "🔧 MCP tool parameters: {}",
            serde_json::to_string_pretty(&input).unwrap_or_else(|_| "Invalid JSON".to_string())
        );

        let start_time = std::time::Instant::now();

        // Get a write lock on the MCP client (required for call_tool)
        let mut client = self.mcp_client.write().await;

        // Call the MCP tool
        let content_result = client
            .call_tool(&self.mcp_tool.name, Some(input))
            .await
            .map_err(|e| {
                let duration = start_time.elapsed();
                tracing::error!(
                    "❌ MCP TOOL FAILED: '{}' after {:?} - {}",
                    self.prefixed_name,
                    duration,
                    e
                );
                ToolError::ExecutionFailed {
                    message: format!("MCP Tool '{}' execution failed: {}", self.prefixed_name, e),
                }
            })?;

        let duration = start_time.elapsed();
        tracing::info!(
            "✅ MCP TOOL SUCCESS: '{}' completed in {:?} - {} content items",
            self.prefixed_name,
            duration,
            content_result.len()
        );
        tracing::debug!("📝 MCP tool raw response: {} items", content_result.len());

        // Convert the result to the expected format
        match self.convert_content_to_value(content_result) {
            Ok(value) => Ok(ToolResult::success(value)),
            Err(e) => Ok(ToolResult::error(e.to_string())),
        }
    }

    fn source(&self) -> crate::tools::ToolSource {
        crate::tools::ToolSource::MCP
    }
}

impl MCPAgentTool {
    /// Convert MCP Content Vec to JSON Value for tool system
    fn convert_content_to_value(&self, content: Vec<Content>) -> Result<Value, StoodError> {
        // Convert content to ToolResultContent
        let tool_result_content = Self::convert_mcp_content(&content);

        // Serialize the ToolResultContent to JSON Value
        serde_json::to_value(tool_result_content).map_err(|e| {
            StoodError::serialization_error(format!("Failed to serialize MCP tool result: {}", e))
        })
    }
}

/// Registry for managing MCP tools integrated with the main tool system
pub struct MCPToolRegistry {
    /// Reference to the main tool registry
    main_registry: Arc<crate::tools::ToolRegistry>,
    /// Map of MCP client session IDs to tool namespaces
    client_namespaces: Arc<RwLock<std::collections::HashMap<String, String>>>,
}

impl MCPToolRegistry {
    /// Create a new MCP tool registry
    pub fn new(main_registry: Arc<crate::tools::ToolRegistry>) -> Self {
        Self {
            main_registry,
            client_namespaces: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Register all tools from an MCP client
    pub async fn register_mcp_tools(
        &self,
        mcp_client: Arc<RwLock<MCPClient>>,
        namespace: Option<String>,
    ) -> Result<Vec<String>, StoodError> {
        let mut registered_tools = Vec::new();

        // Get the available tools from the MCP client
        let tools = {
            let client = mcp_client.read().await;
            client
                .list_tools()
                .await
                .map_err(|e| StoodError::tool_error(format!("Failed to list MCP tools: {}", e)))?
        };

        // Register each tool using the new Tool trait
        for mcp_tool in tools {
            let adapter =
                MCPAgentTool::new(mcp_tool.clone(), mcp_client.clone(), namespace.clone());

            let tool_name = adapter.prefixed_name().to_string();

            self.main_registry
                .register_tool(Box::new(adapter))
                .await
                .map_err(|e| {
                    StoodError::tool_error(format!(
                        "Failed to register MCP tool '{}': {}",
                        tool_name, e
                    ))
                })?;

            registered_tools.push(tool_name);
        }

        // Store the namespace mapping if provided
        if let Some(ns) = namespace {
            let session_id = {
                let client = mcp_client.read().await;
                match client.session_info().await {
                    Ok((id, _, _, _)) => id,
                    Err(_) => "unknown_session".to_string(),
                }
            };

            let mut namespaces = self.client_namespaces.write().await;
            namespaces.insert(session_id, ns);
        }

        Ok(registered_tools)
    }

    /// Unregister tools for a specific MCP client session
    pub async fn unregister_mcp_session(&self, session_id: &str) -> Result<(), StoodError> {
        let namespace = {
            let mut namespaces = self.client_namespaces.write().await;
            namespaces.remove(session_id)
        };

        if let Some(_ns) = namespace {
            // TODO: Implement tool unregistration from main registry
            // This would require extending the ToolRegistry interface
            tracing::warn!(
                "MCP session {} disconnected, but tool unregistration not yet implemented",
                session_id
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::error::MCPOperationError;
    use crate::mcp::transport::{MCPTransport, TransportInfo, TransportStreams};
    use crate::mcp::types::TextContent;
    use async_trait::async_trait;

    /// Mock transport for testing
    struct MockTransport;

    #[async_trait]
    impl MCPTransport for MockTransport {
        async fn connect(&mut self) -> Result<TransportStreams, MCPOperationError> {
            // For testing, return an error since we don't actually connect
            Err(MCPOperationError::transport(
                "Mock transport - no actual connection",
            ))
        }

        async fn disconnect(&mut self) -> Result<(), MCPOperationError> {
            Ok(())
        }

        fn is_connected(&self) -> bool {
            false
        }

        fn transport_info(&self) -> TransportInfo {
            TransportInfo {
                transport_type: "mock".to_string(),
                endpoint: "mock://test".to_string(),
                supports_reconnection: false,
                max_message_size: None,
            }
        }
    }

    /// Comprehensive Mock MCP Server for integration testing
    pub struct MockMCPServer {
        /// Available tools
        tools: Vec<MCPTool>,
        /// Tool execution handlers
        tool_handlers: std::collections::HashMap<
            String,
            Box<dyn Fn(&serde_json::Value) -> Vec<Content> + Send + Sync>,
        >,
        /// Server capabilities
        capabilities: crate::mcp::types::ServerCapabilities,
        /// Whether server should simulate errors
        simulate_errors: bool,
        /// Connection state
        connected: bool,
    }

    impl MockMCPServer {
        /// Create a new mock MCP server with default tools
        pub fn new() -> Self {
            let mut server = Self {
                tools: Vec::new(),
                tool_handlers: std::collections::HashMap::new(),
                capabilities: crate::mcp::types::ServerCapabilities {
                    tools: Some(crate::mcp::types::ToolsCapability {
                        list_changed: Some(false),
                    }),
                    resources: None,
                    prompts: None,
                },
                simulate_errors: false,
                connected: false,
            };

            // Add default test tools
            server.add_calculator_tool();
            server.add_echo_tool();
            server.add_error_tool();

            server
        }

        /// Add a calculator tool that can perform simple math
        fn add_calculator_tool(&mut self) {
            let tool = MCPTool {
                name: "calculator".to_string(),
                description: "Performs basic mathematical calculations".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "Mathematical expression to evaluate (e.g., '2 + 3 * 4')"
                        }
                    },
                    "required": ["expression"]
                }),
            };

            self.tool_handlers.insert(
                "calculator".to_string(),
                Box::new(|params| {
                    let expression = params["expression"].as_str().unwrap_or("0");

                    // Simple expression evaluator for testing
                    let result = match expression {
                        "2 + 3" => "5",
                        "10 - 4" => "6",
                        "3 * 7" => "21",
                        "15 / 3" => "5",
                        "2 + 3 * 4" => "14", // 2 + (3 * 4)
                        _ => "Error: Expression not supported in mock",
                    };

                    vec![Content::Text(TextContent {
                        text: format!("{} = {}", expression, result),
                    })]
                }),
            );

            self.tools.push(tool);
        }

        /// Add an echo tool that returns input
        fn add_echo_tool(&mut self) {
            let tool = MCPTool {
                name: "echo".to_string(),
                description: "Echoes back the provided message".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message to echo back"
                        },
                        "repeat": {
                            "type": "integer",
                            "description": "Number of times to repeat the message",
                            "default": 1
                        }
                    },
                    "required": ["message"]
                }),
            };

            self.tool_handlers.insert(
                "echo".to_string(),
                Box::new(|params| {
                    let message = params["message"].as_str().unwrap_or("");
                    let repeat = params["repeat"].as_u64().unwrap_or(1) as usize;

                    let repeated_message =
                        (0..repeat).map(|_| message).collect::<Vec<_>>().join(" ");

                    vec![Content::Text(TextContent {
                        text: repeated_message,
                    })]
                }),
            );

            self.tools.push(tool);
        }

        /// Add an error tool that simulates errors
        fn add_error_tool(&mut self) {
            let tool = MCPTool {
                name: "error_tool".to_string(),
                description: "Tool that always returns an error for testing error handling"
                    .to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "error_type": {
                            "type": "string",
                            "description": "Type of error to simulate",
                            "enum": ["timeout", "invalid_params", "server_error"]
                        }
                    },
                    "required": ["error_type"]
                }),
            };

            self.tool_handlers.insert(
                "error_tool".to_string(),
                Box::new(|params| {
                    let error_type = params["error_type"].as_str().unwrap_or("server_error");
                    vec![Content::Text(TextContent {
                        text: format!("Simulated {} error", error_type),
                    })]
                }),
            );

            self.tools.push(tool);
        }

        /// Get available tools
        pub fn list_tools(&self) -> &[MCPTool] {
            &self.tools
        }

        /// Execute a tool call
        pub fn call_tool(
            &self,
            tool_name: &str,
            params: &serde_json::Value,
        ) -> Result<Vec<Content>, String> {
            if self.simulate_errors {
                return Err("Simulated server error".to_string());
            }

            match self.tool_handlers.get(tool_name) {
                Some(handler) => Ok(handler(params)),
                None => Err(format!("Tool '{}' not found", tool_name)),
            }
        }

        /// Set whether to simulate errors
        pub fn set_simulate_errors(&mut self, simulate: bool) {
            self.simulate_errors = simulate;
        }

        /// Get server capabilities
        pub fn capabilities(&self) -> &crate::mcp::types::ServerCapabilities {
            &self.capabilities
        }

        /// Simulate connection
        pub fn connect(&mut self) {
            self.connected = true;
        }

        /// Simulate disconnection
        pub fn disconnect(&mut self) {
            self.connected = false;
        }

        /// Check if connected
        pub fn is_connected(&self) -> bool {
            self.connected
        }
    }

    /// Create a mock MCP client for testing
    fn create_mock_mcp_client() -> MCPClient {
        let config = crate::mcp::client::MCPClientConfig::default();
        let transport = Box::new(MockTransport);
        MCPClient::new(config, transport)
    }

    /// Create a mock MCP tool for testing
    fn create_mock_mcp_tool() -> MCPTool {
        MCPTool {
            name: "test_tool".to_string(),
            description: "A test tool for unit testing".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Test message"
                    }
                },
                "required": ["message"]
            }),
        }
    }

    #[tokio::test]
    async fn test_mcp_agent_tool_creation() {
        let mcp_tool = create_mock_mcp_tool();
        let mcp_client = Arc::new(RwLock::new(create_mock_mcp_client()));

        let adapter = MCPAgentTool::new(mcp_tool, mcp_client, Some("test_namespace".to_string()));

        assert_eq!(adapter.prefixed_name(), "test_namespacetest_tool");
    }

    #[tokio::test]
    async fn test_mcp_tool_specification() {
        let mcp_tool = create_mock_mcp_tool();
        let mcp_client = Arc::new(RwLock::new(create_mock_mcp_client()));

        let adapter = MCPAgentTool::new(mcp_tool.clone(), mcp_client, None);

        assert_eq!(adapter.name(), "test_tool");
        assert_eq!(adapter.description(), &mcp_tool.description);
        assert_eq!(adapter.parameters_schema(), mcp_tool.input_schema);
    }

    #[test]
    fn test_convert_mcp_content_text() {
        let content = vec![Content::Text(TextContent {
            text: "Hello, world!".to_string(),
        })];

        let result = MCPAgentTool::convert_mcp_content(&content);

        match result {
            ToolResultContent::Text { text } => {
                assert_eq!(text, "Hello, world!");
            }
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_convert_mcp_content_multiple() {
        let content = vec![
            Content::Text(TextContent {
                text: "First text".to_string(),
            }),
            Content::Text(TextContent {
                text: "Second text".to_string(),
            }),
        ];

        let result = MCPAgentTool::convert_mcp_content(&content);

        match result {
            ToolResultContent::Multiple { blocks } => {
                assert_eq!(blocks.len(), 2);
            }
            _ => panic!("Expected multiple content"),
        }
    }

    #[test]
    fn test_mock_mcp_server_creation() {
        let server = MockMCPServer::new();
        let tools = server.list_tools();

        // Should have default tools: calculator, echo, error_tool
        assert_eq!(tools.len(), 3);
        assert!(tools.iter().any(|t| t.name == "calculator"));
        assert!(tools.iter().any(|t| t.name == "echo"));
        assert!(tools.iter().any(|t| t.name == "error_tool"));
    }

    #[test]
    fn test_mock_mcp_server_calculator_tool() {
        let server = MockMCPServer::new();

        // Test calculator tool
        let params = serde_json::json!({"expression": "2 + 3"});
        let result = server.call_tool("calculator", &params).unwrap();

        assert_eq!(result.len(), 1);
        if let Content::Text(text_content) = &result[0] {
            assert_eq!(text_content.text, "2 + 3 = 5");
        } else {
            panic!("Expected text content");
        }
    }

    #[test]
    fn test_mock_mcp_server_echo_tool() {
        let server = MockMCPServer::new();

        // Test echo tool
        let params = serde_json::json!({"message": "Hello, world!"});
        let result = server.call_tool("echo", &params).unwrap();

        assert_eq!(result.len(), 1);
        if let Content::Text(text_content) = &result[0] {
            assert_eq!(text_content.text, "Hello, world!");
        } else {
            panic!("Expected text content");
        }
    }

    #[test]
    fn test_mock_mcp_server_echo_tool_with_repeat() {
        let server = MockMCPServer::new();

        // Test echo tool with repeat
        let params = serde_json::json!({"message": "Hi", "repeat": 3});
        let result = server.call_tool("echo", &params).unwrap();

        assert_eq!(result.len(), 1);
        if let Content::Text(text_content) = &result[0] {
            assert_eq!(text_content.text, "Hi Hi Hi");
        } else {
            panic!("Expected text content");
        }
    }

    #[test]
    fn test_mock_mcp_server_error_simulation() {
        let mut server = MockMCPServer::new();

        // Test normal operation
        let params = serde_json::json!({"message": "test"});
        assert!(server.call_tool("echo", &params).is_ok());

        // Enable error simulation
        server.set_simulate_errors(true);
        assert!(server.call_tool("echo", &params).is_err());

        // Disable error simulation
        server.set_simulate_errors(false);
        assert!(server.call_tool("echo", &params).is_ok());
    }

    #[test]
    fn test_mock_mcp_server_connection_state() {
        let mut server = MockMCPServer::new();

        assert!(!server.is_connected());

        server.connect();
        assert!(server.is_connected());

        server.disconnect();
        assert!(!server.is_connected());
    }

    #[test]
    fn test_mock_mcp_server_capabilities() {
        let server = MockMCPServer::new();
        let capabilities = server.capabilities();

        assert!(capabilities.tools.is_some());
        assert!(capabilities.resources.is_none());
        assert!(capabilities.prompts.is_none());
    }

    #[test]
    fn test_mock_mcp_server_unknown_tool() {
        let server = MockMCPServer::new();

        let params = serde_json::json!({"test": "value"});
        let result = server.call_tool("unknown_tool", &params);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn test_mcp_tool_registry_integration() {
        use crate::tools::ToolRegistry;

        // Create a tool registry
        let tool_registry = Arc::new(ToolRegistry::new());
        let _mcp_registry = MCPToolRegistry::new(tool_registry.clone());

        // Simulate creating an MCP client that would use our mock server
        // In real usage, this would connect to an actual MCP server
        let mock_client = Arc::new(RwLock::new(create_mock_mcp_client()));

        // For this test, we'll manually create tools that would come from the mock server
        let calculator_tool = MCPTool {
            name: "calculator".to_string(),
            description: "Performs basic mathematical calculations".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "Mathematical expression to evaluate"
                    }
                },
                "required": ["expression"]
            }),
        };

        // Create adapter for the tool
        let adapter = MCPAgentTool::new(
            calculator_tool.clone(),
            mock_client,
            Some("mock_server".to_string()),
        );

        // Verify the adapter works correctly
        assert_eq!(adapter.prefixed_name(), "mock_servercalculator");

        assert_eq!(adapter.name(), "mock_servercalculator");
        assert_eq!(adapter.description(), &calculator_tool.description);
        assert_eq!(adapter.parameters_schema(), calculator_tool.input_schema);
    }
}
