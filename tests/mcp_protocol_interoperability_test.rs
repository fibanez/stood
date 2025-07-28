//! Integration tests for MCP client against real servers
//!
//! These tests validate the MCP implementation against actual external MCP server
//! implementations to ensure real-world protocol compliance and interoperability.

use serde_json::json;
use std::path::Path;
use stood::mcp::test_utils::*;
use stood::mcp::*;
use tokio::time::timeout;
use std::time::Duration;

/// Test MCP client against Python test server
#[tokio::test]
async fn test_python_server_basic_protocol() {
    // Skip test if Python is not available
    if !python_available().await {
        eprintln!("Skipping Python test - python3 not available");
        return;
    }

    let server_path = "examples/test-servers/python-server.py";
    if !Path::new(server_path).exists() {
        eprintln!("Skipping Python test - server script not found");
        return;
    }

    let config = TestServerConfig::python_server(server_path);
    let mut server = TestMCPServer::spawn(config).await.expect("Failed to spawn Python server");

    // Test basic protocol flow
    test_basic_protocol_flow(server.client()).await.expect("Protocol test failed");

    // Test Python-specific tools
    test_python_specific_tools(server.client()).await.expect("Python tools test failed");

    server.shutdown().await.expect("Failed to shutdown server");
}

/// Test MCP client against Node.js test server
#[tokio::test]
async fn test_nodejs_server_basic_protocol() {
    // Skip test if Node.js is not available
    if !nodejs_available().await {
        eprintln!("Skipping Node.js test - node not available");
        return;
    }

    let server_path = "examples/test-servers/node-server.js";
    if !Path::new(server_path).exists() {
        eprintln!("Skipping Node.js test - server script not found");
        return;
    }

    let config = TestServerConfig::nodejs_server(server_path);
    let mut server = TestMCPServer::spawn(config).await.expect("Failed to spawn Node.js server");

    // Test basic protocol flow
    test_basic_protocol_flow(server.client()).await.expect("Protocol test failed");

    // Test Node.js-specific tools
    test_nodejs_specific_tools(server.client()).await.expect("Node.js tools test failed");

    server.shutdown().await.expect("Failed to shutdown server");
}

/// Test concurrent connections to multiple servers
#[tokio::test]
async fn test_multiple_servers_concurrent() {
    let mut manager = TestServerManager::new();

    // Try to spawn both servers
    let mut servers_spawned = 0;

    if python_available().await && Path::new("examples/test-servers/python-server.py").exists() {
        let python_config = TestServerConfig::python_server("examples/test-servers/python-server.py");
        if manager.spawn_server(python_config).await.is_ok() {
            servers_spawned += 1;
        }
    }

    if nodejs_available().await && Path::new("examples/test-servers/node-server.js").exists() {
        let nodejs_config = TestServerConfig::nodejs_server("examples/test-servers/node-server.js");
        if manager.spawn_server(nodejs_config).await.is_ok() {
            servers_spawned += 1;
        }
    }

    if servers_spawned == 0 {
        eprintln!("Skipping concurrent test - no servers available");
        return;
    }

    // Test all servers sequentially to avoid borrowing issues
    let mut all_passed = true;
    for i in 0..manager.server_count() {
        if let Some(server) = manager.get_server(i) {
            let result = test_basic_protocol_flow(server.client()).await;
            if result.is_err() {
                eprintln!("Server {} failed test: {:?}", i, result.err());
                all_passed = false;
            }
        }
    }
    
    assert!(all_passed, "All servers should pass basic protocol test");

    manager.shutdown_all().await.expect("Failed to shutdown servers");
}

/// Test error handling with real servers
#[tokio::test]
async fn test_real_server_error_scenarios() {
    // Test with Python server if available
    if python_available().await && Path::new("examples/test-servers/python-server.py").exists() {
        let config = TestServerConfig::python_server("examples/test-servers/python-server.py");
        let mut server = TestMCPServer::spawn(config).await.expect("Failed to spawn Python server");
        
        test_error_scenarios(server.client()).await.expect("Error scenario test failed");
        
        server.shutdown().await.expect("Failed to shutdown server");
    }
}

/// Test server lifecycle and reconnection
#[tokio::test]
async fn test_server_lifecycle() {
    if !python_available().await || !Path::new("examples/test-servers/python-server.py").exists() {
        eprintln!("Skipping lifecycle test - Python server not available");
        return;
    }

    let config = TestServerConfig::python_server("examples/test-servers/python-server.py");
    let mut server = TestMCPServer::spawn(config).await.expect("Failed to spawn Python server");

    // Verify server is running
    assert!(server.is_running(), "Server should be running");

    // Test basic functionality
    let tools = server.client().list_tools().await.expect("Failed to list tools");
    assert!(!tools.is_empty(), "Server should have tools");

    // Shutdown server
    server.shutdown().await.expect("Failed to shutdown server");
}

/// Helper function to test basic MCP protocol flow
async fn test_basic_protocol_flow(client: &mut MCPClient) -> Result<(), MCPOperationError> {
    // Test tool listing
    let tools = client.list_tools().await?;
    assert!(!tools.is_empty(), "Server should have at least one tool");
    
    // Verify each tool has required fields
    for tool in &tools {
        assert!(!tool.name.is_empty(), "Tool name should not be empty");
        assert!(!tool.description.is_empty(), "Tool description should not be empty");
        assert!(tool.input_schema.is_object(), "Tool input schema should be an object");
    }

    // Test echo tool (should be available on both servers)
    let echo_result = client.call_tool("echo", Some(json!({"text": "test message"}))).await?;
    assert!(!echo_result.is_empty(), "Echo should return content");
    
    if let Content::Text(text_content) = &echo_result[0] {
        assert!(text_content.text.contains("test message"), "Echo should contain original text");
    } else {
        panic!("Echo should return text content");
    }

    Ok(())
}

/// Test Python-specific tools
async fn test_python_specific_tools(client: &mut MCPClient) -> Result<(), MCPOperationError> {
    // Test add tool
    let add_result = client.call_tool("add", Some(json!({"a": 5, "b": 3}))).await?;
    if let Content::Text(text_content) = &add_result[0] {
        assert!(text_content.text.contains("8"), "Add result should contain sum");
    }

    // Test get_time tool
    let time_result = client.call_tool("get_time", None).await?;
    if let Content::Text(text_content) = &time_result[0] {
        assert!(text_content.text.contains("Current time"), "Time result should contain current time");
    }

    Ok(())
}

/// Test Node.js-specific tools
async fn test_nodejs_specific_tools(client: &mut MCPClient) -> Result<(), MCPOperationError> {
    // Test multiply tool
    let multiply_result = client.call_tool("multiply", Some(json!({"a": 4, "b": 3}))).await?;
    if let Content::Text(text_content) = &multiply_result[0] {
        assert!(text_content.text.contains("12"), "Multiply result should contain product");
    }

    // Test get_env tool
    let env_result = client.call_tool("get_env", Some(json!({"name": "PATH"}))).await?;
    if let Content::Text(text_content) = &env_result[0] {
        // PATH should exist in most environments
        assert!(text_content.text.contains("PATH=") || text_content.text.contains("not found"));
    }

    Ok(())
}

/// Test error scenarios
async fn test_error_scenarios(client: &mut MCPClient) -> Result<(), MCPOperationError> {
    // Test calling non-existent tool
    let result = client.call_tool("nonexistent_tool", None).await;
    assert!(result.is_err(), "Should fail for non-existent tool");

    // Test calling tool with wrong parameters
    let _result = client.call_tool("echo", Some(json!({"wrong_param": "value"}))).await;
    // Note: Some servers might be lenient with parameters, so we don't assert failure

    Ok(())
}

/// Check if Python 3 is available
async fn python_available() -> bool {
    timeout(Duration::from_secs(5), async {
        tokio::process::Command::new("python3")
            .arg("--version")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    })
    .await
    .unwrap_or(false)
}

/// Check if Node.js is available
async fn nodejs_available() -> bool {
    timeout(Duration::from_secs(5), async {
        tokio::process::Command::new("node")
            .arg("--version")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    })
    .await
    .unwrap_or(false)
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    /// Benchmark tool calls against real servers
    #[tokio::test]
    async fn benchmark_tool_calls() {
        if !python_available().await || !Path::new("examples/test-servers/python-server.py").exists() {
            eprintln!("Skipping benchmark - Python server not available");
            return;
        }

        let config = TestServerConfig::python_server("examples/test-servers/python-server.py");
        let mut server = TestMCPServer::spawn(config).await.expect("Failed to spawn Python server");

        // Warm up
        let _ = server.client().list_tools().await;

        // Benchmark tool listing
        let start = Instant::now();
        for _ in 0..10 {
            server.client().list_tools().await.expect("Tool listing failed");
        }
        let list_duration = start.elapsed();
        println!("Tool listing: 10 calls in {:?} ({:?} per call)", 
                 list_duration, list_duration / 10);

        // Benchmark tool calls
        let start = Instant::now();
        for i in 0..20 {
            server.client().call_tool("echo", Some(json!({"text": format!("test {}", i)}))).await
                .expect("Tool call failed");
        }
        let call_duration = start.elapsed();
        println!("Tool calls: 20 calls in {:?} ({:?} per call)", 
                 call_duration, call_duration / 20);

        server.shutdown().await.expect("Failed to shutdown server");
    }

    /// Test high-frequency tool calls
    #[tokio::test]
    async fn test_high_frequency_calls() {
        if !python_available().await || !Path::new("examples/test-servers/python-server.py").exists() {
            eprintln!("Skipping high frequency test - Python server not available");
            return;
        }

        let config = TestServerConfig::python_server("examples/test-servers/python-server.py");
        let mut server = TestMCPServer::spawn(config).await.expect("Failed to spawn Python server");

        // Make many rapid calls sequentially to avoid borrowing issues
        let mut successes = 0;
        for i in 0..50 {
            match server.client().call_tool("add", Some(json!({"a": i, "b": 1}))).await {
                Ok(_) => successes += 1,
                Err(e) => eprintln!("Tool call {} failed: {}", i, e),
            }
        }
        
        assert!(successes >= 40, "At least 40 out of 50 rapid calls should succeed (got {})", successes);

        server.shutdown().await.expect("Failed to shutdown server");
    }
}