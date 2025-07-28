//! Basic end-to-end tests for the Stood agentic CLI
//!
//! These tests verify fundamental CLI functionality like startup, basic commands,
//! and graceful shutdown.

use super::lib::*;

#[tokio::test]
async fn test_cli_startup_and_exit() -> Result<()> {
    // Skip if no AWS credentials (CLI requires them)
    if !check_aws_credentials() {
        println!("⚠️  Skipping CLI startup test - AWS credentials not available");
        return Ok(());
    }

    println!("🚀 Testing CLI startup and graceful exit...");

    let mut session = spawn_cli().await?;

    // CLI should have started and shown the banner
    println!("✅ CLI started successfully");

    // Send help command to verify CLI is responsive
    session.send_line("help").await?;
    session.expect("Commands:").await?;
    println!("✅ Help command works");

    // Exit gracefully
    session.send_line("exit").await?;
    session.expect("Goodbye").await?;
    println!("✅ Exit command works");

    // Wait for process to actually exit
    session.wait_for_exit().await?;
    println!("✅ CLI exited successfully");

    Ok(())
}

#[tokio::test]
async fn test_basic_commands() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping basic commands test - AWS credentials not available");
        return Ok(());
    }

    println!("📝 Testing basic CLI commands...");

    let mut session = spawn_cli().await?;

    // Test help command
    session.send_line("help").await?;
    session.expect("Commands:").await?;
    session.expect("tools").await?;
    session.expect("exit").await?;
    println!("✅ Help command shows expected content");

    // Test tools command
    session.send_line("tools").await?;
    session.expect("Available Tools").await?;
    session.expect("calculator").await?;
    println!("✅ Tools command shows tool list");

    // Test status command
    session.send_line("status").await?;
    session.expect("Current Status").await?;
    session.expect("Mode:").await?;
    println!("✅ Status command shows current state");

    // Exit
    session.send_line("quit").await?;
    session.expect("Goodbye").await?;

    Ok(())
}

#[tokio::test]
async fn test_invalid_commands() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping invalid commands test - AWS credentials not available");
        return Ok(());
    }

    println!("🚨 Testing invalid command handling...");

    let mut session = spawn_cli().await?;

    // Test unknown command - CLI should remain responsive
    session.send_line("unknown_command_xyz").await?;
    // CLI might just ignore unknown commands or pass them to the LLM

    // Verify CLI is still responsive by running a known command
    session.send_line("help").await?;
    session.expect("Commands:").await?;
    println!("✅ CLI remains responsive after invalid input");

    session.send_line("exit").await?;
    session.expect("Goodbye").await?;

    Ok(())
}

#[tokio::test]
async fn test_mode_switching() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping mode switching test - AWS credentials not available");
        return Ok(());
    }

    println!("🔄 Testing mode switching...");

    let mut session = spawn_cli().await?;

    // Test agentic mode toggle
    session.send_line("agentic off").await?;
    session.expect("Standard mode enabled").await?;
    println!("✅ Switched to standard mode");

    session.send_line("agentic on").await?;
    session.expect("Agentic mode enabled").await?;
    println!("✅ Switched to agentic mode");

    // Test streaming mode toggle
    session.send_line("streaming on").await?;
    session.expect("Streaming responses enabled").await?;
    println!("✅ Enabled streaming");

    session.send_line("streaming off").await?;
    session.expect("Streaming responses disabled").await?;
    println!("✅ Disabled streaming");

    // Verify status reflects changes
    session.send_line("status").await?;
    session.expect("Current Status").await?;

    session.send_line("exit").await?;
    session.expect("Goodbye").await?;

    Ok(())
}

#[tokio::test]
async fn test_conversation_clear() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping conversation clear test - AWS credentials not available");
        return Ok(());
    }

    println!("🧹 Testing conversation history clearing...");

    let mut session = spawn_cli().await?;

    // Have a short conversation to build history
    session.send_line("Hello").await?;
    session.expect("Assistant:").await?;

    // Clear conversation
    session.send_line("clear").await?;
    session.expect("Conversation history cleared").await?;
    println!("✅ Conversation cleared successfully");

    // Verify status shows empty history
    session.send_line("status").await?;
    session.expect("History: 0 messages").await?;
    println!("✅ Status confirms empty history");

    session.send_line("exit").await?;
    session.expect("Goodbye").await?;

    Ok(())
}

#[tokio::test]
async fn test_cli_with_debug_mode() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping debug mode test - AWS credentials not available");
        return Ok(());
    }

    println!("🔍 Testing CLI with debug mode...");

    let mut config = TestConfig::default();
    config.debug_mode = true;
    let mut session = spawn_cli_with_config(config).await?;

    // Debug mode should show more verbose output
    session.send_line("help").await?;
    session.expect("Commands:").await?;

    session.send_line("exit").await?;
    session.expect("Goodbye").await?;

    println!("✅ Debug mode CLI works");

    Ok(())
}

#[tokio::test]
async fn test_multiple_sessions() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping multiple sessions test - AWS credentials not available");
        return Ok(());
    }

    println!("🔀 Testing multiple CLI sessions...");

    // Start two sessions concurrently
    let mut session1 = spawn_cli().await?;
    let mut session2 = spawn_cli().await?;

    // Both should be responsive
    session1.send_line("help").await?;
    session2.send_line("tools").await?;

    session1.expect("Commands:").await?;
    session2.expect("Available Tools").await?;

    println!("✅ Multiple sessions work independently");

    // Clean up
    session1.send_line("exit").await?;
    session2.send_line("exit").await?;

    session1.expect("Goodbye").await?;
    session2.expect("Goodbye").await?;

    Ok(())
}

#[tokio::test]
async fn test_graceful_interrupt_handling() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping interrupt handling test - AWS credentials not available");
        return Ok(());
    }

    println!("⚡ Testing graceful interrupt handling...");

    let mut session = spawn_cli().await?;

    // Verify CLI is running
    session.send_line("help").await?;
    session.expect("Commands:").await?;

    // Send Ctrl+C
    session.send_control_c().await?;
    session.expect("Goodbye").await?;

    println!("✅ Ctrl+C handled gracefully");

    Ok(())
}

// Test that the binary path detection works
#[test]
fn test_binary_detection() {
    let binary_path = get_cli_binary_path();
    assert!(
        binary_path.is_ok(),
        "Should be able to determine binary path"
    );

    let path = binary_path.unwrap();
    println!("Detected binary path: {}", path);

    // Should be either a path to the binary or "cargo"
    assert!(path.contains("stood-agentic-cli") || path == "cargo");
}

// Test configuration creation
#[test]
fn test_config_customization() {
    let mut config = TestConfig::default();
    config.model = "claude-sonnet-3".to_string();
    config.debug_mode = true;
    config.extra_args.push("--max-cycles".to_string());
    config.extra_args.push("5".to_string());

    assert_eq!(config.model, "claude-sonnet-3");
    assert!(config.debug_mode);
    assert_eq!(config.extra_args, vec!["--max-cycles", "5"]);
}
