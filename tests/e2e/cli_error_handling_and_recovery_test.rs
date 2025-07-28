//! End-to-end tests for error handling and recovery mechanisms
//!
//! This module tests how the CLI handles various error conditions and
//! recovers gracefully while maintaining responsiveness.

use crate::e2e::*;
use std::time::Duration;

/// Test invalid input handling
#[tokio::test]
async fn test_invalid_input_handling() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test completely invalid command
    session
        .send_line("this_is_not_a_valid_command_12345")
        .await?;
    session.expect("Unknown command").await?;

    // Verify CLI remains responsive
    session.send_line("help").await?;
    session.expect("Commands:").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test tool execution failures
#[tokio::test]
async fn test_tool_execution_failures() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test file operation with non-existent file
    session
        .send_line("Read the file /this/path/does/not/exist/file.txt")
        .await?;
    session.expect("Error").await?;

    // Test invalid mathematical expression
    session.send_line("Calculate 1 / 0").await?;
    // Should handle division by zero gracefully

    // Verify CLI is still responsive after errors
    session.send_line("What is 2 + 2?").await?;
    session.expect("4").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test network timeout scenarios
#[tokio::test]
async fn test_network_timeout_handling() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test HTTP request to a slow/non-responsive endpoint
    session
        .send_line("Make a GET request to httpbin.org/delay/30")
        .await?;

    // Wait for timeout or error response
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Verify CLI is still responsive
    session.send_line("help").await?;
    session.expect("Commands:").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test conversation state preservation during errors
#[tokio::test]
async fn test_conversation_state_preservation() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Establish conversation context
    session.send_line("My favorite number is 42").await?;
    session.expect("Assistant:").await?;

    // Cause an error
    session.send_line("invalid_command_xyz").await?;
    session.expect("Unknown command").await?;

    // Verify context is preserved
    session.send_line("What was my favorite number?").await?;
    session.expect("42").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test CLI responsiveness under stress
#[tokio::test]
async fn test_rapid_error_recovery() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Send multiple invalid commands rapidly
    let invalid_commands = vec!["invalid1", "invalid2", "invalid3", "invalid4", "invalid5"];

    for cmd in invalid_commands {
        session.send_line(cmd).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Verify CLI is still responsive
    session.send_line("help").await?;
    session.expect("Commands:").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test resource exhaustion scenarios
#[tokio::test]
async fn test_resource_exhaustion_handling() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Send very long message to test memory handling
    let long_message = "A".repeat(10000);
    session
        .send_line(&format!("Process this long text: {}", long_message))
        .await?;

    // Wait for response
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Verify CLI is still responsive
    session.send_line("What is 1 + 1?").await?;
    session.expect("2").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test context window management under pressure
#[tokio::test]
async fn test_context_window_stress() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Fill up context with many messages
    for i in 1..=50 {
        session
            .send_line(&format!("Message number {} with some content", i))
            .await?;
        // Don't wait for full response to stress the system
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Test context management
    session.send_line("context").await?;
    session.expect("Context").await?;

    // Verify system is still functional
    session.send_line("What is 5 * 5?").await?;
    session.expect("25").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test graceful degradation of features
#[tokio::test]
async fn test_feature_degradation() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Turn off agentic mode to test basic functionality
    session.send_line("agentic off").await?;
    session.expect("Agentic mode: off").await?;

    // Verify basic commands still work
    session.send_line("help").await?;
    session.expect("Commands:").await?;

    session.send_line("tools").await?;
    session.expect("Available tools:").await?;

    // Turn agentic mode back on
    session.send_line("agentic on").await?;
    session.expect("Agentic mode: on").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test error handling with malformed requests
#[tokio::test]
async fn test_malformed_request_handling() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test request with special characters
    session.send_line("Calculate @#$%^&*()").await?;

    // Wait for error handling
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Test Unicode characters
    session.send_line("Process this: 🚀🎉🔥💻").await?;

    // Wait for response
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Verify CLI is still responsive
    session.send_line("What is 3 + 3?").await?;
    session.expect("6").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test concurrent error scenarios
#[tokio::test]
async fn test_concurrent_error_handling() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Send multiple problematic requests
    session.send_line("Read /nonexistent/file1.txt").await?;
    session.send_line("Read /nonexistent/file2.txt").await?;
    session.send_line("Calculate invalid_math").await?;

    // Wait for all errors to be processed
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Verify system recovered
    session.send_line("What is 7 * 7?").await?;
    session.expect("49").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test interruption handling
#[tokio::test]
async fn test_interruption_handling() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Send a request that might take time
    session
        .send_line("Make a GET request to httpbin.org/delay/5")
        .await?;

    // Wait a bit then try to send another command
    tokio::time::sleep(Duration::from_secs(2)).await;
    session.send_line("help").await?;

    // Verify CLI handles concurrent requests gracefully
    tokio::time::sleep(Duration::from_secs(8)).await;

    session.send_line("What is 8 + 8?").await?;
    session.expect("16").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test memory usage stability during errors
#[tokio::test]
async fn test_memory_stability() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Cause multiple types of errors
    let long_text = "Process this extremely long text that goes on and on and repeats".repeat(100);
    let error_commands = vec![
        "Read /tmp/nonexistent.txt",
        "Calculate undefined_variable",
        "invalid_command_test",
        "Make a GET request to invalid.url.test",
        long_text.as_str(),
    ];

    for cmd in error_commands {
        session.send_line(cmd).await?;
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // Wait for all errors to be processed
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Verify system is still functional
    session.send_line("What is 9 * 9?").await?;
    session.expect("81").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}
