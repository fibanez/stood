//! MCP protocol message types and data structures
//!
//! This module provides the complete type system for Model Context Protocol communication.
//! You'll find all message formats, capability definitions, and content types needed
//! for MCP integration.
//!
//! # Message Types
//!
//! MCP uses JSON-RPC 2.0 as its foundation with three core message types:
//!
//! ```rust
//! use stood::mcp::types::{MCPRequest, MCPResponse, MCPNotification};
//! use serde_json::json;
//!
//! // Create a request
//! let request = MCPRequest::new(json!(1), "tools/list", None);
//!
//! // Create a successful response
//! let response = MCPResponse::success(json!(1), json!({"tools": []}));
//!
//! // Create a notification
//! let notification = MCPNotification::new("tools/list_changed", None);
//! ```
//!
//! # Content Types
//!
//! MCP supports multiple content formats for tool results and resource data:
//!
//! - **Text** - Plain text content
//! - **Image** - Base64-encoded images with MIME types
//! - **Resource** - References to external resources
//!
//! # Capabilities
//!
//! Server and client capabilities define what features are supported:
//!
//! ```rust
//! use stood::mcp::types::{ServerCapabilities, ToolsCapability};
//!
//! let capabilities = ServerCapabilities {
//!     tools: Some(ToolsCapability {
//!         list_changed: Some(true),
//!     }),
//!     resources: None,
//!     prompts: None,
//! };
//! ```
//!
//! # JSON-RPC Compliance
//!
//! All types are fully compliant with JSON-RPC 2.0 specification and include
//! proper error handling, request/response correlation, and notification support.

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 version identifier
pub const JSONRPC_VERSION: &str = "2.0";

/// Unique identifier for JSON-RPC requests
pub type RequestId = serde_json::Value;

/// Core MCP message variants following JSON-RPC 2.0 specification
///
/// These are the three fundamental message types in MCP communication.
/// Messages are automatically serialized/deserialized when sent over transports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MCPMessage {
    /// A request message initiating an operation
    Request(MCPRequest),
    /// A response message containing results or errors
    Response(MCPResponse),
    /// A notification message (request without expecting response)
    Notification(MCPNotification),
}

/// Request message for initiating MCP operations
///
/// Requests expect a response from the server. Each request has a unique ID
/// for correlating with the corresponding response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MCPRequest {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Unique identifier for this request
    pub id: RequestId,
    /// Method name to invoke
    pub method: String,
    /// Optional parameters for the method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Response message containing operation results or errors
///
/// Responses correspond to requests via the ID field. They contain either
/// successful results or detailed error information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MCPResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Request identifier this response corresponds to
    pub id: RequestId,
    /// Either result or error, but not both
    #[serde(flatten)]
    pub payload: MCPResponsePayload,
}

/// Response content - either successful result data or error details
///
/// This ensures responses contain exactly one of: result or error, never both.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MCPResponsePayload {
    /// Successful response with result data
    Success { result: serde_json::Value },
    /// Error response with error details
    Error { error: MCPError },
}

/// Notification message for events that don't expect responses
///
/// Notifications are used for server-initiated events like tool list changes
/// or progress updates. Clients should not respond to notifications.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MCPNotification {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,
    /// Method name to invoke
    pub method: String,
    /// Optional parameters for the method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Error information following JSON-RPC 2.0 error format
///
/// Provides structured error reporting with standard codes and optional
/// additional data for debugging and error handling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MCPError {
    /// Numeric error code
    pub code: i32,
    /// Human-readable error message
    pub message: String,
    /// Optional additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Standard JSON-RPC 2.0 error codes
#[allow(dead_code)]
impl MCPError {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Create a parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self {
            code: Self::PARSE_ERROR,
            message: message.into(),
            data: None,
        }
    }

    /// Create an invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            code: Self::INVALID_REQUEST,
            message: message.into(),
            data: None,
        }
    }

    /// Create a method not found error
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self {
            code: Self::METHOD_NOT_FOUND,
            message: format!("Method not found: {}", method.into()),
            data: None,
        }
    }

    /// Create an invalid parameters error
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: Self::INVALID_PARAMS,
            message: message.into(),
            data: None,
        }
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            code: Self::INTERNAL_ERROR,
            message: message.into(),
            data: None,
        }
    }
}

/// Server feature support advertised during connection handshake
///
/// Servers declare which MCP features they support. Clients use this
/// information to determine available functionality.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Whether the server supports listing and calling tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    /// Whether the server supports listing and reading resources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    /// Whether the server supports listing and getting prompts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
}

/// Tool-related server capabilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolsCapability {
    /// Whether the server supports listing available tools
    #[serde(skip_serializing_if = "Option::is_none", rename = "listChanged")]
    pub list_changed: Option<bool>,
}

/// Resource-related server capabilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourcesCapability {
    /// Whether the server supports subscribing to resource changes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    /// Whether the server supports listing available resources
    #[serde(skip_serializing_if = "Option::is_none", rename = "listChanged")]
    pub list_changed: Option<bool>,
}

/// Prompt-related server capabilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptsCapability {
    /// Whether the server supports listing available prompts
    #[serde(skip_serializing_if = "Option::is_none", rename = "listChanged")]
    pub list_changed: Option<bool>,
}

/// Client feature support declared during connection handshake
///
/// Clients advertise their capabilities to servers. This enables servers
/// to use advanced features when the client supports them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ClientCapabilities {
    /// Whether the client supports sampling operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<SamplingCapability>,
    /// Whether the client supports root directory operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootsCapability>,
}

/// Sampling-related client capabilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SamplingCapability {}

/// Root directory-related client capabilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RootsCapability {
    /// Whether the client supports listing root directories
    #[serde(skip_serializing_if = "Option::is_none", rename = "listChanged")]
    pub list_changed: Option<bool>,
}

/// Content formats for tool results and resource data
///
/// MCP supports multiple content types to handle different kinds of data
/// that tools and resources might provide.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    /// Plain text content
    #[serde(rename = "text")]
    Text(TextContent),
    /// Image content with base64 data
    #[serde(rename = "image")]
    Image(ImageContent),
    /// Resource reference content
    #[serde(rename = "resource")]
    Resource(ResourceContent),
}

/// Text content type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextContent {
    /// The text content
    pub text: String,
}

/// Image content type with base64 encoding
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageContent {
    /// Base64-encoded image data
    pub data: String,
    /// MIME type of the image (e.g., "image/png")
    #[serde(rename = "mimeType")]
    pub mime_type: String,
}

/// Resource content type for referencing external resources
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceContent {
    /// URI of the resource
    pub uri: String,
    /// Optional MIME type of the resource
    #[serde(skip_serializing_if = "Option::is_none", rename = "mimeType")]
    pub mime_type: Option<String>,
    /// Optional text content of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Tool metadata including name, description, and parameter schema
///
/// Servers provide tool definitions during discovery. The input schema
/// defines the JSON structure expected for tool parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    /// Name of the tool
    pub name: String,
    /// Description of what the tool does
    pub description: String,
    /// JSON schema for the tool's input parameters
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// Tool execution results containing content and error status
///
/// Tools return content (text, images, resources) and may indicate
/// whether the execution resulted in an error condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallToolResult {
    /// Content returned by the tool
    pub content: Vec<Content>,
    /// Whether the tool call resulted in an error
    #[serde(skip_serializing_if = "Option::is_none", rename = "isError")]
    pub is_error: Option<bool>,
}

/// Progress reporting for operations that may take significant time
///
/// Servers can send progress notifications to keep clients informed
/// about long-running tool executions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Progress {
    /// Current progress value
    pub progress: u64,
    /// Total expected value (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

/// Initialize request parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitializeRequest {
    /// Protocol version requested by client
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Client capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<serde_json::Value>,
    /// Information about the client
    #[serde(rename = "clientInfo")]
    pub client_info: serde_json::Value,
}

/// Initialize response result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Protocol version supported by server
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Server capabilities
    pub capabilities: ServerCapabilities,
    /// Information about the server
    #[serde(rename = "serverInfo")]
    pub server_info: serde_json::Value,
    /// Optional setup instructions for the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

/// List tools request parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListToolsRequest {
    // No parameters currently defined for list tools
}

/// List tools response result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListToolsResult {
    /// Available tools
    pub tools: Vec<Tool>,
}

/// Call tool request parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallToolRequest {
    /// Name of the tool to call
    pub name: String,
    /// Arguments to pass to the tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}

/// Standard MCP method names as constants
///
/// These constants ensure consistent method naming across the implementation
/// and provide compile-time checking for method names.
pub mod methods {
    /// Initialize the MCP session
    pub const INITIALIZE: &str = "initialize";
    /// List available tools
    pub const LIST_TOOLS: &str = "tools/list";
    /// Call a specific tool
    pub const CALL_TOOL: &str = "tools/call";
    /// List available resources
    pub const LIST_RESOURCES: &str = "resources/list";
    /// Read a specific resource
    pub const READ_RESOURCE: &str = "resources/read";
    /// List available prompts
    pub const LIST_PROMPTS: &str = "prompts/list";
    /// Get a specific prompt
    pub const GET_PROMPT: &str = "prompts/get";
    /// Complete text using sampling
    pub const COMPLETE: &str = "completion/complete";
    /// List client roots
    pub const LIST_ROOTS: &str = "roots/list";
    /// Notification for tool list changes
    pub const TOOLS_LIST_CHANGED: &str = "notifications/tools/list_changed";
    /// Notification for resource list changes
    pub const RESOURCES_LIST_CHANGED: &str = "notifications/resources/list_changed";
    /// Notification for prompt list changes
    pub const PROMPTS_LIST_CHANGED: &str = "notifications/prompts/list_changed";
    /// Notification for roots list changes
    pub const ROOTS_LIST_CHANGED: &str = "notifications/roots/list_changed";
    /// Progress notification
    pub const PROGRESS: &str = "notifications/progress";
}

impl MCPRequest {
    /// Create a new request with the given method and parameters
    pub fn new(
        id: RequestId,
        method: impl Into<String>,
        params: Option<serde_json::Value>,
    ) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

impl MCPResponse {
    /// Create a successful response
    pub fn success(id: RequestId, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            payload: MCPResponsePayload::Success { result },
        }
    }

    /// Create an error response
    pub fn error(id: RequestId, error: MCPError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            payload: MCPResponsePayload::Error { error },
        }
    }
}

impl MCPNotification {
    /// Create a new notification
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.into(),
            params,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mcp_request_serialization() {
        let request = MCPRequest::new(json!(1), "test_method", Some(json!({"param": "value"})));

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: MCPRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request, deserialized);
        assert_eq!(request.jsonrpc, JSONRPC_VERSION);
        assert_eq!(request.method, "test_method");
    }

    #[test]
    fn test_mcp_response_success() {
        let response = MCPResponse::success(json!(1), json!({"result": "success"}));

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: MCPResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response, deserialized);
        assert!(matches!(
            response.payload,
            MCPResponsePayload::Success { .. }
        ));
    }

    #[test]
    fn test_mcp_response_error() {
        let error = MCPError::method_not_found("test_method");
        let response = MCPResponse::error(json!(1), error);

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: MCPResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response, deserialized);
        assert!(matches!(response.payload, MCPResponsePayload::Error { .. }));
    }

    #[test]
    fn test_content_types_serialization() {
        let text_content = Content::Text(TextContent {
            text: "Hello, world!".to_string(),
        });

        let image_content = Content::Image(ImageContent {
            data: "base64data".to_string(),
            mime_type: "image/png".to_string(),
        });

        let resource_content = Content::Resource(ResourceContent {
            uri: "file://test.txt".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: Some("file content".to_string()),
        });

        // Test serialization and deserialization
        for content in [text_content, image_content, resource_content] {
            let serialized = serde_json::to_string(&content).unwrap();
            let deserialized: Content = serde_json::from_str(&serialized).unwrap();
            assert_eq!(content, deserialized);
        }
    }

    #[test]
    fn test_capabilities_serialization() {
        let server_caps = ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            prompts: None,
        };

        let client_caps = ClientCapabilities {
            sampling: Some(SamplingCapability {}),
            roots: Some(RootsCapability {
                list_changed: Some(true),
            }),
        };

        // Test serialization and deserialization
        let server_serialized = serde_json::to_string(&server_caps).unwrap();
        let server_deserialized: ServerCapabilities =
            serde_json::from_str(&server_serialized).unwrap();
        assert_eq!(server_caps, server_deserialized);

        let client_serialized = serde_json::to_string(&client_caps).unwrap();
        let client_deserialized: ClientCapabilities =
            serde_json::from_str(&client_serialized).unwrap();
        assert_eq!(client_caps, client_deserialized);
    }

    #[test]
    fn test_mcp_message_variants() {
        let request = MCPMessage::Request(MCPRequest::new(json!(1), "test", None));

        let response =
            MCPMessage::Response(MCPResponse::success(json!(1), json!({"status": "ok"})));

        let notification = MCPMessage::Notification(MCPNotification::new(
            "test_notification",
            Some(json!({"data": "test"})),
        ));

        // Test serialization and deserialization
        for message in [request, response, notification] {
            let serialized = serde_json::to_string(&message).unwrap();
            let deserialized: MCPMessage = serde_json::from_str(&serialized).unwrap();
            assert_eq!(message, deserialized);
        }
    }

    #[test]
    fn test_error_constructors() {
        let parse_err = MCPError::parse_error("Invalid JSON");
        assert_eq!(parse_err.code, MCPError::PARSE_ERROR);
        assert_eq!(parse_err.message, "Invalid JSON");

        let method_err = MCPError::method_not_found("unknown_method");
        assert_eq!(method_err.code, MCPError::METHOD_NOT_FOUND);
        assert!(method_err.message.contains("unknown_method"));

        let params_err = MCPError::invalid_params("Missing required parameter");
        assert_eq!(params_err.code, MCPError::INVALID_PARAMS);
        assert_eq!(params_err.message, "Missing required parameter");
    }
}
