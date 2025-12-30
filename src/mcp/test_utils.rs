//! Test utilities for MCP server lifecycle management
//!
//! This module provides utilities for managing test MCP servers during integration testing,
//! including spawning external servers, managing their lifecycle, and communicating with them.

use crate::error::StoodError;
use crate::mcp::{MCPClient, MCPClientConfig, StdioTransport};
use crate::Result;
use std::path::Path;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::time::{timeout, Duration};

/// Configuration for a test MCP server
#[derive(Debug, Clone)]
pub struct TestServerConfig {
    /// Display name for the server
    pub name: String,
    /// Command to execute the server
    pub command: String,
    /// Arguments to pass to the server
    pub args: Vec<String>,
    /// Working directory for the server process
    pub working_dir: Option<String>,
    /// Environment variables to set
    pub env_vars: Vec<(String, String)>,
    /// Timeout for server startup
    pub startup_timeout: Duration,
}

impl TestServerConfig {
    /// Create configuration for the Python test server
    pub fn python_server(server_path: impl AsRef<Path>) -> Self {
        Self {
            name: "Python MCP Test Server".to_string(),
            command: "python3".to_string(),
            args: vec![server_path.as_ref().to_string_lossy().to_string()],
            working_dir: None,
            env_vars: Vec::new(),
            startup_timeout: Duration::from_secs(5),
        }
    }

    /// Create configuration for the Node.js test server
    pub fn nodejs_server(server_path: impl AsRef<Path>) -> Self {
        Self {
            name: "Node.js MCP Test Server".to_string(),
            command: "node".to_string(),
            args: vec![server_path.as_ref().to_string_lossy().to_string()],
            working_dir: None,
            env_vars: Vec::new(),
            startup_timeout: Duration::from_secs(5),
        }
    }

    /// Create configuration for a Docker-based test server
    pub fn docker_server(image_name: &str, container_name: &str) -> Self {
        Self {
            name: format!("Docker MCP Server ({})", container_name),
            command: "docker".to_string(),
            args: vec![
                "run".to_string(),
                "--rm".to_string(),
                "-i".to_string(),
                "--name".to_string(),
                container_name.to_string(),
                image_name.to_string(),
            ],
            working_dir: None,
            env_vars: Vec::new(),
            startup_timeout: Duration::from_secs(10),
        }
    }

    /// Add environment variable to the server configuration
    pub fn with_env_var(mut self, key: String, value: String) -> Self {
        self.env_vars.push((key, value));
        self
    }

    /// Set working directory for the server process
    pub fn with_working_dir(mut self, dir: String) -> Self {
        self.working_dir = Some(dir);
        self
    }

    /// Set startup timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = timeout;
        self
    }
}

/// Managed test MCP server instance
pub struct TestMCPServer {
    /// Server configuration
    pub config: TestServerConfig,
    /// Child process handle
    process: Child,
    /// MCP client connected to the server
    client: MCPClient,
}

impl TestMCPServer {
    /// Spawn a new test MCP server and establish connection
    pub async fn spawn(config: TestServerConfig) -> Result<Self> {
        // Build command
        let mut command = Command::new(&config.command);
        command.args(&config.args);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Set working directory if specified
        if let Some(ref dir) = config.working_dir {
            command.current_dir(dir);
        }

        // Set environment variables
        for (key, value) in &config.env_vars {
            command.env(key, value);
        }

        // Spawn process
        let mut process = command
            .spawn()
            .map_err(|e| StoodError::ConfigurationError {
                message: format!("Failed to spawn {} server: {}", config.name, e),
            })?;

        // Create stdio transport from process handles
        let stdin = process
            .stdin
            .take()
            .ok_or_else(|| StoodError::ConfigurationError {
                message: "Failed to get stdin handle".to_string(),
            })?;

        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| StoodError::ConfigurationError {
                message: "Failed to get stdout handle".to_string(),
            })?;

        let transport = StdioTransport::from_handles(stdin, stdout);
        let client_config = MCPClientConfig::default();
        let mut client = MCPClient::new(client_config, Box::new(transport));

        // Initialize connection with timeout
        let init_result = timeout(config.startup_timeout, async { client.connect().await }).await;

        match init_result {
            Ok(Ok(_)) => {
                tracing::info!("Successfully initialized {}", config.name);
                Ok(Self {
                    config,
                    process,
                    client,
                })
            }
            Ok(Err(e)) => {
                drop(process.kill());
                Err(StoodError::ConfigurationError {
                    message: format!("Failed to initialize {}: {}", config.name, e),
                })
            }
            Err(_) => {
                drop(process.kill());
                Err(StoodError::ConfigurationError {
                    message: format!(
                        "Timeout initializing {} ({}s)",
                        config.name,
                        config.startup_timeout.as_secs()
                    ),
                })
            }
        }
    }

    /// Get reference to the MCP client
    pub fn client(&mut self) -> &mut MCPClient {
        &mut self.client
    }

    /// Check if the server process is still running
    pub fn is_running(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(Some(_)) => false, // Process has exited
            Ok(None) => true,     // Process is still running
            Err(_) => false,      // Error checking status
        }
    }

    /// Get the server name for logging/debugging
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Gracefully shutdown the server
    pub async fn shutdown(mut self) -> Result<()> {
        // Disconnect client first
        let _ = self.client.disconnect().await;

        // Try to terminate gracefully
        let _ = self.process.kill().await;

        // Wait for process to exit
        match self.process.wait().await {
            Ok(status) => {
                tracing::info!("{} exited with status: {}", self.config.name, status);
                Ok(())
            }
            Err(e) => {
                tracing::warn!("Error waiting for {} to exit: {}", self.config.name, e);
                Ok(())
            }
        }
    }
}

impl Drop for TestMCPServer {
    fn drop(&mut self) {
        // Ensure process is terminated when dropped
        // Note: kill() is not async, so we use start_kill() for graceful termination attempt
        if let Err(e) = self.process.start_kill() {
            tracing::warn!("Failed to terminate process: {}", e);
        }
    }
}

/// Test server manager for handling multiple test servers
pub struct TestServerManager {
    servers: Vec<TestMCPServer>,
}

impl TestServerManager {
    /// Create a new test server manager
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
        }
    }

    /// Spawn and add a server to management
    pub async fn spawn_server(&mut self, config: TestServerConfig) -> Result<usize> {
        let server = TestMCPServer::spawn(config).await?;
        let index = self.servers.len();
        self.servers.push(server);
        Ok(index)
    }

    /// Get mutable reference to a server by index
    pub fn get_server(&mut self, index: usize) -> Option<&mut TestMCPServer> {
        self.servers.get_mut(index)
    }

    /// Get number of managed servers
    pub fn server_count(&self) -> usize {
        self.servers.len()
    }

    /// Check if all servers are running
    pub fn all_running(&mut self) -> bool {
        self.servers.iter_mut().all(|s| s.is_running())
    }

    /// Shutdown all managed servers
    pub async fn shutdown_all(self) -> Result<()> {
        for server in self.servers {
            if let Err(e) = server.shutdown().await {
                tracing::warn!("Error shutting down server: {}", e);
            }
        }
        Ok(())
    }
}

impl Default for TestServerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for common test scenarios
pub mod utils {
    use super::*;

    /// Spawn the Python test server from examples directory
    pub async fn spawn_python_test_server() -> Result<TestMCPServer> {
        let server_path = "examples/test-servers/python-server.py";
        let config = TestServerConfig::python_server(server_path);
        TestMCPServer::spawn(config).await
    }

    /// Spawn the Node.js test server from examples directory
    pub async fn spawn_nodejs_test_server() -> Result<TestMCPServer> {
        let server_path = "examples/test-servers/node-server.js";
        let config = TestServerConfig::nodejs_server(server_path);
        TestMCPServer::spawn(config).await
    }

    /// Spawn both Python and Node.js test servers
    pub async fn spawn_all_test_servers() -> Result<TestServerManager> {
        let mut manager = TestServerManager::new();

        // Try to spawn Python server
        let python_config =
            TestServerConfig::python_server("examples/test-servers/python-server.py");
        if manager.spawn_server(python_config).await.is_ok() {
            tracing::info!("Python test server spawned successfully");
        } else {
            tracing::warn!("Failed to spawn Python test server");
        }

        // Try to spawn Node.js server
        let nodejs_config = TestServerConfig::nodejs_server("examples/test-servers/node-server.js");
        if manager.spawn_server(nodejs_config).await.is_ok() {
            tracing::info!("Node.js test server spawned successfully");
        } else {
            tracing::warn!("Failed to spawn Node.js test server");
        }

        Ok(manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_creation() {
        let config = TestServerConfig::python_server("test.py");
        assert_eq!(config.command, "python3");
        assert_eq!(config.args, vec!["test.py"]);
        assert_eq!(config.name, "Python MCP Test Server");
    }

    #[test]
    fn test_server_config_with_env() {
        let config = TestServerConfig::nodejs_server("test.js")
            .with_env_var("NODE_ENV".to_string(), "test".to_string())
            .with_working_dir("/tmp".to_string());

        assert_eq!(
            config.env_vars,
            vec![("NODE_ENV".to_string(), "test".to_string())]
        );
        assert_eq!(config.working_dir, Some("/tmp".to_string()));
    }

    #[test]
    fn test_docker_config() {
        let config = TestServerConfig::docker_server("mcp-test:latest", "test-container");
        assert_eq!(config.command, "docker");
        assert!(config.args.contains(&"mcp-test:latest".to_string()));
        assert!(config.args.contains(&"test-container".to_string()));
    }
}
