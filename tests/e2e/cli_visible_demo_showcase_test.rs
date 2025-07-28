//! Visible demonstration tests to show CLI interactions
//!
//! These tests use visible_mode to show exactly what's happening
//! during CLI interactions for debugging and demonstration purposes.

use crate::e2e::*;
use std::time::Duration;

/// Demo test with visible interactions - shows all CLI communication
#[tokio::test]
async fn test_visible_help_demo() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    println!("🎭 DEMO: Visible CLI Interaction Test");
    println!("=====================================");

    let mut session = spawn_cli_visible().await?;

    println!("\n💬 Testing help command...");
    session.send_line("help").await?;
    session.expect("Commands:").await?;

    println!("\n💬 Testing tools command...");
    session.send_line("tools").await?;
    session.expect("Available tools:").await?;

    println!("\n💬 Testing status command...");
    session.send_line("status").await?;
    session.expect("Status:").await?;

    println!("\n💬 Exiting gracefully...");
    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    println!("\n✅ Demo completed successfully!");
    Ok(())
}

/// Demo test showing calculator tool interaction
#[tokio::test]
async fn test_visible_calculator_demo() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    println!("🧮 DEMO: Calculator Tool Interaction");
    println!("====================================");

    let mut session = spawn_cli_visible().await?;

    println!("\n💬 Testing simple calculation...");
    session.send_line("What is 15 * 23?").await?;
    session.expect("345").await?;

    println!("\n💬 Testing complex calculation...");
    session
        .send_line("Calculate the square root of 144")
        .await?;
    session.expect("12").await?;

    println!("\n💬 Exiting...");
    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    println!("\n✅ Calculator demo completed!");
    Ok(())
}

/// Demo test showing conversation flow
#[tokio::test]
async fn test_visible_conversation_demo() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    println!("💭 DEMO: Conversation Flow");
    println!("=========================");

    let mut session = spawn_cli_visible().await?;

    println!("\n💬 Starting conversation...");
    session.send_line("My name is Alice").await?;
    session.expect("Assistant:").await?;

    println!("\n💬 Testing context preservation...");
    session.send_line("What is my name?").await?;
    session.expect("Alice").await?;

    println!("\n💬 Ending conversation...");
    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    println!("\n✅ Conversation demo completed!");
    Ok(())
}

/// Demo test showing error handling
#[tokio::test]
async fn test_visible_error_demo() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    println!("❌ DEMO: Error Handling");
    println!("======================");

    let mut session = spawn_cli_visible().await?;

    println!("\n💬 Testing invalid command...");
    session.send_line("invalid_command_xyz").await?;
    session.expect("Unknown command").await?;

    println!("\n💬 Verifying CLI still responsive...");
    session.send_line("help").await?;
    session.expect("Commands:").await?;

    println!("\n💬 Exiting...");
    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    println!("\n✅ Error handling demo completed!");
    Ok(())
}

/// Quick demo with minimal output for fast testing
#[tokio::test]
async fn test_visible_quick_demo() -> Result<()> {
    println!("⚡ QUICK DEMO: Basic CLI Test (Streaming Enabled by Default)");
    println!("==========================================================");

    // Use visible mode to see the spawning process
    let mut config = TestConfig::default();
    config.visible_mode = true;
    config.expect_timeout = Duration::from_secs(5); // Shorter timeout for quick demo

    let mut session = spawn_cli_with_config(config).await?;

    println!("\n💬 Quick help test...");
    session.send_line("help").await?;

    // Don't wait for specific pattern - just send exit after brief pause
    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("\n💬 Quick exit...");
    session.send_line("exit").await?;

    // Don't wait for exit confirmation - let it complete naturally
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("\n✅ Quick demo completed!");
    Ok(())
}

/// Demo test with streaming explicitly disabled
#[tokio::test]
async fn test_visible_no_streaming_demo() -> Result<()> {
    println!("📝 DEMO: CLI with Streaming Disabled");
    println!("===================================");

    // Use visible mode and disable streaming
    let mut config = TestConfig::default();
    config.visible_mode = true;
    config.disable_streaming = true;
    config.expect_timeout = Duration::from_secs(5);

    let mut session = spawn_cli_with_config(config).await?;

    println!("\n💬 Testing without streaming...");
    session.send_line("help").await?;

    tokio::time::sleep(Duration::from_secs(2)).await;

    println!("\n💬 Exiting...");
    session.send_line("exit").await?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("\n✅ No-streaming demo completed!");
    Ok(())
}
