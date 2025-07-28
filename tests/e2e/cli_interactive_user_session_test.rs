//! End-to-end tests for interactive CLI functionality
//!
//! This module tests the core interactive features of the Stood agentic CLI,
//! including basic commands, conversation flow, and mode switching.

use crate::e2e::*;
use std::time::Duration;

/// Test basic CLI commands functionality
/// Test basic CLI commands functionality
#[tokio::test]
async fn test_basic_help_command() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test help command
    session.send_line("help").await?;
    session.expect("Commands:").await?;

    // Exit gracefully
    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test tools list command
#[tokio::test]
async fn test_tools_command() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test tools command
    session.send_line("tools").await?;
    session.expect("Available tools:").await?;

    // Should show built-in tools
    session.expect("calculator").await?;
    session.expect("current_time").await?;
    session.expect("file_read").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test status command
#[tokio::test]
async fn test_status_command() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test status command
    session.send_line("status").await?;
    session.expect("Status:").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test clear command
#[tokio::test]
async fn test_clear_command() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Have a conversation first
    session.send_line("Hello").await?;
    session.expect("Assistant:").await?;

    // Clear the conversation
    session.send_line("clear").await?;
    session.expect("Conversation cleared").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test exit and quit commands
#[tokio::test]
async fn test_exit_commands() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    // Test exit command
    let mut session = spawn_cli().await?;
    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    // Test quit command
    let mut session = spawn_cli().await?;
    session.send_line("quit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test multi-turn conversation with context preservation
#[tokio::test]
async fn test_multi_turn_conversation() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Start a conversation
    session.send_line("My name is Alice").await?;
    session.expect("Assistant:").await?;

    // Follow up with context reference
    session.send_line("What is my name?").await?;
    session.expect("Alice").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test conversation flow with calculator tool
#[tokio::test]
async fn test_conversation_with_tool_usage() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Ask for a calculation
    session.send_line("What is 15 * 23?").await?;
    session.expect("345").await?;

    // Reference previous calculation
    session
        .send_line("What was that calculation again?")
        .await?;
    session.expect("15").await?;
    session.expect("23").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test agentic mode switching
#[tokio::test]
async fn test_agentic_mode_switching() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Turn off agentic mode
    session.send_line("agentic off").await?;
    session.expect("Agentic mode: off").await?;

    // Turn on agentic mode
    session.send_line("agentic on").await?;
    session.expect("Agentic mode: on").await?;

    // Check status to verify mode persistence
    session.send_line("status").await?;
    session.expect("Agentic mode: on").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test streaming mode switching
#[tokio::test]
async fn test_streaming_mode_switching() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Turn off streaming mode
    session.send_line("streaming off").await?;
    session.expect("Streaming: off").await?;

    // Turn on streaming mode
    session.send_line("streaming on").await?;
    session.expect("Streaming: on").await?;

    // Check status to verify mode persistence
    session.send_line("status").await?;
    session.expect("Streaming: on").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test conversation history growth and preservation
#[tokio::test]
async fn test_conversation_history_preservation() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Have multiple exchanges to build history
    for i in 1..=5 {
        session
            .send_line(&format!("This is message number {}", i))
            .await?;
        session.expect("Assistant:").await?;
    }

    // Reference an earlier message
    session
        .send_line("What was the first message I sent?")
        .await?;
    session.expect("1").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test context command for conversation state inspection
#[tokio::test]
async fn test_context_management_inspection() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Have some conversation to build context
    session
        .send_line("Hello, I'm testing the context management")
        .await?;
    session.expect("Assistant:").await?;

    session
        .send_line("Can you tell me about Rust programming?")
        .await?;
    session.expect("Assistant:").await?;

    // Check context status
    session.send_line("context").await?;
    session.expect("Context").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test graceful handling when CLI receives invalid commands
#[tokio::test]
async fn test_invalid_command_handling() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Send an invalid command
    session.send_line("invalid_command_xyz").await?;
    session.expect("Unknown command").await?;

    // Verify CLI is still responsive
    session.send_line("help").await?;
    session.expect("Commands:").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test conversation continuation after errors
#[tokio::test]
async fn test_conversation_after_error() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Cause an error (invalid command)
    session.send_line("invalid_command").await?;
    session.expect("Unknown command").await?;

    // Continue with normal conversation
    session.send_line("Hello").await?;
    session.expect("Assistant:").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test multiple mode switches in sequence
#[tokio::test]
async fn test_multiple_mode_switches() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test sequence of mode changes
    session.send_line("agentic off").await?;
    session.expect("Agentic mode: off").await?;

    session.send_line("streaming off").await?;
    session.expect("Streaming: off").await?;

    session.send_line("agentic on").await?;
    session.expect("Agentic mode: on").await?;

    session.send_line("streaming on").await?;
    session.expect("Streaming: on").await?;

    // Verify final state
    session.send_line("status").await?;
    session.expect("Agentic mode: on").await?;
    session.expect("Streaming: on").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test CLI responsiveness with rapid commands
#[tokio::test]
async fn test_rapid_command_sequence() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Send multiple commands in quick succession
    let commands = vec!["help", "tools", "status", "clear"];

    for cmd in commands {
        session.send_line(cmd).await?;
        // Give a shorter timeout for rapid testing
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Verify CLI is still responsive
    session.send_line("help").await?;
    session.expect("Commands:").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test conversation with time tool
#[tokio::test]
async fn test_time_tool_conversation() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Ask for current time
    session.send_line("What time is it?").await?;
    session.expect("UTC").await?;

    // Follow up on the time request
    session.send_line("What timezone was that time in?").await?;
    session.expect("UTC").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}
