//! MCP Server implementation
//!
//! This module provides the core server-side implementation of the Model Context Protocol (MCP).
//! It enables the Stood library to act as an MCP server, exposing tools, resources, and prompts
//! to MCP clients through standardized JSON-RPC 2.0 communication.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::mcp::error::MCPOperationError;
use crate::mcp::types::{
    CallToolRequest, CallToolResult, Content, InitializeRequest, InitializeResult,
    ListToolsRequest, ListToolsResult, MCPRequest, MCPResponse, ServerCapabilities, TextContent,
    Tool as MCPTool, ToolsCapability,
};
use crate::tools::{ToolRegistry, ToolResult};

/// Core trait for MCP server implementations
///
/// This trait defines the interface that MCP servers must implement to handle
/// client requests. Implementations can choose which capabilities to support.
#[async_trait]
pub trait MCPServer: Send + Sync {
    /// Initialize a new MCP session with capability negotiation
    ///
    /// This is called when a client connects and sends an initialize request.
    /// The server should return its capabilities and any initialization parameters.
    async fn initialize(
        &self,
        request: InitializeRequest,
    ) -> Result<InitializeResult, MCPOperationError>;

    /// List available tools that this server provides
    ///
    /// Returns the tools that clients can execute through this server.
    /// Tools include their schema definitions for parameter validation.
    async fn list_tools(
        &self,
        request: ListToolsRequest,
    ) -> Result<ListToolsResult, MCPOperationError>;

    /// Execute a tool with the given parameters
    ///
    /// This is the core functionality - executing a named tool with provided
    /// parameters and returning the results in MCP format.
    async fn call_tool(
        &self,
        request: CallToolRequest,
    ) -> Result<CallToolResult, MCPOperationError>;

    /// Get the server's capabilities
    ///
    /// Returns what features this server supports (tools, resources, prompts).
    fn capabilities(&self) -> ServerCapabilities;

    /// Handle server shutdown and cleanup
    ///
    /// Called when the server is shutting down to perform any necessary cleanup.
    async fn shutdown(&self) -> Result<(), MCPOperationError> {
        Ok(())
    }
}

/// Configuration for MCP server instances
#[derive(Debug, Clone)]
pub struct MCPServerConfig {
    /// Server name for identification
    pub name: String,
    /// Server version
    pub version: String,
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    /// Session timeout in seconds
    pub session_timeout_secs: u64,
    /// Whether to enable experimental features
    pub experimental_features: bool,
}

impl Default for MCPServerConfig {
    fn default() -> Self {
        Self {
            name: "stood-mcp-server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            max_sessions: 100,
            session_timeout_secs: 300, // 5 minutes
            experimental_features: false,
        }
    }
}

/// Session state for an MCP client connection
#[derive(Debug, Clone)]
pub struct MCPSession {
    /// Unique session identifier
    pub session_id: String,
    /// Client capabilities from initialization
    pub client_capabilities: Option<Value>,
    /// When the session was created
    pub created_at: std::time::Instant,
    /// Whether the session is initialized
    pub initialized: bool,
}

impl Default for MCPSession {
    fn default() -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            client_capabilities: None,
            created_at: std::time::Instant::now(),
            initialized: false,
        }
    }
}

impl MCPSession {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Tool-based MCP server that exposes tools from a ToolRegistry
///
/// This implementation allows any Rust application using the Stood library
/// to act as an MCP server by exposing its tools through the MCP protocol.
pub struct StoodMCPServer {
    /// Configuration for this server
    config: MCPServerConfig,
    /// Tool registry containing available tools
    tool_registry: Arc<ToolRegistry>,
    /// Active sessions by session ID
    sessions: Arc<RwLock<HashMap<String, MCPSession>>>,
    /// Server capabilities
    capabilities: ServerCapabilities,
}

impl StoodMCPServer {
    /// Create a new Stood MCP server
    pub fn new(config: MCPServerConfig, tool_registry: Arc<ToolRegistry>) -> Self {
        let capabilities = ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            resources: None, // Not implemented yet
            prompts: None,   // Not implemented yet
        };

        Self {
            config,
            tool_registry,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            capabilities,
        }
    }

    /// Register a new session
    async fn register_session(&self) -> Result<String, MCPOperationError> {
        let mut sessions = self.sessions.write().await;

        // Check max sessions limit
        if sessions.len() >= self.config.max_sessions {
            return Err(MCPOperationError::session(
                "Maximum number of sessions reached",
            ));
        }

        let session = MCPSession::new();
        let session_id = session.session_id.clone();
        sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// Get a session by ID
    #[allow(dead_code)]
    async fn get_session(&self, session_id: &str) -> Result<MCPSession, MCPOperationError> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| MCPOperationError::session("Session not found"))
    }

    /// Update session state
    async fn update_session<F>(
        &self,
        session_id: &str,
        update_fn: F,
    ) -> Result<(), MCPOperationError>
    where
        F: FnOnce(&mut MCPSession),
    {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| MCPOperationError::session("Session not found"))?;

        update_fn(session);
        Ok(())
    }

    /// Remove a session
    #[allow(dead_code)]
    async fn remove_session(&self, session_id: &str) -> Result<(), MCPOperationError> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }

    /// Convert a Stood ToolResult to MCP Content
    fn tool_result_to_content(&self, result: &ToolResult) -> Vec<Content> {
        vec![Content::Text(TextContent {
            text: serde_json::to_string_pretty(&result.content)
                .unwrap_or_else(|_| "Invalid tool result".to_string()),
        })]
    }
}

#[async_trait]
impl MCPServer for StoodMCPServer {
    async fn initialize(
        &self,
        request: InitializeRequest,
    ) -> Result<InitializeResult, MCPOperationError> {
        // Register new session
        let session_id = self.register_session().await?;

        // Update session with client capabilities
        self.update_session(&session_id, |session| {
            session.client_capabilities = Some(request.capabilities.unwrap_or_default());
            session.initialized = true;
        })
        .await?;

        Ok(InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: self.capabilities.clone(),
            server_info: serde_json::json!({
                "name": self.config.name,
                "version": self.config.version
            }),
            instructions: Some(format!(
                "Connected to {} v{} - {} tools available",
                self.config.name,
                self.config.version,
                self.tool_registry.tool_names().await.len()
            )),
        })
    }

    async fn list_tools(
        &self,
        _request: ListToolsRequest,
    ) -> Result<ListToolsResult, MCPOperationError> {
        let tool_schemas = self.tool_registry.get_tool_schemas().await;
        let mut mcp_tools = Vec::new();

        for schema in tool_schemas {
            let mcp_tool = MCPTool {
                name: schema["name"].as_str().unwrap_or("unknown").to_string(),
                description: schema["description"].as_str().unwrap_or("").to_string(),
                input_schema: schema["input_schema"].clone(),
            };
            mcp_tools.push(mcp_tool);
        }

        Ok(ListToolsResult { tools: mcp_tools })
    }

    async fn call_tool(
        &self,
        request: CallToolRequest,
    ) -> Result<CallToolResult, MCPOperationError> {
        // Execute tool through registry
        let result = self
            .tool_registry
            .execute_tool(&request.name, request.arguments, None)
            .await;
        match result {
            Ok(tool_result) => {
                let content = self.tool_result_to_content(&tool_result);
                Ok(CallToolResult {
                    content,
                    is_error: Some(!tool_result.success),
                })
            }
            Err(e) => {
                let error_content = vec![Content::Text(TextContent {
                    text: format!("Tool execution failed: {}", e),
                })];
                Ok(CallToolResult {
                    content: error_content,
                    is_error: Some(true),
                })
            }
        }
    }

    fn capabilities(&self) -> ServerCapabilities {
        self.capabilities.clone()
    }

    async fn shutdown(&self) -> Result<(), MCPOperationError> {
        // Clear all sessions
        let mut sessions = self.sessions.write().await;
        sessions.clear();
        Ok(())
    }
}

/// MCP Server handler that manages request routing
///
/// This handles the JSON-RPC protocol and routes requests to the appropriate
/// server methods based on the MCP specification.
pub struct MCPServerHandler<S: MCPServer> {
    server: Arc<S>,
    #[allow(dead_code)]
    current_session: Option<String>,
}

impl<S: MCPServer> MCPServerHandler<S> {
    pub fn new(server: Arc<S>) -> Self {
        Self {
            server,
            current_session: None,
        }
    }

    /// Handle an incoming MCP request and generate a response
    pub async fn handle_request(
        &mut self,
        request: MCPRequest,
    ) -> Result<MCPResponse, MCPOperationError> {
        match request.method.as_str() {
            "initialize" => {
                let params: InitializeRequest =
                    serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                        MCPOperationError::json_rpc_invalid_params(format!(
                            "Invalid initialize params: {}",
                            e
                        ))
                    })?;

                let result = self.server.initialize(params).await?;

                let result_value = serde_json::to_value(result).map_err(|e| {
                    MCPOperationError::json_rpc_internal_error(format!(
                        "Failed to serialize result: {}",
                        e
                    ))
                })?;

                Ok(MCPResponse::success(request.id, result_value))
            }
            "tools/list" => {
                let params: ListToolsRequest =
                    serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                        MCPOperationError::json_rpc_invalid_params(format!(
                            "Invalid list_tools params: {}",
                            e
                        ))
                    })?;

                let result = self.server.list_tools(params).await?;

                let result_value = serde_json::to_value(result).map_err(|e| {
                    MCPOperationError::json_rpc_internal_error(format!(
                        "Failed to serialize result: {}",
                        e
                    ))
                })?;

                Ok(MCPResponse::success(request.id, result_value))
            }
            "tools/call" => {
                let params: CallToolRequest =
                    serde_json::from_value(request.params.unwrap_or_default()).map_err(|e| {
                        MCPOperationError::json_rpc_invalid_params(format!(
                            "Invalid call_tool params: {}",
                            e
                        ))
                    })?;

                let result = self.server.call_tool(params).await?;

                let result_value = serde_json::to_value(result).map_err(|e| {
                    MCPOperationError::json_rpc_internal_error(format!(
                        "Failed to serialize result: {}",
                        e
                    ))
                })?;

                Ok(MCPResponse::success(request.id, result_value))
            }
            _ => Err(MCPOperationError::json_rpc_method_not_found(format!(
                "Unknown method: {}",
                request.method
            ))),
        }
    }

    /// Handle an error and convert it to an error response
    pub fn handle_error(
        &self,
        request_id: serde_json::Value,
        error: MCPOperationError,
    ) -> MCPResponse {
        use crate::mcp::types::MCPError;
        let mcp_error = MCPError::internal_error(error.to_string());
        MCPResponse::error(request_id, mcp_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::builtin::CalculatorTool;
    use serde_json::json;

    async fn create_test_server() -> StoodMCPServer {
        let config = MCPServerConfig::default();
        let registry = Arc::new(ToolRegistry::new());

        // Register a test tool
        let calc_tool = CalculatorTool::new();
        registry.register_tool(Box::new(calc_tool)).await.unwrap();

        StoodMCPServer::new(config, registry)
    }

    #[tokio::test]
    async fn test_server_creation() {
        let server = create_test_server().await;
        let capabilities = server.capabilities();
        assert!(capabilities.tools.is_some());
        assert!(capabilities.resources.is_none());
        assert!(capabilities.prompts.is_none());
    }

    #[tokio::test]
    async fn test_initialize() {
        let server = create_test_server().await;

        let request = InitializeRequest {
            protocol_version: "2024-11-05".to_string(),
            capabilities: Some(json!({
                "roots": {
                    "listChanged": true
                }
            })),
            client_info: json!({
                "name": "test-client",
                "version": "1.0.0"
            }),
        };

        let result = server.initialize(request).await.unwrap();
        assert_eq!(result.protocol_version, "2024-11-05");
        assert!(result.capabilities.tools.is_some());
        assert!(result.instructions.is_some());
    }

    #[tokio::test]
    async fn test_list_tools() {
        let server = create_test_server().await;

        let request = ListToolsRequest {};
        let result = server.list_tools(request).await.unwrap();

        assert_eq!(result.tools.len(), 1);
        assert_eq!(result.tools[0].name, "calculator");
        assert!(!result.tools[0].description.is_empty());
    }

    #[tokio::test]
    async fn test_call_tool() {
        let server = create_test_server().await;

        let request = CallToolRequest {
            name: "calculator".to_string(),
            arguments: Some(json!({
                "expression": "2 + 2"
            })),
        };

        let result = server.call_tool(request).await.unwrap();
        assert!(!result.content.is_empty());

        if let Content::Text(text) = &result.content[0] {
            // The content should contain the calculation result
            assert!(!text.text.is_empty());
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_call_tool_error() {
        let server = create_test_server().await;

        let request = CallToolRequest {
            name: "nonexistent_tool".to_string(),
            arguments: Some(json!({})),
        };

        let result = server.call_tool(request).await.unwrap();
        assert_eq!(result.is_error, Some(true));
        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_server_handler() {
        let server = Arc::new(create_test_server().await);
        let mut handler = MCPServerHandler::new(server);

        // Test initialize request
        let init_request = MCPRequest::new(
            json!(1),
            "initialize",
            Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            })),
        );

        let response = handler.handle_request(init_request).await.unwrap();
        assert!(matches!(
            response.payload,
            crate::mcp::types::MCPResponsePayload::Success { .. }
        ));

        // Test list tools request
        let list_request = MCPRequest::new(json!(2), "tools/list", Some(json!({})));

        let response = handler.handle_request(list_request).await.unwrap();
        assert!(matches!(
            response.payload,
            crate::mcp::types::MCPResponsePayload::Success { .. }
        ));

        // Test call tool request
        let call_request = MCPRequest::new(
            json!(3),
            "tools/call",
            Some(json!({
                "name": "calculator",
                "arguments": {"expression": "2 + 2"}
            })),
        );

        let response = handler.handle_request(call_request).await.unwrap();
        assert!(matches!(
            response.payload,
            crate::mcp::types::MCPResponsePayload::Success { .. }
        ));
    }

    #[tokio::test]
    async fn test_unknown_method_error() {
        let server = Arc::new(create_test_server().await);
        let mut handler = MCPServerHandler::new(server);

        let request = MCPRequest::new(json!(1), "unknown/method", None);

        let result = handler.handle_request(request).await;
        assert!(result.is_err());

        // Just check that we get an error for unknown method
        assert!(result.unwrap_err().to_string().contains("Method not found"));
    }

    #[tokio::test]
    async fn test_session_management() {
        let server = create_test_server().await;

        // Test session registration
        let session_id = server.register_session().await.unwrap();
        assert!(!session_id.is_empty());

        // Test session retrieval
        let session = server.get_session(&session_id).await.unwrap();
        assert_eq!(session.session_id, session_id);
        assert!(!session.initialized);

        // Test session update
        server
            .update_session(&session_id, |session| {
                session.initialized = true;
            })
            .await
            .unwrap();

        let updated_session = server.get_session(&session_id).await.unwrap();
        assert!(updated_session.initialized);

        // Test session removal
        server.remove_session(&session_id).await.unwrap();
        let result = server.get_session(&session_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_max_sessions_limit() {
        let mut config = MCPServerConfig::default();
        config.max_sessions = 1;

        let registry = Arc::new(ToolRegistry::new());
        let server = StoodMCPServer::new(config, registry);

        // First session should succeed
        let session1 = server.register_session().await.unwrap();
        assert!(!session1.is_empty());

        // Second session should fail
        let result = server.register_session().await;
        assert!(result.is_err());
    }
}
