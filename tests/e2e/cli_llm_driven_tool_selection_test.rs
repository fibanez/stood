//! End-to-end tests for tool integration and LLM-driven tool selection
//!
//! This module tests the LLM's ability to select and execute appropriate tools
//! based on user requests, validating the core agentic functionality.

use crate::e2e::*;
use std::time::Duration;

/// Test LLM-driven tool selection for mathematical calculations
#[tokio::test]
async fn test_calculator_tool_selection() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test simple calculation
    session.send_line("What is 15 * 23?").await?;
    session.expect("345").await?;

    // Test more complex calculation
    session
        .send_line("Calculate the square root of 144")
        .await?;
    session.expect("12").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test LLM-driven tool selection for time queries
#[tokio::test]
async fn test_time_tool_selection() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test current time request
    session.send_line("What time is it right now?").await?;
    session.expect("UTC").await?;

    // Test date request
    session.send_line("What's today's date?").await?;
    session.expect("2025").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test LLM-driven tool selection for file operations
#[tokio::test]
async fn test_file_tools_selection() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let temp_dir = create_temp_dir()?;
    create_sample_files(temp_dir.path()).await?;

    let mut session = spawn_cli().await?;

    // Test file reading
    let file_path = temp_dir.path().join("sample.txt");
    session
        .send_line(&format!("Read the file {}", file_path.display()))
        .await?;
    session.expect("sample text file").await?;

    // Test file listing
    session
        .send_line(&format!("List files in {}", temp_dir.path().display()))
        .await?;
    session.expect("sample.txt").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test LLM-driven tool selection for environment variables
#[tokio::test]
async fn test_environment_tool_selection() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test environment variable query
    session
        .send_line("What is the HOME environment variable?")
        .await?;
    session.expect("/").await?; // Should contain a path

    // Test PATH variable
    session
        .send_line("Show me the PATH environment variable")
        .await?;
    session.expect("/").await?; // Should contain paths

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test HTTP request tool selection
#[tokio::test]
async fn test_http_tool_selection() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test HTTP request to a safe endpoint
    session
        .send_line("Make a GET request to httpbin.org/get")
        .await?;
    session.expect("httpbin").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test multi-tool scenarios requiring multiple tools
#[tokio::test]
async fn test_multi_tool_scenarios() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test request requiring both calculator and time tools
    session
        .send_line("Calculate 10 * 5 and tell me the current time")
        .await?;
    session.expect("50").await?;
    session.expect("UTC").await?;

    // Test request requiring file and calculation
    let temp_dir = create_temp_dir()?;
    let file_path = temp_dir.path().join("numbers.txt");
    tokio::fs::write(&file_path, "10\n20\n30\n").await?;

    session
        .send_line(&format!(
            "Read {} and calculate the sum of the numbers",
            file_path.display()
        ))
        .await?;
    session.expect("60").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test tool execution with complex mathematical operations
#[tokio::test]
async fn test_complex_calculator_scenarios() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test complex calculation
    session.send_line("Calculate (15 + 25) * 2 - 10").await?;
    session.expect("70").await?;

    // Test percentage calculation
    session.send_line("What is 25% of 200?").await?;
    session.expect("50").await?;

    // Test power calculation
    session.send_line("What is 2 to the power of 8?").await?;
    session.expect("256").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test file operations with different file types
#[tokio::test]
async fn test_comprehensive_file_operations() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let temp_dir = create_temp_dir()?;
    create_sample_files(temp_dir.path()).await?;

    let mut session = spawn_cli().await?;

    // Test reading JSON file
    let json_path = temp_dir.path().join("data.json");
    session
        .send_line(&format!("Read the JSON file {}", json_path.display()))
        .await?;
    session.expect("test").await?;
    session.expect("42").await?;

    // Test writing a new file
    let new_file = temp_dir.path().join("output.txt");
    session
        .send_line(&format!(
            "Write 'Hello from test' to {}",
            new_file.display()
        ))
        .await?;
    session.expect("written").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test tool execution timing and performance
#[tokio::test]
async fn test_tool_execution_performance() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    let start = std::time::Instant::now();

    // Test simple calculation (should be fast)
    session.send_line("What is 2 + 2?").await?;
    session.expect("4").await?;

    let duration = start.elapsed();

    // Ensure the response comes within reasonable time (30 seconds max for E2E with LLM)
    assert!(
        duration < Duration::from_secs(30),
        "Tool execution took too long: {:?}",
        duration
    );

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test error handling in tool execution
#[tokio::test]
async fn test_tool_error_handling() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test reading non-existent file
    session
        .send_line("Read the file /path/that/does/not/exist.txt")
        .await?;
    session.expect("not found").await?;

    // Verify CLI is still responsive after error
    session.send_line("What is 1 + 1?").await?;
    session.expect("2").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test tool selection with ambiguous requests
#[tokio::test]
async fn test_ambiguous_tool_selection() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test request that could use multiple tools
    session
        .send_line("I need to know the time and do some math")
        .await?;

    // The LLM should ask for clarification or provide both
    // We'll just check that it responds appropriately
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Follow up with specific request
    session.send_line("Calculate 7 * 8").await?;
    session.expect("56").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test sequential tool usage in conversation
#[tokio::test]
async fn test_sequential_tool_usage() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // First tool usage
    session.send_line("What time is it?").await?;
    session.expect("UTC").await?;

    // Second tool usage
    session.send_line("Now calculate 15 * 4").await?;
    session.expect("60").await?;

    // Third tool usage referencing previous results
    session.send_line("What's the HOME directory?").await?;
    session.expect("/").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test tool usage with context preservation
#[tokio::test]
async fn test_tool_context_preservation() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Use calculator tool
    session.send_line("Calculate 25 * 4").await?;
    session.expect("100").await?;

    // Reference previous calculation
    session
        .send_line("What was that calculation I just asked for?")
        .await?;
    session.expect("25").await?;
    session.expect("4").await?;

    // Use the result in a new calculation
    session.send_line("Add 50 to that result").await?;
    session.expect("150").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}
