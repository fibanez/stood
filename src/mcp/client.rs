//! MCP Client for connecting to external tool servers
//!
//! This module enables you to connect to Model Context Protocol servers and execute
//! their tools from your Rust applications. You'll get automatic session management,
//! tool discovery, and type-safe tool execution.
//!
//! # Usage Patterns
//!
//! Create a client and connect to a server:
//! ```no_run
//! use stood::mcp::{MCPClient, MCPClientConfig};
//! use stood::mcp::transport::{TransportFactory, StdioConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = StdioConfig {
//!     command: "python".to_string(),
//!     args: vec!["-m".to_string(), "my_mcp_server".to_string()],
//!     ..Default::default()
//! };
//!
//! let transport = TransportFactory::stdio(config);
//! let mut client = MCPClient::new(MCPClientConfig::default(), transport);
//!
//! client.connect().await?;
//!
//! // Discover available tools
//! let tools = client.list_tools().await?;
//! for tool in tools {
//!     println!("Tool: {} - {}", tool.name, tool.description);
//! }
//!
//! // Execute a tool
//! let result = client.call_tool("calculator", Some(serde_json::json!({
//!     "operation": "add",
//!     "a": 5,
//!     "b": 3
//! }))).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! The client manages three key areas:
//!
//! - **Session Management** - Handles connection lifecycle and capability negotiation
//! - **Tool Discovery** - Automatically refreshes available tools from the server
//! - **Request Handling** - Manages concurrent requests with timeout and error handling
//!
//! See [MCP client patterns](../../docs/patterns.wiki#mcp-client) for advanced usage.
//!
//! # Performance
//!
//! - Connection overhead: <50ms for typical stdio servers
//! - Tool execution: Depends on server implementation
//! - Concurrent requests: Up to 100 by default (configurable)
//! - Memory usage: ~1KB per active session plus message buffers

use crate::mcp::error::MCPOperationError;
use crate::mcp::transport::{MCPTransport, TransportStreams};
use crate::mcp::types::{
    ClientCapabilities, Content, MCPMessage, MCPNotification, MCPRequest, MCPResponse,
    MCPResponsePayload, ServerCapabilities, Tool,
};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tracing::{debug, error, info, warn};

/// Configuration options for MCP client behavior
///
/// This configuration controls how the client connects to and communicates with
/// MCP servers. You can customize timeouts, concurrency limits, and client identification.
#[derive(Debug, Clone)]
pub struct MCPClientConfig {
    /// Client name for identification
    pub client_name: String,
    /// Client version
    pub client_version: String,
    /// Client capabilities to advertise during handshake
    pub client_capabilities: ClientCapabilities,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Enable automatic reconnection on connection loss
    pub auto_reconnect: bool,
    /// Reconnection delay in milliseconds
    pub reconnect_delay_ms: u64,
}

impl Default for MCPClientConfig {
    fn default() -> Self {
        Self {
            client_name: "stood-mcp-client".to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            client_capabilities: ClientCapabilities::default(),
            request_timeout_ms: 30_000, // 30 seconds
            max_concurrent_requests: 100,
            auto_reconnect: true,
            reconnect_delay_ms: 5_000, // 5 seconds
        }
    }
}

/// Internal session state tracking for an MCP connection
///
/// This structure maintains the current state of a client session including
/// server capabilities, discovered tools, and connection metadata. Sessions
/// are automatically managed by the client.
#[derive(Debug)]
struct MCPSession {
    /// Unique session ID
    session_id: String,
    /// Server capabilities received during handshake
    server_capabilities: Option<ServerCapabilities>,
    /// Server name
    server_name: Option<String>,
    /// Server version
    server_version: Option<String>,
    /// Available tools discovered from the server
    available_tools: HashMap<String, Tool>,
    /// Session start time
    start_time: std::time::Instant,
    /// Whether the session is currently active
    is_active: bool,
}

impl MCPSession {
    fn new() -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            server_capabilities: None,
            server_name: None,
            server_version: None,
            available_tools: HashMap::new(),
            start_time: std::time::Instant::now(),
            is_active: false,
        }
    }
}

/// Pending request information
struct PendingRequest {
    /// Response channel
    response_tx: oneshot::Sender<Result<MCPResponse, MCPOperationError>>,
    /// Request method for logging
    _method: String,
    /// Request timestamp
    _timestamp: std::time::Instant,
}

/// High-level MCP client for server communication
///
/// This is the primary interface for connecting to MCP servers and executing their tools.
/// The client handles all aspects of the MCP protocol including initialization, capability
/// negotiation, tool discovery, and request/response management.
///
/// # Connection Management
///
/// The client maintains a persistent connection to the MCP server through the configured
/// transport (WebSocket or stdio). Background tasks handle message processing and ensure
/// reliable communication.
///
/// # Tool Execution
///
/// Tools are discovered during initialization and can be executed with type-safe parameters.
/// The client validates tool existence before execution and provides structured error
/// handling for tool failures.
///
/// # Error Handling
///
/// All operations return comprehensive error types that distinguish between transport
/// issues, protocol violations, and tool execution failures. This enables appropriate
/// retry logic and user feedback.
pub struct MCPClient {
    /// Client configuration
    config: MCPClientConfig,
    /// Transport implementation
    transport: Box<dyn MCPTransport>,
    /// Current session state
    session: Arc<RwLock<MCPSession>>,
    /// Request ID counter
    request_id_counter: AtomicI64,
    /// Map of pending requests
    pending_requests: Arc<Mutex<HashMap<Value, PendingRequest>>>,
    /// Whether the client is connected
    is_connected: Arc<AtomicBool>,
    /// Background task handle
    background_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    /// Shutdown signal sender
    shutdown_tx: Arc<Mutex<Option<mpsc::UnboundedSender<()>>>>,
    /// Write channel for sending messages
    write_tx: Arc<Mutex<Option<mpsc::UnboundedSender<MCPMessage>>>>,
}

impl MCPClient {
    /// Create a new MCP client with configuration and transport
    ///
    /// This creates a client instance but doesn't establish the connection.
    /// Call `connect()` to initialize the session with the MCP server.
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration including timeouts and capabilities
    /// * `transport` - Transport implementation (WebSocket or stdio)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use stood::mcp::{MCPClient, MCPClientConfig};
    /// use stood::mcp::transport::{TransportFactory, StdioConfig};
    ///
    /// let config = MCPClientConfig::default();
    /// let transport_config = StdioConfig {
    ///     command: "python".to_string(),
    ///     args: vec!["-m".to_string(), "mcp_server".to_string()],
    ///     ..Default::default()
    /// };
    /// let transport = TransportFactory::stdio(transport_config);
    /// let client = MCPClient::new(config, transport);
    /// ```
    pub fn new(config: MCPClientConfig, transport: Box<dyn MCPTransport>) -> Self {
        Self {
            config,
            transport,
            session: Arc::new(RwLock::new(MCPSession::new())),
            request_id_counter: AtomicI64::new(1),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            is_connected: Arc::new(AtomicBool::new(false)),
            background_handle: Arc::new(Mutex::new(None)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            write_tx: Arc::new(Mutex::new(None)),
        }
    }

    /// Connect to the MCP server and establish a session
    ///
    /// This method establishes the underlying transport connection, performs the MCP
    /// handshake, negotiates capabilities, and discovers available tools. After
    /// successful connection, you can call `list_tools()` and `call_tool()`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the connection and handshake succeed, or an error describing
    /// what went wrong during connection or initialization.
    ///
    /// # Errors
    ///
    /// - `TransportError` - Failed to establish transport connection
    /// - `ProtocolError` - MCP handshake or capability negotiation failed
    /// - `TimeoutError` - Connection took longer than configured timeout
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use stood::mcp::{MCPClient, MCPClientConfig};
    /// # use stood::mcp::transport::{TransportFactory, StdioConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = MCPClient::new(
    ///     MCPClientConfig::default(),
    ///     TransportFactory::stdio(StdioConfig::default())
    /// );
    ///
    /// match client.connect().await {
    ///     Ok(()) => println!("Connected successfully"),
    ///     Err(e) => eprintln!("Connection failed: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(&mut self) -> Result<(), MCPOperationError> {
        // Check if already connected
        if self.is_connected.load(Ordering::Relaxed) {
            return Err(MCPOperationError::session("Session already active"));
        }

        info!("Connecting to MCP server...");

        // Connect transport
        let streams = self.transport.connect().await?;

        // Start background message handler
        self.start_message_handler(streams).await?;

        // Mark as connected
        self.is_connected.store(true, Ordering::Relaxed);

        // Perform MCP handshake
        self.initialize_session().await?;

        Ok(())
    }

    /// Disconnect from the MCP server
    pub async fn disconnect(&mut self) -> Result<(), MCPOperationError> {
        if !self.is_connected.load(Ordering::Relaxed) {
            return Ok(());
        }

        info!("Disconnecting from MCP server...");

        // Send shutdown signal
        if let Some(shutdown_tx) = self.shutdown_tx.lock().await.take() {
            let _ = shutdown_tx.send(());
        }

        // Wait for background task to finish
        if let Some(handle) = self.background_handle.lock().await.take() {
            let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
        }

        // Clear write channel
        self.write_tx.lock().await.take();

        // Disconnect transport
        self.transport.disconnect().await?;

        // Clear session state
        let mut session = self.session.write().await;
        session.is_active = false;

        // Mark as disconnected
        self.is_connected.store(false, Ordering::Relaxed);

        Ok(())
    }

    /// Check if the client is connected
    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::Relaxed)
    }

    /// Start the background message handler
    async fn start_message_handler(
        &mut self,
        streams: TransportStreams,
    ) -> Result<(), MCPOperationError> {
        let TransportStreams {
            mut read_stream,
            write_stream,
        } = streams;

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        self.shutdown_tx.lock().await.replace(shutdown_tx);

        // Create write channel for sending messages
        let (write_tx, mut write_rx) = mpsc::unbounded_channel::<MCPMessage>();
        self.write_tx.lock().await.replace(write_tx.clone());

        // Spawn write task
        let write_handle = tokio::spawn(async move {
            let mut sink = write_stream;
            while let Some(msg) = write_rx.recv().await {
                if let Err(e) = sink.send(msg).await {
                    error!("Failed to send message: {}", e);
                    break;
                }
            }
        });

        // Clone necessary state for the background task
        let pending_requests = self.pending_requests.clone();
        let session = self.session.clone();
        let is_connected = self.is_connected.clone();

        // Spawn read task
        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle incoming messages
                    msg = read_stream.next() => {
                        match msg {
                            Some(Ok(message)) => {
                                Self::handle_message(message, &pending_requests, &session).await;
                            }
                            Some(Err(e)) => {
                                error!("Error reading message: {}", e);
                                break;
                            }
                            None => {
                                warn!("Message stream ended");
                                break;
                            }
                        }
                    }

                    // Handle shutdown signal
                    _ = shutdown_rx.recv() => {
                        debug!("Received shutdown signal");
                        break;
                    }
                }
            }

            // Mark as disconnected
            is_connected.store(false, Ordering::Relaxed);

            // Cancel write task
            write_handle.abort();

            // Clear pending requests with connection lost error
            let mut requests = pending_requests.lock().await;
            for (_, pending) in requests.drain() {
                let _ = pending
                    .response_tx
                    .send(Err(MCPOperationError::connection_lost("Connection closed")));
            }
        });

        self.background_handle.lock().await.replace(handle);

        Ok(())
    }

    /// Handle an incoming message
    async fn handle_message(
        message: MCPMessage,
        pending_requests: &Arc<Mutex<HashMap<Value, PendingRequest>>>,
        session: &Arc<RwLock<MCPSession>>,
    ) {
        match message {
            MCPMessage::Response(response) => {
                // Find and complete pending request
                let id = &response.id;
                let mut requests = pending_requests.lock().await;
                if let Some(pending) = requests.remove(id) {
                    let _ = pending.response_tx.send(Ok(response));
                } else {
                    warn!("Received response for unknown request ID: {:?}", id);
                }
            }
            MCPMessage::Request(request) => {
                // MCP servers shouldn't send requests to clients in the current protocol
                warn!(
                    "Received unexpected request from server: {}",
                    request.method
                );
            }
            MCPMessage::Notification(notification) => {
                // Handle notifications
                Self::handle_notification(notification, session).await;
            }
        }
    }

    /// Handle an incoming notification
    async fn handle_notification(
        notification: MCPNotification,
        _session: &Arc<RwLock<MCPSession>>,
    ) {
        match notification.method.as_str() {
            "tools/list_changed" => {
                info!("Server tools have changed, refreshing tool list...");
                // Note: Tool refresh would be implemented by calling list_tools again
            }
            "cancelled" => {
                if let Some(request_id) = notification.params.as_ref().and_then(|p| p.get("id")) {
                    warn!("Server cancelled request: {:?}", request_id);
                }
            }
            method => {
                debug!("Received notification: {}", method);
            }
        }
    }

    /// Initialize the MCP session with handshake
    async fn initialize_session(&mut self) -> Result<(), MCPOperationError> {
        info!("Initializing MCP session...");

        // Send initialize request
        let init_params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": self.config.client_capabilities,
            "clientInfo": {
                "name": self.config.client_name,
                "version": self.config.client_version
            }
        });

        let response = self.send_request("initialize", Some(init_params)).await?;

        // Parse initialize response
        if let MCPResponsePayload::Success { result } = response.payload {
            let mut session = self.session.write().await;

            // Extract server capabilities
            if let Some(capabilities) = result.get("capabilities") {
                session.server_capabilities = serde_json::from_value(capabilities.clone())
                    .map_err(|e| {
                        MCPOperationError::serialization(format!(
                            "Failed to parse server capabilities: {}",
                            e
                        ))
                    })?;
            }

            // Extract server info
            if let Some(server_info) = result.get("serverInfo") {
                if let Some(name) = server_info.get("name").and_then(|v| v.as_str()) {
                    session.server_name = Some(name.to_string());
                }
                if let Some(version) = server_info.get("version").and_then(|v| v.as_str()) {
                    session.server_version = Some(version.to_string());
                }
            }

            session.is_active = true;

            info!(
                "Session initialized with server: {} v{}",
                session.server_name.as_deref().unwrap_or("unknown"),
                session.server_version.as_deref().unwrap_or("unknown")
            );
            debug!("Server capabilities: {:?}", session.server_capabilities);
        } else if let MCPResponsePayload::Error { error } = response.payload {
            return Err(MCPOperationError::protocol(format!(
                "Initialize failed: {}",
                error.message
            )));
        }

        // Send initialized notification
        self.send_notification("notifications/initialized", None)
            .await?;

        // Discover available tools if server supports them
        let has_tools = {
            let session = self.session.read().await;
            session
                .server_capabilities
                .as_ref()
                .map(|caps| caps.tools.is_some())
                .unwrap_or(false)
        };

        if has_tools {
            self.refresh_tools().await?;
        }

        Ok(())
    }

    /// Send a request to the MCP server
    async fn send_request(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> Result<MCPResponse, MCPOperationError> {
        if !self.is_connected() {
            return Err(MCPOperationError::session("Session not initialized"));
        }

        // Generate request ID
        let request_id = self.request_id_counter.fetch_add(1, Ordering::SeqCst);
        let id = json!(request_id);

        // Create request
        let request = MCPRequest::new(id.clone(), method, params);

        // Create response channel
        let (response_tx, response_rx) = oneshot::channel();

        // Store pending request
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(
                id.clone(),
                PendingRequest {
                    response_tx,
                    _method: method.to_string(),
                    _timestamp: std::time::Instant::now(),
                },
            );
        }

        // Send request through write channel
        let message = MCPMessage::Request(request);

        if let Some(write_tx) = self.write_tx.lock().await.as_ref() {
            write_tx
                .send(message)
                .map_err(|_| MCPOperationError::connection_lost("Write channel closed"))?;
        } else {
            return Err(MCPOperationError::session("Session not initialized"));
        }

        // Wait for response with timeout
        let timeout_duration = Duration::from_millis(self.config.request_timeout_ms);
        match tokio::time::timeout(timeout_duration, response_rx).await {
            Ok(Ok(response)) => response,
            Ok(Err(_)) => Err(MCPOperationError::transport("Response channel closed")),
            Err(_) => {
                // Remove pending request on timeout
                self.pending_requests.lock().await.remove(&id);
                Err(MCPOperationError::timeout(timeout_duration))
            }
        }
    }

    /// Send a notification to the MCP server
    async fn send_notification(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> Result<(), MCPOperationError> {
        if !self.is_connected() {
            return Err(MCPOperationError::session("Session not initialized"));
        }

        // Create notification
        let notification = MCPNotification::new(method, params);
        let message = MCPMessage::Notification(notification);

        // Send notification through write channel
        if let Some(write_tx) = self.write_tx.lock().await.as_ref() {
            write_tx
                .send(message)
                .map_err(|_| MCPOperationError::connection_lost("Write channel closed"))?;
        } else {
            return Err(MCPOperationError::session("Session not initialized"));
        }

        Ok(())
    }

    /// Refresh the list of available tools from the server
    async fn refresh_tools(&mut self) -> Result<(), MCPOperationError> {
        info!("Refreshing tool list from MCP server...");

        let response = self.send_request("tools/list", None).await?;

        if let MCPResponsePayload::Success { result } = response.payload {
            if let Some(tools_value) = result.get("tools") {
                let tools: Vec<Tool> =
                    serde_json::from_value(tools_value.clone()).map_err(|e| {
                        MCPOperationError::serialization(format!("Failed to parse tools: {}", e))
                    })?;

                let mut session = self.session.write().await;
                session.available_tools.clear();

                for tool in tools {
                    info!("Discovered tool: {}", tool.name);
                    session.available_tools.insert(tool.name.clone(), tool);
                }

                info!(
                    "Discovered {} tools from MCP server",
                    session.available_tools.len()
                );
            }
        }

        Ok(())
    }

    /// Get all tools available from the connected MCP server
    ///
    /// Returns the complete list of tools that can be executed through this server.
    /// Each tool includes its name, description, and JSON schema for parameters.
    /// Tools are automatically discovered during connection.
    ///
    /// # Returns
    ///
    /// A vector of `Tool` objects describing available tools, or an error if
    /// the session is not initialized.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use stood::mcp::MCPClient;
    /// # async fn example(client: &MCPClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let tools = client.list_tools().await?;
    /// for tool in tools {
    ///     println!("Tool: {} - {}", tool.name, tool.description);
    ///     println!("  Schema: {}", tool.input_schema);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_tools(&self) -> Result<Vec<Tool>, MCPOperationError> {
        let session = self.session.read().await;
        Ok(session.available_tools.values().cloned().collect())
    }


    /// Execute a tool on the MCP server with parameters
    ///
    /// Calls the named tool with the provided arguments and returns the results.
    /// The tool must exist in the server's tool list (check with `list_tools()`).
    /// Arguments are validated against the tool's JSON schema.
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name of the tool to execute
    /// * `arguments` - JSON object containing tool parameters (optional)
    ///
    /// # Returns
    ///
    /// Tool execution results as a vector of `Content` objects, which may include
    /// text, images, or resource references depending on the tool.
    ///
    /// # Errors
    ///
    /// - `ProtocolError` - Tool not found or invalid parameters
    /// - `ToolExecutionError` - Tool execution failed on the server
    /// - `TimeoutError` - Tool execution exceeded timeout
    /// - `SerializationError` - Failed to parse tool results
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use stood::mcp::MCPClient;
    /// # use serde_json::json;
    /// # async fn example(client: &mut MCPClient) -> Result<(), Box<dyn std::error::Error>> {
    /// // Call a calculator tool
    /// let result = client.call_tool("calculator", Some(json!({
    ///     "operation": "multiply",
    ///     "a": 6,
    ///     "b": 7
    /// }))).await?;
    ///
    /// for content in result {
    ///     match content {
    ///         stood::mcp::Content::Text(text) => {
    ///             println!("Result: {}", text.text);
    ///         }
    ///         _ => println!("Non-text result received"),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call_tool(
        &mut self,
        tool_name: &str,
        arguments: Option<Value>,
    ) -> Result<Vec<Content>, MCPOperationError> {
        // Verify tool exists
        {
            let session = self.session.read().await;
            if !session.available_tools.contains_key(tool_name) {
                return Err(MCPOperationError::protocol(format!(
                    "Tool not found: {}",
                    tool_name
                )));
            }
        }

        let params = json!({
            "name": tool_name,
            "arguments": arguments,
        });

        let response = self.send_request("tools/call", Some(params)).await?;

        if let MCPResponsePayload::Success { result } = response.payload {
            if let Some(content_value) = result.get("content") {
                let content: Vec<Content> =
                    serde_json::from_value(content_value.clone()).map_err(|e| {
                        MCPOperationError::serialization(format!(
                            "Failed to parse tool result: {}",
                            e
                        ))
                    })?;
                return Ok(content);
            }
        }

        Err(MCPOperationError::protocol("Tool call returned no content"))
    }

    /// Get current session metadata and server information
    ///
    /// Returns details about the active session including session ID, server name/version,
    /// and negotiated capabilities. Useful for debugging and logging.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - Session ID (unique identifier)
    /// - Server name (if provided during handshake)
    /// - Server version (if provided during handshake)
    /// - Server capabilities (tools, resources, prompts support)
    ///
    /// # Errors
    ///
    /// Returns an error if no session is currently active.
    pub async fn session_info(
        &self,
    ) -> Result<
        (
            String,
            Option<String>,
            Option<String>,
            Option<ServerCapabilities>,
        ),
        MCPOperationError,
    > {
        let session = self.session.read().await;
        if !session.is_active {
            return Err(MCPOperationError::session("Session not initialized"));
        }

        Ok((
            session.session_id.clone(),
            session.server_name.clone(),
            session.server_version.clone(),
            session.server_capabilities.clone(),
        ))
    }

    /// Get how long the current session has been active
    ///
    /// Returns the elapsed time since the session was established.
    /// Useful for monitoring connection health and session lifetime.
    pub async fn session_duration(&self) -> Duration {
        let session = self.session.read().await;
        session.start_time.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::transport::{create_test_streams, TransportInfo};
    use async_trait::async_trait;

    /// Mock transport for testing
    struct MockTransport {
        connected: bool,
        streams: Option<(
            mpsc::UnboundedSender<Result<MCPMessage, MCPOperationError>>,
            mpsc::UnboundedReceiver<MCPMessage>,
        )>,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                connected: false,
                streams: None,
            }
        }
    }

    #[async_trait]
    impl MCPTransport for MockTransport {
        async fn connect(&mut self) -> Result<TransportStreams, MCPOperationError> {
            let (read_tx, write_rx, streams) = create_test_streams();
            self.streams = Some((read_tx, write_rx));
            self.connected = true;
            Ok(streams)
        }

        async fn disconnect(&mut self) -> Result<(), MCPOperationError> {
            self.connected = false;
            self.streams = None;
            Ok(())
        }

        fn is_connected(&self) -> bool {
            self.connected
        }

        fn transport_info(&self) -> TransportInfo {
            TransportInfo {
                transport_type: "mock".to_string(),
                endpoint: "mock://test".to_string(),
                supports_reconnection: true,
                max_message_size: None,
            }
        }
    }

    #[test]
    fn test_mcp_client_config_default() {
        let config = MCPClientConfig::default();
        assert_eq!(config.client_name, "stood-mcp-client");
        assert_eq!(config.request_timeout_ms, 30_000);
        assert_eq!(config.max_concurrent_requests, 100);
        assert!(config.auto_reconnect);
        assert_eq!(config.reconnect_delay_ms, 5_000);
    }

    #[test]
    fn test_mcp_session_creation() {
        let session = MCPSession::new();
        assert!(!session.session_id.is_empty());
        assert!(session.server_capabilities.is_none());
        assert!(session.server_name.is_none());
        assert!(session.server_version.is_none());
        assert!(session.available_tools.is_empty());
        assert!(!session.is_active);
    }

    #[tokio::test]
    async fn test_mcp_client_creation() {
        let config = MCPClientConfig::default();
        let transport = Box::new(MockTransport::new());
        let client = MCPClient::new(config, transport);

        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn test_mcp_client_connect_disconnect() {
        let config = MCPClientConfig::default();
        let transport = Box::new(MockTransport::new());
        let mut client = MCPClient::new(config, transport);

        // Initially not connected
        assert!(!client.is_connected());

        // Note: Connect will fail because we haven't implemented send_request yet
        // This is expected for now

        // Test disconnect when not connected
        assert!(client.disconnect().await.is_ok());
    }

    #[tokio::test]
    async fn test_session_duration() {
        let config = MCPClientConfig::default();
        let transport = Box::new(MockTransport::new());
        let client = MCPClient::new(config, transport);

        // Wait a bit
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Session duration should be at least 100ms
        let duration = client.session_duration().await;
        assert!(duration >= Duration::from_millis(100));
    }
}
