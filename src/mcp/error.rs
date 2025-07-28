//! Comprehensive error handling for MCP operations
//!
//! This module provides detailed error types that help you understand and handle
//! failures in MCP communication. You'll get specific error categories for transport
//! issues, protocol violations, tool execution problems, and session management.
//!
//! # Error Categories
//!
//! MCP errors are organized into logical categories:
//!
//! - **Transport Errors** - Network, WebSocket, and process communication failures
//! - **Protocol Errors** - JSON-RPC violations and MCP specification issues
//! - **Tool Errors** - Tool discovery and execution failures
//! - **Session Errors** - Connection state and capability negotiation problems
//!
//! # Usage Patterns
//!
//! Check error types for appropriate handling:
//! ```rust
//! use stood::mcp::error::MCPOperationError;
//!
//! match error {
//!     MCPOperationError::TimeoutError { duration } => {
//!         println!("Operation timed out after {:?}, retrying...", duration);
//!         // Implement retry logic
//!     }
//!     MCPOperationError::ToolExecutionError { tool_name, message } => {
//!         eprintln!("Tool '{}' failed: {}", tool_name, message);
//!         // Handle tool-specific error
//!     }
//!     MCPOperationError::ConnectionLostError { .. } => {
//!         println!("Connection lost, attempting reconnection...");
//!         // Reconnection logic
//!     }
//!     _ => {
//!         eprintln!("Unexpected error: {}", error);
//!     }
//! }
//! ```
//!
//! # Error Classification
//!
//! Errors include helper methods for classification:
//!
//! ```rust
//! # use stood::mcp::error::MCPOperationError;
//! # let error = MCPOperationError::transport("test");
//! if error.is_retryable() {
//!     // Safe to retry this operation
//! }
//!
//! if error.is_connection_error() {
//!     // Connection-related issue, may need reconnection
//! }
//!
//! if error.is_timeout() {
//!     // Operation took too long, adjust timeouts or retry
//! }
//! ```

use std::time::Duration;
use thiserror::Error;

/// Primary error type for all MCP operations
///
/// This enum covers all possible failure modes in MCP communication.
/// Each variant provides specific context about what went wrong and
/// includes helper methods for error classification and handling.
#[derive(Error, Debug, Clone)]
pub enum MCPOperationError {
    /// Transport-level connection errors
    #[error("Transport error: {message}")]
    TransportError { message: String },

    /// WebSocket-specific connection errors
    #[error("WebSocket error: {message}")]
    WebSocketError { message: String },

    /// Stdio transport errors (process communication)
    #[error("Stdio transport error: {message}")]
    StdioError { message: String },

    /// JSON-RPC protocol errors
    #[error("Protocol error: {message}")]
    ProtocolError { message: String },

    /// Message serialization/deserialization errors
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Connection initialization failures
    #[error("Initialization error: {message}")]
    InitializationError { message: String },

    /// Connection timeout errors
    #[error("Timeout error: operation timed out after {duration:?}")]
    TimeoutError { duration: Duration },

    /// MCP server capability negotiation failures
    #[error("Capability negotiation failed: {message}")]
    CapabilityError { message: String },

    /// Tool discovery errors
    #[error("Tool discovery error: {message}")]
    ToolDiscoveryError { message: String },

    /// Tool execution errors from MCP servers
    #[error("Tool execution error: {tool_name}: {message}")]
    ToolExecutionError { tool_name: String, message: String },

    /// Invalid tool arguments or schemas
    #[error("Invalid tool arguments for {tool_name}: {message}")]
    InvalidToolArguments { tool_name: String, message: String },

    /// MCP session management errors
    #[error("Session error: {message}")]
    SessionError { message: String },

    /// Connection state errors (not connected, already connected, etc.)
    #[error("Connection state error: {message}")]
    ConnectionStateError { message: String },

    /// Authentication/authorization errors
    #[error("Authentication error: {message}")]
    AuthenticationError { message: String },

    /// Rate limiting from MCP servers
    #[error("Rate limited: {message}")]
    RateLimitError { message: String },

    /// Server-side errors reported by MCP servers
    #[error("Server error: {message}")]
    ServerError { message: String },

    /// Invalid MCP server responses
    #[error("Invalid response: {message}")]
    InvalidResponseError { message: String },

    /// Connection lost/disconnected errors
    #[error("Connection lost: {message}")]
    ConnectionLostError { message: String },
}

/// Transport layer errors for network and process communication
///
/// These errors occur at the transport level before messages reach
/// the MCP protocol layer. They typically indicate infrastructure
/// or configuration problems.
#[derive(Error, Debug, Clone)]
pub enum TransportError {
    /// Network connectivity issues
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// DNS resolution failures
    #[error("DNS resolution failed: {hostname}")]
    DnsError { hostname: String },

    /// TLS/SSL certificate errors
    #[error("TLS error: {message}")]
    TlsError { message: String },

    /// Process spawn errors for stdio transport
    #[error("Process spawn failed: {command}: {message}")]
    ProcessSpawnError { command: String, message: String },

    /// Process communication errors
    #[error("Process communication error: {message}")]
    ProcessCommunicationError { message: String },

    /// Process unexpected exit
    #[error("Process exited unexpectedly: exit code {}", code.map_or("unknown".to_string(), |c| c.to_string()))]
    ProcessExitError { code: Option<i32> },

    /// Invalid transport configuration
    #[error("Invalid transport config: {message}")]
    ConfigurationError { message: String },

    /// Connection refused by server
    #[error("Connection refused: {address}")]
    ConnectionRefused { address: String },

    /// Connection timeout during handshake
    #[error("Connection timeout: {timeout:?}")]
    ConnectionTimeout { timeout: Duration },
}

/// JSON-RPC protocol errors following the official specification
///
/// These errors occur when messages violate the JSON-RPC 2.0 standard
/// or when servers report protocol-level failures.
#[derive(Error, Debug, Clone)]
pub enum JsonRpcError {
    /// Parse error - Invalid JSON received
    #[error("Parse error: {message}")]
    ParseError { message: String },

    /// Invalid request - JSON is not a valid Request object
    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },

    /// Method not found - Method does not exist
    #[error("Method not found: {method}")]
    MethodNotFound { method: String },

    /// Invalid parameters - Invalid method parameters
    #[error("Invalid params: {message}")]
    InvalidParams { message: String },

    /// Internal error - Internal JSON-RPC error
    #[error("Internal error: {message}")]
    InternalError { message: String },

    /// Server error - Reserved for implementation-defined server errors
    #[error("Server error: {code}: {message}")]
    ServerError { code: i32, message: String },
}

/// Session lifecycle and state management errors
///
/// These errors occur during session initialization, capability negotiation,
/// or when operations are attempted in invalid session states.
#[derive(Error, Debug, Clone)]
pub enum SessionError {
    /// Session not initialized
    #[error("Session not initialized")]
    NotInitialized,

    /// Session already initialized
    #[error("Session already initialized")]
    AlreadyInitialized,

    /// Session terminated
    #[error("Session terminated: {reason}")]
    Terminated { reason: String },

    /// Invalid session state for operation
    #[error("Invalid session state: {message}")]
    InvalidState { message: String },

    /// Session capability mismatch
    #[error("Capability mismatch: {message}")]
    CapabilityMismatch { message: String },
}

impl MCPOperationError {
    /// Create a transport error
    pub fn transport(message: impl Into<String>) -> Self {
        Self::TransportError {
            message: message.into(),
        }
    }

    /// Create a WebSocket error
    pub fn websocket(message: impl Into<String>) -> Self {
        Self::WebSocketError {
            message: message.into(),
        }
    }

    /// Create a stdio transport error
    pub fn stdio(message: impl Into<String>) -> Self {
        Self::StdioError {
            message: message.into(),
        }
    }

    /// Create a protocol error
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::ProtocolError {
            message: message.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::SerializationError {
            message: message.into(),
        }
    }

    /// Create an initialization error
    pub fn initialization(message: impl Into<String>) -> Self {
        Self::InitializationError {
            message: message.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(duration: Duration) -> Self {
        Self::TimeoutError { duration }
    }

    /// Create a capability error
    pub fn capability(message: impl Into<String>) -> Self {
        Self::CapabilityError {
            message: message.into(),
        }
    }

    /// Create a tool discovery error
    pub fn tool_discovery(message: impl Into<String>) -> Self {
        Self::ToolDiscoveryError {
            message: message.into(),
        }
    }

    /// Create a tool execution error
    pub fn tool_execution(tool_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ToolExecutionError {
            tool_name: tool_name.into(),
            message: message.into(),
        }
    }

    /// Create an invalid tool arguments error
    pub fn invalid_tool_arguments(
        tool_name: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::InvalidToolArguments {
            tool_name: tool_name.into(),
            message: message.into(),
        }
    }

    /// Create a session error
    pub fn session(message: impl Into<String>) -> Self {
        Self::SessionError {
            message: message.into(),
        }
    }

    /// Create a connection state error
    pub fn connection_state(message: impl Into<String>) -> Self {
        Self::ConnectionStateError {
            message: message.into(),
        }
    }

    /// Create an authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::AuthenticationError {
            message: message.into(),
        }
    }

    /// Create a rate limit error
    pub fn rate_limit(message: impl Into<String>) -> Self {
        Self::RateLimitError {
            message: message.into(),
        }
    }

    /// Create a server error
    pub fn server(message: impl Into<String>) -> Self {
        Self::ServerError {
            message: message.into(),
        }
    }

    /// Create an invalid response error
    pub fn invalid_response(message: impl Into<String>) -> Self {
        Self::InvalidResponseError {
            message: message.into(),
        }
    }

    /// Create a connection lost error
    pub fn connection_lost(message: impl Into<String>) -> Self {
        Self::ConnectionLostError {
            message: message.into(),
        }
    }

    /// Determine if this error indicates a retryable condition
    ///
    /// Returns `true` for transient errors like network timeouts or temporary
    /// server failures. Returns `false` for permanent errors like authentication
    /// failures or invalid configurations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use stood::mcp::error::MCPOperationError;
    /// # use std::time::Duration;
    /// let timeout_error = MCPOperationError::timeout(Duration::from_secs(30));
    /// assert!(timeout_error.is_retryable());
    ///
    /// let auth_error = MCPOperationError::authentication("Invalid token");
    /// assert!(!auth_error.is_retryable());
    /// ```
    pub fn is_retryable(&self) -> bool {
        match self {
            // Network/transport errors are generally retryable
            Self::TransportError { .. }
            | Self::WebSocketError { .. }
            | Self::TimeoutError { .. }
            | Self::ConnectionLostError { .. }
            | Self::RateLimitError { .. } => true,

            // Server errors might be retryable depending on the code
            Self::ServerError { .. } => true,

            // Protocol and configuration errors are not retryable
            Self::ProtocolError { .. }
            | Self::SerializationError { .. }
            | Self::InvalidToolArguments { .. }
            | Self::AuthenticationError { .. }
            | Self::InvalidResponseError { .. }
            | Self::CapabilityError { .. } => false,

            // Other errors depend on context
            _ => false,
        }
    }

    /// Determine if this error indicates a connection issue
    ///
    /// Returns `true` for errors related to establishing or maintaining
    /// connections. These errors often require reconnection logic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use stood::mcp::error::MCPOperationError;
    /// let connection_error = MCPOperationError::connection_lost("Socket closed");
    /// assert!(connection_error.is_connection_error());
    ///
    /// let tool_error = MCPOperationError::tool_execution("calc", "Division by zero");
    /// assert!(!tool_error.is_connection_error());
    /// ```
    pub fn is_connection_error(&self) -> bool {
        matches!(
            self,
            Self::TransportError { .. }
                | Self::WebSocketError { .. }
                | Self::StdioError { .. }
                | Self::InitializationError { .. }
                | Self::ConnectionStateError { .. }
                | Self::ConnectionLostError { .. }
        )
    }

    /// Determine if this error represents a timeout condition
    ///
    /// Returns `true` only for timeout errors, which may indicate the need
    /// to adjust timeout configurations or retry with longer timeouts.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use stood::mcp::error::MCPOperationError;
    /// # use std::time::Duration;
    /// let timeout_error = MCPOperationError::timeout(Duration::from_secs(30));
    /// assert!(timeout_error.is_timeout());
    /// ```
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::TimeoutError { .. })
    }

    /// Create a JSON-RPC invalid parameters error
    pub fn json_rpc_invalid_params(message: impl Into<String>) -> Self {
        Self::ProtocolError {
            message: format!("Invalid params: {}", message.into()),
        }
    }

    /// Create a JSON-RPC method not found error
    pub fn json_rpc_method_not_found(message: impl Into<String>) -> Self {
        Self::ProtocolError {
            message: format!("Method not found: {}", message.into()),
        }
    }

    /// Create a JSON-RPC internal error
    pub fn json_rpc_internal_error(message: impl Into<String>) -> Self {
        Self::ProtocolError {
            message: format!("Internal error: {}", message.into()),
        }
    }
}

impl TransportError {
    /// Create a network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }

    /// Create a DNS error
    pub fn dns(hostname: impl Into<String>) -> Self {
        Self::DnsError {
            hostname: hostname.into(),
        }
    }

    /// Create a TLS error
    pub fn tls(message: impl Into<String>) -> Self {
        Self::TlsError {
            message: message.into(),
        }
    }

    /// Create a process spawn error
    pub fn process_spawn(command: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ProcessSpawnError {
            command: command.into(),
            message: message.into(),
        }
    }

    /// Create a process communication error
    pub fn process_communication(message: impl Into<String>) -> Self {
        Self::ProcessCommunicationError {
            message: message.into(),
        }
    }

    /// Create a process exit error
    pub fn process_exit(code: Option<i32>) -> Self {
        Self::ProcessExitError { code }
    }

    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::ConfigurationError {
            message: message.into(),
        }
    }

    /// Create a connection refused error
    pub fn connection_refused(address: impl Into<String>) -> Self {
        Self::ConnectionRefused {
            address: address.into(),
        }
    }

    /// Create a connection timeout error
    pub fn connection_timeout(timeout: Duration) -> Self {
        Self::ConnectionTimeout { timeout }
    }
}

impl JsonRpcError {
    /// Create a parse error
    pub fn parse(message: impl Into<String>) -> Self {
        Self::ParseError {
            message: message.into(),
        }
    }

    /// Create an invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest {
            message: message.into(),
        }
    }

    /// Create a method not found error
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self::MethodNotFound {
            method: method.into(),
        }
    }

    /// Create an invalid params error
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::InvalidParams {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }

    /// Create a server error
    pub fn server(code: i32, message: impl Into<String>) -> Self {
        Self::ServerError {
            code,
            message: message.into(),
        }
    }
}

impl SessionError {
    /// Create a not initialized error
    pub fn not_initialized() -> Self {
        Self::NotInitialized
    }

    /// Create an already initialized error
    pub fn already_initialized() -> Self {
        Self::AlreadyInitialized
    }

    /// Create a terminated error
    pub fn terminated(reason: impl Into<String>) -> Self {
        Self::Terminated {
            reason: reason.into(),
        }
    }

    /// Create an invalid state error
    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::InvalidState {
            message: message.into(),
        }
    }

    /// Create a capability mismatch error
    pub fn capability_mismatch(message: impl Into<String>) -> Self {
        Self::CapabilityMismatch {
            message: message.into(),
        }
    }
}

// Convert transport errors to MCP errors
impl From<TransportError> for MCPOperationError {
    fn from(err: TransportError) -> Self {
        Self::TransportError {
            message: err.to_string(),
        }
    }
}

// Convert JSON-RPC errors to MCP errors
impl From<JsonRpcError> for MCPOperationError {
    fn from(err: JsonRpcError) -> Self {
        Self::ProtocolError {
            message: err.to_string(),
        }
    }
}

// Convert session errors to MCP errors
impl From<SessionError> for MCPOperationError {
    fn from(err: SessionError) -> Self {
        Self::SessionError {
            message: err.to_string(),
        }
    }
}

// Convert serde_json errors to MCP errors
impl From<serde_json::Error> for MCPOperationError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError {
            message: err.to_string(),
        }
    }
}

// Convert to main Stood error type
impl From<MCPOperationError> for crate::StoodError {
    fn from(err: MCPOperationError) -> Self {
        match err {
            MCPOperationError::TransportError { message }
            | MCPOperationError::WebSocketError { message }
            | MCPOperationError::StdioError { message }
            | MCPOperationError::ConnectionLostError { message } => {
                Self::ConfigurationError { message }
            }

            MCPOperationError::ToolExecutionError { tool_name, message }
            | MCPOperationError::InvalidToolArguments { tool_name, message } => Self::ToolError {
                message: format!("{}: {}", tool_name, message),
            },

            MCPOperationError::ToolDiscoveryError { message } => Self::ToolError { message },

            MCPOperationError::AuthenticationError { message } => Self::AccessDenied { message },

            MCPOperationError::RateLimitError { message } => Self::ThrottlingError { message },

            // Default mapping for other errors
            _ => Self::InvalidInput {
                message: err.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_mcp_error_constructors() {
        let transport_err = MCPOperationError::transport("Connection failed");
        assert!(matches!(
            transport_err,
            MCPOperationError::TransportError { .. }
        ));

        let timeout_err = MCPOperationError::timeout(Duration::from_secs(30));
        assert!(matches!(
            timeout_err,
            MCPOperationError::TimeoutError { .. }
        ));

        let tool_err = MCPOperationError::tool_execution("calculator", "Division by zero");
        assert!(matches!(
            tool_err,
            MCPOperationError::ToolExecutionError { .. }
        ));
    }

    #[test]
    fn test_error_classification() {
        let retryable = MCPOperationError::transport("Network issue");
        assert!(retryable.is_retryable());
        assert!(retryable.is_connection_error());

        let not_retryable = MCPOperationError::authentication("Invalid token");
        assert!(!not_retryable.is_retryable());
        assert!(!not_retryable.is_connection_error());

        let timeout = MCPOperationError::timeout(Duration::from_secs(10));
        assert!(timeout.is_timeout());
        assert!(timeout.is_retryable());
    }

    #[test]
    fn test_transport_error_constructors() {
        let network_err = TransportError::network("DNS failure");
        assert!(matches!(network_err, TransportError::NetworkError { .. }));

        let process_err = TransportError::process_spawn("python", "File not found");
        assert!(matches!(
            process_err,
            TransportError::ProcessSpawnError { .. }
        ));
    }

    #[test]
    fn test_jsonrpc_error_constructors() {
        let parse_err = JsonRpcError::parse("Invalid JSON");
        assert!(matches!(parse_err, JsonRpcError::ParseError { .. }));

        let method_err = JsonRpcError::method_not_found("unknown_method");
        assert!(matches!(method_err, JsonRpcError::MethodNotFound { .. }));
    }

    #[test]
    fn test_session_error_constructors() {
        let not_init = SessionError::not_initialized();
        assert!(matches!(not_init, SessionError::NotInitialized));

        let terminated = SessionError::terminated("Server shutdown");
        assert!(matches!(terminated, SessionError::Terminated { .. }));
    }

    #[test]
    fn test_error_conversions() {
        let transport_err = TransportError::network("Connection failed");
        let mcp_err: MCPOperationError = transport_err.into();
        assert!(matches!(mcp_err, MCPOperationError::TransportError { .. }));

        let session_err = SessionError::not_initialized();
        let mcp_err: MCPOperationError = session_err.into();
        assert!(matches!(mcp_err, MCPOperationError::SessionError { .. }));

        let stood_err: crate::StoodError = MCPOperationError::authentication("Bad token").into();
        assert!(matches!(stood_err, crate::StoodError::AccessDenied { .. }));
    }

    #[test]
    fn test_error_display() {
        let err = MCPOperationError::tool_execution("calculator", "Division by zero");
        let display = format!("{}", err);
        assert!(display.contains("calculator"));
        assert!(display.contains("Division by zero"));

        let timeout_err = MCPOperationError::timeout(Duration::from_secs(30));
        let display = format!("{}", timeout_err);
        assert!(display.contains("30s"));
    }
}
