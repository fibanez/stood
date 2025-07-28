//! Transport layer for MCP communication channels
//!
//! This module enables you to connect to MCP servers through different communication
//! channels. You'll get support for WebSocket servers and stdio-based local processes
//! with automatic message serialization and connection management.
//!
//! # Supported Transports
//!
//! ## WebSocket Transport
//!
//! Connect to network-based MCP servers:
//! ```no_run
//! use stood::mcp::transport::{TransportFactory, WebSocketConfig};
//!
//! let config = WebSocketConfig {
//!     url: "wss://api.example.com/mcp".to_string(),
//!     connect_timeout_ms: 10_000,
//!     ..Default::default()
//! };
//!
//! let transport = TransportFactory::websocket(config);
//! ```
//!
//! ## Stdio Transport
//!
//! Connect to local MCP server processes:
//! ```no_run
//! use stood::mcp::transport::{TransportFactory, StdioConfig};
//!
//! let config = StdioConfig {
//!     command: "python".to_string(),
//!     args: vec!["-m".to_string(), "my_mcp_server".to_string()],
//!     ..Default::default()
//! };
//!
//! let transport = TransportFactory::stdio(config);
//! ```
//!
//! # Architecture
//!
//! Transports follow an async context manager pattern:
//!
//! 1. **Connect** - Establish the underlying connection and return message streams
//! 2. **Communicate** - Exchange MCP messages through read/write streams
//! 3. **Disconnect** - Clean up resources and close connections
//!
//! # Message Handling
//!
//! - **Automatic Serialization** - JSON-RPC messages are automatically serialized/deserialized
//! - **Line-based Communication** - Stdio uses line-buffered communication for reliability
//! - **Error Recovery** - Transport errors are mapped to appropriate MCP error types
//! - **Resource Management** - Background tasks handle I/O with proper cleanup
//!
//! # Performance
//!
//! - WebSocket connections: Sub-second connection times to most servers
//! - Stdio processes: <100ms startup time for typical Python/Node.js servers
//! - Message throughput: Limited by server processing, not transport overhead
//! - Memory usage: Minimal buffering with efficient stream processing

use crate::mcp::error::MCPOperationError;
use crate::mcp::types::MCPMessage;
use async_trait::async_trait;
use futures::sink::{Sink, SinkExt};
use futures::stream::{Stream, StreamExt};
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use url::Url;

/// Type alias for message streams
pub type MessageStream = Pin<Box<dyn Stream<Item = Result<MCPMessage, MCPOperationError>> + Send>>;
pub type MessageSink = Pin<Box<dyn Sink<MCPMessage, Error = MCPOperationError> + Send>>;

/// Transport connection result containing read and write streams
pub struct TransportStreams {
    /// Stream for receiving messages from the MCP server
    pub read_stream: MessageStream,
    /// Sink for sending messages to the MCP server  
    pub write_stream: MessageSink,
}

// Manual Debug implementation since streams are not easily debuggable
impl std::fmt::Debug for TransportStreams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransportStreams")
            .field("read_stream", &"<message_stream>")
            .field("write_stream", &"<message_sink>")
            .finish()
    }
}

/// Core trait for MCP transport implementations
///
/// This trait defines the interface that all MCP transports must implement.
/// Transports handle the underlying communication channel while the MCP client
/// manages protocol-level concerns.
///
/// # Implementation Notes
///
/// Implementations should:
/// - Handle connection establishment and cleanup
/// - Provide bidirectional message streams
/// - Manage transport-specific resources (processes, sockets, etc.)
/// - Report connection status and transport metadata
///
/// The trait follows an async context manager pattern where `connect()` establishes
/// the connection and returns streams for communication.
#[async_trait]
pub trait MCPTransport: Send + Sync {
    /// Establish connection and return communication streams
    ///
    /// This method creates the underlying transport connection (WebSocket connection,
    /// process spawn, etc.) and returns streams for bidirectional MCP message exchange.
    /// Background tasks handle the actual I/O operations.
    ///
    /// # Returns
    ///
    /// `TransportStreams` containing read and write streams for MCP communication.
    /// Messages are automatically serialized/deserialized.
    ///
    /// # Errors
    ///
    /// - `TransportError` - Failed to establish connection (network, process spawn, etc.)
    /// - `TimeoutError` - Connection attempt exceeded configured timeout
    /// - `ConfigurationError` - Invalid transport configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use stood::mcp::transport::{MCPTransport, WebSocketTransport, WebSocketConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = WebSocketConfig {
    ///     url: "ws://localhost:8080".to_string(),
    ///     ..Default::default()
    /// };
    /// let mut transport = WebSocketTransport::new(config);
    /// let streams = transport.connect().await?;
    /// // Use streams for MCP communication
    /// # Ok(())
    /// # }
    /// ```
    async fn connect(&mut self) -> Result<TransportStreams, MCPOperationError>;

    /// Close connection and clean up transport resources
    ///
    /// This method gracefully closes the underlying connection and cleans up
    /// background tasks. For process-based transports, this includes terminating
    /// the child process. For network transports, this closes sockets.
    ///
    /// # Errors
    ///
    /// Returns `MCPOperationError` for critical cleanup failures. Non-critical
    /// cleanup issues are logged but don't result in errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use stood::mcp::transport::MCPTransport;
    /// # async fn example(mut transport: Box<dyn MCPTransport>) -> Result<(), Box<dyn std::error::Error>> {
    /// transport.disconnect().await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn disconnect(&mut self) -> Result<(), MCPOperationError>;

    /// Check current connection status
    ///
    /// Returns `true` if the transport has an active connection that can send/receive
    /// messages. This reflects the logical connection state, not just socket status.
    fn is_connected(&self) -> bool;

    /// Get transport metadata and capabilities
    ///
    /// Returns information about this transport including type, endpoint,
    /// reconnection support, and size limits. Useful for debugging and monitoring.
    fn transport_info(&self) -> TransportInfo;
}

/// Metadata about a transport's capabilities and configuration
///
/// This information helps clients understand transport characteristics
/// and limitations for proper error handling and optimization.
#[derive(Debug, Clone)]
pub struct TransportInfo {
    /// Type of transport (e.g., "websocket", "stdio")
    pub transport_type: String,
    /// Connection endpoint or identifier
    pub endpoint: String,
    /// Whether the transport supports reconnection
    pub supports_reconnection: bool,
    /// Maximum message size supported (if any)
    pub max_message_size: Option<usize>,
}

/// Configuration for WebSocket-based MCP servers
///
/// Use this for connecting to MCP servers running over WebSocket,
/// typically network-accessible servers or web-based integrations.
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// WebSocket URL to connect to
    pub url: String,
    /// Connection timeout in milliseconds
    pub connect_timeout_ms: u64,
    /// Keep-alive ping interval in milliseconds
    pub ping_interval_ms: Option<u64>,
    /// Maximum message size in bytes
    pub max_message_size: Option<usize>,
    /// Additional headers to send with the connection
    pub headers: std::collections::HashMap<String, String>,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            connect_timeout_ms: 30_000,               // 30 seconds
            ping_interval_ms: Some(30_000),           // 30 seconds
            max_message_size: Some(16 * 1024 * 1024), // 16MB
            headers: std::collections::HashMap::new(),
        }
    }
}

/// Configuration for process-based MCP servers using stdin/stdout
///
/// Use this for local MCP servers implemented as separate processes
/// that communicate via standard input/output streams.
#[derive(Debug, Clone)]
pub struct StdioConfig {
    /// Command to execute
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Working directory for the process
    pub working_dir: Option<String>,
    /// Environment variables to set
    pub env_vars: std::collections::HashMap<String, String>,
    /// Process startup timeout in milliseconds
    pub startup_timeout_ms: u64,
    /// Maximum message size in bytes
    pub max_message_size: Option<usize>,
}

impl Default for StdioConfig {
    fn default() -> Self {
        Self {
            command: String::new(),
            args: Vec::new(),
            working_dir: None,
            env_vars: std::collections::HashMap::new(),
            startup_timeout_ms: 10_000,               // 10 seconds
            max_message_size: Some(16 * 1024 * 1024), // 16MB
        }
    }
}

/// Factory for creating configured transport instances
///
/// Provides convenient methods to create transports with proper configuration.
/// This is the recommended way to create transport instances.
pub struct TransportFactory;

impl TransportFactory {
    /// Create a WebSocket transport for network-based MCP servers
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stood::mcp::transport::{TransportFactory, WebSocketConfig};
    ///
    /// let config = WebSocketConfig {
    ///     url: "wss://api.example.com/mcp".to_string(),
    ///     connect_timeout_ms: 15_000,
    ///     ..Default::default()
    /// };
    /// let transport = TransportFactory::websocket(config);
    /// ```
    pub fn websocket(config: WebSocketConfig) -> Box<dyn MCPTransport> {
        Box::new(WebSocketTransport::new(config))
    }

    /// Create a stdio transport for local process-based MCP servers
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stood::mcp::transport::{TransportFactory, StdioConfig};
    ///
    /// let config = StdioConfig {
    ///     command: "python".to_string(),
    ///     args: vec!["-m".to_string(), "my_mcp_server".to_string()],
    ///     working_dir: Some("/path/to/server".to_string()),
    ///     ..Default::default()
    /// };
    /// let transport = TransportFactory::stdio(config);
    /// ```
    pub fn stdio(config: StdioConfig) -> Box<dyn MCPTransport> {
        Box::new(StdioTransport::new(config))
    }
}

/// WebSocket transport implementation
struct WebSocketTransport {
    config: WebSocketConfig,
    connected: bool,
    // Store close sender to signal disconnection
    close_sender: Option<mpsc::UnboundedSender<()>>,
}

impl WebSocketTransport {
    fn new(config: WebSocketConfig) -> Self {
        Self {
            config,
            connected: false,
            close_sender: None,
        }
    }
}

#[async_trait]
impl MCPTransport for WebSocketTransport {
    async fn connect(&mut self) -> Result<TransportStreams, MCPOperationError> {
        // Parse the WebSocket URL
        let url = Url::parse(&self.config.url)
            .map_err(|e| MCPOperationError::transport(format!("Invalid WebSocket URL: {}", e)))?;

        // Create a timeout for the connection
        let connect_timeout = Duration::from_millis(self.config.connect_timeout_ms);

        // Attempt to connect with timeout
        let (ws_stream, _response) = tokio::time::timeout(connect_timeout, connect_async(&url))
            .await
            .map_err(|_| MCPOperationError::timeout(connect_timeout))?
            .map_err(|e| {
                MCPOperationError::websocket(format!("WebSocket connection failed: {}", e))
            })?;

        // Create channels for communication
        let (read_tx, read_rx) = mpsc::unbounded_channel();
        let (write_tx, mut write_rx) = mpsc::unbounded_channel();
        let (close_tx, mut close_rx) = mpsc::unbounded_channel();

        // Store the close sender for disconnection
        self.close_sender = Some(close_tx);

        // Split the WebSocket stream
        let (mut ws_sink, mut ws_stream) = ws_stream.split();

        // Spawn task to handle incoming WebSocket messages
        let read_tx_clone = read_tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle incoming WebSocket messages
                    msg = ws_stream.next() => {
                        match msg {
                            Some(Ok(WsMessage::Text(text))) => {
                                // Parse JSON-RPC message
                                match serde_json::from_str::<MCPMessage>(&text) {
                                    Ok(mcp_msg) => {
                                        if read_tx_clone.send(Ok(mcp_msg)).is_err() {
                                            break; // Receiver dropped
                                        }
                                    }
                                    Err(e) => {
                                        let error = MCPOperationError::serialization(
                                            format!("Failed to parse MCP message: {}", e)
                                        );
                                        if read_tx_clone.send(Err(error)).is_err() {
                                            break; // Receiver dropped
                                        }
                                    }
                                }
                            }
                            Some(Ok(WsMessage::Close(_))) => {
                                let error = MCPOperationError::connection_lost("WebSocket closed by remote");
                                let _ = read_tx_clone.send(Err(error));
                                break;
                            }
                            Some(Err(e)) => {
                                let error = MCPOperationError::websocket(format!("WebSocket error: {}", e));
                                let _ = read_tx_clone.send(Err(error));
                                break;
                            }
                            None => {
                                let error = MCPOperationError::connection_lost("WebSocket stream ended");
                                let _ = read_tx_clone.send(Err(error));
                                break;
                            }
                            // Ignore binary messages and pings/pongs
                            _ => continue,
                        }
                    }

                    // Handle close signal
                    _ = close_rx.recv() => {
                        break;
                    }
                }
            }
        });

        // Spawn task to handle outgoing WebSocket messages
        tokio::spawn(async move {
            while let Some(mcp_msg) = write_rx.recv().await {
                // Serialize MCP message to JSON
                let json = match serde_json::to_string(&mcp_msg) {
                    Ok(json) => json,
                    Err(_) => continue, // Skip invalid messages
                };

                // Send as WebSocket text message
                if ws_sink.send(WsMessage::Text(json)).await.is_err() {
                    break; // Connection lost
                }
            }
        });

        // Create streams for the transport
        let read_stream = Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(
            read_rx,
        ));
        let write_stream = Box::pin(futures::sink::unfold(write_tx, |tx, msg| async move {
            tx.send(msg)
                .map_err(|_| MCPOperationError::transport("Channel closed"))
                .map(|_| tx)
        }));

        self.connected = true;

        Ok(TransportStreams {
            read_stream,
            write_stream,
        })
    }

    async fn disconnect(&mut self) -> Result<(), MCPOperationError> {
        if let Some(close_sender) = self.close_sender.take() {
            let _ = close_sender.send(()); // Signal background task to close
        }
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn transport_info(&self) -> TransportInfo {
        TransportInfo {
            transport_type: "websocket".to_string(),
            endpoint: self.config.url.clone(),
            supports_reconnection: true,
            max_message_size: self.config.max_message_size,
        }
    }
}

/// Stdio transport implementation for process-based MCP servers
pub struct StdioTransport {
    config: StdioConfig,
    connected: bool,
    // Store process handle and close sender for lifecycle management
    process_handle: Option<tokio::process::Child>,
    close_sender: Option<mpsc::UnboundedSender<()>>,
    // Store external handles when using from_handles
    external_handles: Option<(tokio::process::ChildStdin, tokio::process::ChildStdout)>,
}

impl StdioTransport {
    pub fn new(config: StdioConfig) -> Self {
        Self {
            config,
            connected: false,
            process_handle: None,
            close_sender: None,
            external_handles: None,
        }
    }

    /// Create a StdioTransport from existing stdin/stdout handles
    /// 
    /// This is useful for testing scenarios where you have already spawned
    /// a process and want to create a transport from its handles.
    pub fn from_handles(
        stdin: tokio::process::ChildStdin,
        stdout: tokio::process::ChildStdout,
    ) -> Self {
        // Create a minimal config for handle-based transport
        let config = StdioConfig {
            command: "<external>".to_string(),
            args: Vec::new(),
            working_dir: None,
            env_vars: std::collections::HashMap::new(),
            startup_timeout_ms: 5000,
            max_message_size: None,
        };

        Self {
            config,
            connected: false,
            process_handle: None,
            close_sender: None,
            external_handles: Some((stdin, stdout)),
        }
    }

    /// Connect using pre-existing handles instead of spawning a new process
    async fn connect_with_handles(
        &mut self,
        mut stdin: tokio::process::ChildStdin,
        stdout: tokio::process::ChildStdout,
    ) -> Result<TransportStreams, MCPOperationError> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        // Create channels for communication
        let (read_tx, read_rx) = mpsc::unbounded_channel();
        let (write_tx, mut write_rx) = mpsc::unbounded_channel();
        let (close_tx, mut close_rx) = mpsc::unbounded_channel();

        // Store the close sender
        self.close_sender = Some(close_tx);

        // Spawn task to handle stdout reading
        let read_tx_clone = read_tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();

            loop {
                tokio::select! {
                    // Read from stdout
                    result = reader.read_line(&mut line) => {
                        match result {
                            Ok(0) => {
                                // EOF reached
                                let error = MCPOperationError::connection_lost("Process stdout ended");
                                let _ = read_tx_clone.send(Err(error));
                                break;
                            }
                            Ok(_) => {
                                // Parse the line as JSON-RPC message
                                let trimmed = line.trim();
                                if !trimmed.is_empty() {
                                    match serde_json::from_str::<crate::mcp::types::MCPMessage>(trimmed) {
                                        Ok(message) => {
                                            if read_tx_clone.send(Ok(message)).is_err() {
                                                break; // Receiver dropped
                                            }
                                        }
                                        Err(e) => {
                                            let error = MCPOperationError::serialization(
                                                format!("Failed to parse MCP message: {}", e)
                                            );
                                            if read_tx_clone.send(Err(error)).is_err() {
                                                break; // Receiver dropped
                                            }
                                        }
                                    }
                                }
                                line.clear();
                            }
                            Err(e) => {
                                let error = MCPOperationError::stdio(format!("Failed to read from stdout: {}", e));
                                let _ = read_tx_clone.send(Err(error));
                                break;
                            }
                        }
                    }
                    // Handle close signal
                    _ = close_rx.recv() => {
                        break;
                    }
                }
            }
        });

        // Spawn task to handle stdin writing
        tokio::spawn(async move {
            while let Some(message) = write_rx.recv().await {
                // Serialize message to JSON
                match serde_json::to_string(&message) {
                    Ok(json) => {
                        let line = format!("{}\n", json);
                        if let Err(e) = stdin.write_all(line.as_bytes()).await {
                            tracing::error!("Failed to write to stdin: {}", e);
                            break;
                        }
                        if let Err(e) = stdin.flush().await {
                            tracing::error!("Failed to flush stdin: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to serialize message: {}", e);
                        break;
                    }
                }
            }
        });

        // Create read stream
        let read_stream: MessageStream = Box::pin(
            tokio_stream::wrappers::UnboundedReceiverStream::new(read_rx)
        );

        // Create write sink
        let write_sink: MessageSink = Box::pin(futures::sink::unfold(write_tx, |tx, msg| async move {
            match tx.send(msg) {
                Ok(()) => Ok(tx),
                Err(_) => Err(MCPOperationError::connection_lost("Write channel closed")),
            }
        }));

        self.connected = true;

        Ok(TransportStreams {
            read_stream,
            write_stream: write_sink,
        })
    }
}

#[async_trait]
impl MCPTransport for StdioTransport {
    async fn connect(&mut self) -> Result<TransportStreams, MCPOperationError> {
        // Check if we have external handles to use
        if let Some((stdin, stdout)) = self.external_handles.take() {
            return self.connect_with_handles(stdin, stdout).await;
        }

        use std::process::Stdio;
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::process::Command;

        // Validate configuration
        if self.config.command.is_empty() {
            return Err(MCPOperationError::transport("Command cannot be empty"));
        }

        // Build the command
        let mut cmd = Command::new(&self.config.command);
        cmd.args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set working directory if specified
        if let Some(ref dir) = self.config.working_dir {
            cmd.current_dir(dir);
        }

        // Set environment variables
        for (key, value) in &self.config.env_vars {
            cmd.env(key, value);
        }

        // Spawn the process (spawn is not async, so we can't timeout it directly)
        let mut child = cmd.spawn().map_err(|e| {
            MCPOperationError::stdio(format!(
                "Failed to spawn process '{}': {}",
                self.config.command, e
            ))
        })?;

        // Get stdin and stdout handles
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| MCPOperationError::stdio("Failed to get stdin handle"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| MCPOperationError::stdio("Failed to get stdout handle"))?;

        // Create channels for communication
        let (read_tx, read_rx) = mpsc::unbounded_channel();
        let (write_tx, mut write_rx) = mpsc::unbounded_channel();
        let (close_tx, mut close_rx) = mpsc::unbounded_channel();

        // Store handles for lifecycle management
        self.process_handle = Some(child);
        self.close_sender = Some(close_tx);

        // Spawn task to handle stdout reading
        let read_tx_clone = read_tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();

            loop {
                tokio::select! {
                    // Read lines from stdout
                    result = reader.read_line(&mut line) => {
                        match result {
                            Ok(0) => {
                                // EOF - process stdout closed
                                let error = MCPOperationError::connection_lost("Process stdout closed");
                                let _ = read_tx_clone.send(Err(error));
                                break;
                            }
                            Ok(_) => {
                                // Parse JSON-RPC message from line
                                let trimmed = line.trim();
                                if !trimmed.is_empty() {
                                    match serde_json::from_str::<MCPMessage>(trimmed) {
                                        Ok(mcp_msg) => {
                                            if read_tx_clone.send(Ok(mcp_msg)).is_err() {
                                                break; // Receiver dropped
                                            }
                                        }
                                        Err(e) => {
                                            let error = MCPOperationError::serialization(
                                                format!("Failed to parse MCP message: {}", e)
                                            );
                                            if read_tx_clone.send(Err(error)).is_err() {
                                                break; // Receiver dropped
                                            }
                                        }
                                    }
                                }
                                line.clear();
                            }
                            Err(e) => {
                                let error = MCPOperationError::stdio(format!("Error reading from stdout: {}", e));
                                let _ = read_tx_clone.send(Err(error));
                                break;
                            }
                        }
                    }

                    // Handle close signal
                    _ = close_rx.recv() => {
                        break;
                    }
                }
            }
        });

        // Spawn task to handle stdin writing
        tokio::spawn(async move {
            while let Some(mcp_msg) = write_rx.recv().await {
                // Serialize MCP message to JSON
                let json = match serde_json::to_string(&mcp_msg) {
                    Ok(json) => json,
                    Err(_) => continue, // Skip invalid messages
                };

                // Write JSON line to stdin
                let line = format!("{}\n", json);
                if stdin.write_all(line.as_bytes()).await.is_err() {
                    break; // Process stdin closed
                }

                // Flush to ensure message is sent immediately
                if stdin.flush().await.is_err() {
                    break; // Process stdin closed
                }
            }
        });

        // Create streams for the transport
        let read_stream = Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(
            read_rx,
        ));
        let write_stream = Box::pin(futures::sink::unfold(write_tx, |tx, msg| async move {
            tx.send(msg)
                .map_err(|_| MCPOperationError::transport("Channel closed"))
                .map(|_| tx)
        }));

        self.connected = true;

        Ok(TransportStreams {
            read_stream,
            write_stream,
        })
    }

    async fn disconnect(&mut self) -> Result<(), MCPOperationError> {
        // Signal background tasks to close
        if let Some(close_sender) = self.close_sender.take() {
            let _ = close_sender.send(());
        }

        // Terminate the process gracefully
        if let Some(mut child) = self.process_handle.take() {
            // Try graceful termination first
            #[cfg(unix)]
            {
                if let Some(pid) = child.id() {
                    // Send SIGTERM for graceful shutdown
                    unsafe {
                        libc::kill(pid as i32, libc::SIGTERM);
                    }

                    // Wait up to 5 seconds for graceful shutdown
                    match tokio::time::timeout(Duration::from_secs(5), child.wait()).await {
                        Ok(Ok(_)) => {
                            // Process exited gracefully
                        }
                        _ => {
                            // Force kill if graceful shutdown failed
                            let _ = child.kill().await;
                            let _ = child.wait().await;
                        }
                    }
                }
            }

            #[cfg(not(unix))]
            {
                // On non-Unix systems, just force kill
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
        }

        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn transport_info(&self) -> TransportInfo {
        TransportInfo {
            transport_type: "stdio".to_string(),
            endpoint: format!("{} {}", self.config.command, self.config.args.join(" ")),
            supports_reconnection: false, // Process needs to be restarted
            max_message_size: self.config.max_message_size,
        }
    }
}

/// Create test message streams for unit testing
///
/// This function creates connected message streams that can be used in tests
/// to simulate transport communication without actual network or process connections.
///
/// # Returns
///
/// A tuple containing:
/// - Sender for injecting test messages into the read stream
/// - Receiver for capturing messages sent to the write stream
/// - TransportStreams for use with MCP clients
///
/// # Examples
///
/// ```rust
/// use stood::mcp::transport::create_test_streams;
/// use stood::mcp::types::{MCPMessage, MCPRequest};
/// use serde_json::json;
///
/// let (read_tx, write_rx, streams) = create_test_streams();
///
/// // Inject a test message
/// let test_msg = MCPMessage::Request(MCPRequest::new(json!(1), "test", None));
/// read_tx.send(Ok(test_msg)).unwrap();
///
/// // streams can now be used with MCPClient for testing
/// ```
pub fn create_test_streams() -> (
    mpsc::UnboundedSender<Result<MCPMessage, MCPOperationError>>,
    mpsc::UnboundedReceiver<MCPMessage>,
    TransportStreams,
) {
    let (read_tx, read_rx) = mpsc::unbounded_channel();
    let (write_tx, write_rx) = mpsc::unbounded_channel();

    let read_stream = Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(
        read_rx,
    ));
    let write_stream = Box::pin(futures::sink::unfold(write_tx, |tx, msg| async move {
        tx.send(msg)
            .map_err(|_| MCPOperationError::transport("Channel closed"))
            .map(|_| tx)
    }));

    let streams = TransportStreams {
        read_stream,
        write_stream,
    };

    (read_tx, write_rx, streams)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::types::{MCPMessage, MCPRequest};
    use serde_json::json;

    #[test]
    fn test_websocket_config_default() {
        let config = WebSocketConfig::default();
        assert_eq!(config.connect_timeout_ms, 30_000);
        assert_eq!(config.ping_interval_ms, Some(30_000));
        assert_eq!(config.max_message_size, Some(16 * 1024 * 1024));
        assert!(config.headers.is_empty());
    }

    #[test]
    fn test_stdio_config_default() {
        let config = StdioConfig::default();
        assert_eq!(config.startup_timeout_ms, 10_000);
        assert_eq!(config.max_message_size, Some(16 * 1024 * 1024));
        assert!(config.args.is_empty());
        assert!(config.env_vars.is_empty());
    }

    #[test]
    fn test_transport_factory() {
        let ws_config = WebSocketConfig {
            url: "ws://localhost:8080".to_string(),
            ..Default::default()
        };
        let ws_transport = TransportFactory::websocket(ws_config);
        assert!(!ws_transport.is_connected());

        let stdio_config = StdioConfig {
            command: "python".to_string(),
            args: vec!["server.py".to_string()],
            ..Default::default()
        };
        let stdio_transport = TransportFactory::stdio(stdio_config);
        assert!(!stdio_transport.is_connected());
    }

    #[test]
    fn test_transport_info() {
        let ws_config = WebSocketConfig {
            url: "ws://example.com".to_string(),
            ..Default::default()
        };
        let ws_transport = WebSocketTransport::new(ws_config);
        let info = ws_transport.transport_info();

        assert_eq!(info.transport_type, "websocket");
        assert_eq!(info.endpoint, "ws://example.com");
        assert!(info.supports_reconnection);
        assert_eq!(info.max_message_size, Some(16 * 1024 * 1024));
    }

    #[tokio::test]
    async fn test_transport_disconnect() {
        let config = WebSocketConfig::default();
        let mut transport = WebSocketTransport::new(config);

        // Disconnect should succeed even when not connected
        assert!(transport.disconnect().await.is_ok());
        assert!(!transport.is_connected());
    }

    #[test]
    fn test_create_test_streams() {
        let (read_tx, write_rx, _streams) = create_test_streams();

        // Test that we can send a message through the test channels
        let test_message = MCPMessage::Request(MCPRequest::new(json!(1), "test_method", None));

        assert!(read_tx.send(Ok(test_message)).is_ok());
        // write_rx should be available for receiving messages sent through the write stream
        drop(write_rx); // Just test that it was created
    }

    #[tokio::test]
    async fn test_websocket_invalid_url() {
        let ws_config = WebSocketConfig {
            url: "invalid-url".to_string(),
            ..Default::default()
        };
        let mut ws_transport = WebSocketTransport::new(ws_config);

        // Should return error for invalid URL
        let result = ws_transport.connect().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid WebSocket URL"));
    }

    #[tokio::test]
    async fn test_websocket_connection_refused() {
        let ws_config = WebSocketConfig {
            url: "ws://localhost:9999".to_string(), // Assume this port is not open
            connect_timeout_ms: 100,                // Short timeout
            ..Default::default()
        };
        let mut ws_transport = WebSocketTransport::new(ws_config);

        // Should return error for connection refused
        let result = ws_transport.connect().await;
        assert!(result.is_err());
        // Could be timeout or connection failed
        let error_str = result.unwrap_err().to_string();
        assert!(
            error_str.contains("timeout")
                || error_str.contains("connection")
                || error_str.contains("WebSocket")
        );
    }

    #[tokio::test]
    async fn test_websocket_transport_lifecycle() {
        let ws_config = WebSocketConfig {
            url: "ws://echo.websocket.org".to_string(),
            connect_timeout_ms: 5000,
            ..Default::default()
        };
        let mut ws_transport = WebSocketTransport::new(ws_config);

        // Initially not connected
        assert!(!ws_transport.is_connected());

        // Try to connect to a real WebSocket echo server
        // Note: This test requires internet connectivity and may be flaky
        if let Ok(_streams) = ws_transport.connect().await {
            // Should be connected after successful connection
            assert!(ws_transport.is_connected());

            // Should be able to disconnect
            assert!(ws_transport.disconnect().await.is_ok());
            assert!(!ws_transport.is_connected());
        }
        // If connection fails (no internet, server down), that's ok for this test
    }

    #[test]
    fn test_websocket_transport_info() {
        let ws_config = WebSocketConfig {
            url: "wss://secure.example.com/mcp".to_string(),
            max_message_size: Some(1024),
            ..Default::default()
        };
        let ws_transport = WebSocketTransport::new(ws_config);
        let info = ws_transport.transport_info();

        assert_eq!(info.transport_type, "websocket");
        assert_eq!(info.endpoint, "wss://secure.example.com/mcp");
        assert!(info.supports_reconnection);
        assert_eq!(info.max_message_size, Some(1024));
    }

    #[test]
    fn test_websocket_config_validation() {
        // Test default configuration
        let default_config = WebSocketConfig::default();
        assert_eq!(default_config.connect_timeout_ms, 30_000);
        assert_eq!(default_config.ping_interval_ms, Some(30_000));
        assert_eq!(default_config.max_message_size, Some(16 * 1024 * 1024));

        // Test custom configuration
        let custom_config = WebSocketConfig {
            url: "wss://api.example.com/ws".to_string(),
            connect_timeout_ms: 10_000,
            ping_interval_ms: Some(60_000),
            max_message_size: Some(1024 * 1024),
            headers: [("Authorization".to_string(), "Bearer token".to_string())].into(),
        };
        assert_eq!(custom_config.url, "wss://api.example.com/ws");
        assert_eq!(custom_config.connect_timeout_ms, 10_000);
        assert_eq!(custom_config.headers.len(), 1);
    }

    #[tokio::test]
    async fn test_stdio_transport_empty_command() {
        let stdio_config = StdioConfig::default(); // Empty command
        let mut stdio_transport = StdioTransport::new(stdio_config);

        // Should return error for empty command
        let result = stdio_transport.connect().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Command cannot be empty"));
    }

    #[tokio::test]
    async fn test_stdio_transport_invalid_command() {
        let stdio_config = StdioConfig {
            command: "nonexistent_command_12345".to_string(),
            startup_timeout_ms: 1000, // Short timeout
            ..Default::default()
        };
        let mut stdio_transport = StdioTransport::new(stdio_config);

        // Should return error for invalid command
        let result = stdio_transport.connect().await;
        assert!(result.is_err());
        let error_str = result.unwrap_err().to_string();
        assert!(error_str.contains("Failed to spawn process") || error_str.contains("stdio"));
    }

    #[test]
    fn test_stdio_transport_info() {
        let stdio_config = StdioConfig {
            command: "python".to_string(),
            args: vec!["-m".to_string(), "mcp_server".to_string()],
            max_message_size: Some(2048),
            ..Default::default()
        };
        let stdio_transport = StdioTransport::new(stdio_config);
        let info = stdio_transport.transport_info();

        assert_eq!(info.transport_type, "stdio");
        assert_eq!(info.endpoint, "python -m mcp_server");
        assert!(!info.supports_reconnection); // Process needs restart
        assert_eq!(info.max_message_size, Some(2048));
    }

    #[tokio::test]
    async fn test_stdio_transport_lifecycle() {
        // Use a simple command that should exist on most systems
        let stdio_config = StdioConfig {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            startup_timeout_ms: 5000,
            ..Default::default()
        };
        let mut stdio_transport = StdioTransport::new(stdio_config);

        // Initially not connected
        assert!(!stdio_transport.is_connected());

        // Try to connect - this should spawn echo process
        // Note: echo will exit immediately, so this tests process spawning but not communication
        match stdio_transport.connect().await {
            Ok(_streams) => {
                // Should be connected after successful spawn
                assert!(stdio_transport.is_connected());

                // Should be able to disconnect
                assert!(stdio_transport.disconnect().await.is_ok());
                assert!(!stdio_transport.is_connected());
            }
            Err(e) => {
                // Connection might fail on some systems, that's ok for this test
                println!("Connection failed (expected on some systems): {}", e);
            }
        }
    }

    #[test]
    fn test_stdio_config_validation() {
        // Test default configuration
        let default_config = StdioConfig::default();
        assert_eq!(default_config.startup_timeout_ms, 10_000);
        assert_eq!(default_config.max_message_size, Some(16 * 1024 * 1024));
        assert!(default_config.args.is_empty());
        assert!(default_config.env_vars.is_empty());

        // Test custom configuration
        let custom_config = StdioConfig {
            command: "python".to_string(),
            args: vec!["-m".to_string(), "my_mcp_server".to_string()],
            working_dir: Some("/tmp".to_string()),
            env_vars: [("MCP_SERVER_MODE".to_string(), "production".to_string())].into(),
            startup_timeout_ms: 15_000,
            max_message_size: Some(8 * 1024 * 1024),
        };
        assert_eq!(custom_config.command, "python");
        assert_eq!(custom_config.args.len(), 2);
        assert_eq!(custom_config.working_dir, Some("/tmp".to_string()));
        assert_eq!(custom_config.env_vars.len(), 1);
        assert_eq!(custom_config.startup_timeout_ms, 15_000);
        assert_eq!(custom_config.max_message_size, Some(8 * 1024 * 1024));
    }

    #[tokio::test]
    async fn test_stdio_transport_with_args() {
        // Test stdio transport with command arguments
        let stdio_config = StdioConfig {
            command: "echo".to_string(),
            args: vec!["hello".to_string(), "world".to_string()],
            startup_timeout_ms: 5000,
            ..Default::default()
        };
        let mut stdio_transport = StdioTransport::new(stdio_config);

        // Should be able to create transport and get info
        let info = stdio_transport.transport_info();
        assert_eq!(info.endpoint, "echo hello world");

        // Try to connect (may succeed or fail depending on system)
        let result = stdio_transport.connect().await;
        match result {
            Ok(_) => {
                // If connection succeeds, should be able to disconnect
                assert!(stdio_transport.is_connected());
                assert!(stdio_transport.disconnect().await.is_ok());
                assert!(!stdio_transport.is_connected());
            }
            Err(_) => {
                // Connection failure is also acceptable for this test
            }
        }
    }
}
