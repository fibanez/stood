//! Example 004: Simple Streaming Demo
//!
//! This example shows the simplest way to enable streaming with Stood agents
//! using the built-in PrintingConfig callbacks. Perfect for getting started
//! with streaming responses from AWS Bedrock.
//!
//! Note: Currently shows some debug output that will be cleaned up in future versions.

use std::env;
use std::io::{self, Write};
use stood::agent::callbacks::PrintingConfig;
use stood::agent::{Agent, LogLevel};

/// Interactive prompt for log level selection
fn select_log_level() -> LogLevel {
    println!("🔧 Select debug log level:");
    println!("  1. Off (no debug output)");
    println!("  2. Info (basic execution flow)");
    println!("  3. Debug (detailed step-by-step)");
    println!("  4. Trace (verbose with full details)");
    print!("Enter your choice (1-4): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    match input.trim() {
        "1" => LogLevel::Off,
        "2" => LogLevel::Info,
        "3" => LogLevel::Debug,
        "4" => LogLevel::Trace,
        _ => {
            println!("Invalid choice, defaulting to Off");
            LogLevel::Off
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 Real AWS Bedrock Streaming Demo");
    println!("====================================");
    println!();

    // Interactive log level selection
    let log_level = select_log_level();

    // Initialize logging based on user selection
    match log_level {
        LogLevel::Off => {
            // For "Off", suppress all logging including telemetry
            tracing_subscriber::fmt()
                .with_env_filter("off") // Turn off all logging
                .init();
        }
        _ => {
            let tracing_level = match log_level {
                LogLevel::Info => tracing::Level::INFO,
                LogLevel::Debug => tracing::Level::DEBUG,
                LogLevel::Trace => tracing::Level::TRACE,
                LogLevel::Off => unreachable!(), // Already handled above
            };

            tracing_subscriber::fmt()
                .with_max_level(tracing_level)
                .init();
        }
    }

    println!("\n✅ Log level set to: {:?}", log_level);

    // Check if AWS credentials are available
    let has_access_key = env::var("AWS_ACCESS_KEY_ID").is_ok();
    let has_profile = env::var("AWS_PROFILE").is_ok();
    let has_role_arn = env::var("AWS_ROLE_ARN").is_ok();

    if !has_access_key && !has_profile && !has_role_arn {
        println!("⚠️  AWS credentials not available - this demo requires valid AWS Bedrock access");
        println!("   Set AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY or AWS_PROFILE");
        return Ok(());
    }

    // Create agent with minimal callbacks and selected log level
    let mut agent = Agent::builder()
        .with_printing_callbacks_config(PrintingConfig::minimal())
        .with_log_level(log_level)
        .build()
        .await?;

    println!("🤖 Agent created with real streaming enabled by default");
    println!();

    // Test simple streaming response
    println!("📝 Testing simple streaming response...");
    let result = agent
        .execute("Tell me joke about programming that is 20 lines long")
        .await?;

    println!();
    println!("✅ Streaming completed!");
    println!("📊 You should have seen your response streamed above in real-time!");
    println!("   Duration: {:?}", result.duration);
    println!("   Used tools: {}", result.used_tools);
    println!("   Cycles: {}", result.execution.cycles);

    Ok(())
}

