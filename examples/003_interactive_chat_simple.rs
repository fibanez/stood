//! Example 003: Simple Interactive Chat
//!
//! This is a simplified interactive chat example that demonstrates basic
//! agent interaction without complex features.
//!
//! Features:
//! - ✅ Basic interactive REPL
//! - ✅ Simple agent responses
//! - ✅ Safe input handling
//! - ✅ Error handling
//! - ✅ Command support
//!
//! Usage:
//! ```bash
//! cargo run --example 003_interactive_chat_simple
//! ```

use std::io::{self, Write};
use stood::agent::Agent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup
    println!("🤖 Stood Simple Interactive Chat");
    println!("=================================");
    println!();

    // Check AWS credentials
    if std::env::var("AWS_PROFILE").is_err() && std::env::var("AWS_ACCESS_KEY_ID").is_err() {
        println!("⚠️  AWS credentials required for Bedrock integration");
        println!("Configure using: export AWS_PROFILE=your-profile");
        println!();
    }

    // Create agent without tools for simplicity
    println!("Setting up agent...");
    let mut agent = Agent::builder()
        .system_prompt(
            "You are a helpful assistant. Provide clear, concise answers to user questions.",
        )
        .build()
        .await?;

    println!("✅ Agent ready!");
    println!();
    println!("Commands: 'quit' to exit, 'help' for help");
    println!("Type your message and press Enter:");

    // Simple interactive loop with safety measures
    let mut message_count = 0;
    let max_messages = 10; // Safety limit to prevent infinite loops

    loop {
        // Safety check
        if message_count >= max_messages {
            println!(
                "\n⚠️  Reached maximum message limit ({}) - exiting for safety",
                max_messages
            );
            break;
        }

        // Show prompt
        print!("\nYou: ");
        io::stdout().flush()?;

        // Read input with error handling
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                println!("\nEOF detected - goodbye!");
                break;
            }
            Ok(_) => {
                let input = input.trim();

                // Skip empty input
                if input.is_empty() {
                    continue;
                }

                // Handle commands
                match input.to_lowercase().as_str() {
                    "quit" | "exit" => {
                        println!("👋 Goodbye!");
                        break;
                    }
                    "help" => {
                        println!("Available commands:");
                        println!("  quit/exit - Exit the chat");
                        println!("  help - Show this help");
                        println!("\nExample messages:");
                        println!("  - What's 15 * 23?");
                        println!("  - Greet me as Alice");
                        println!("  - Hello, how are you?");
                        continue;
                    }
                    _ => {}
                }

                // Process with agent
                message_count += 1;
                println!("\nAssistant:",);

                // Debug: Show what we're sending to the agent
                println!("[DEBUG] Sending to agent: '{}'", input);

                let start_time = std::time::Instant::now();
                match agent.execute(input).await {
                    Ok(result) => {
                        let duration = start_time.elapsed();

                        // Debug: Show response details
                        println!("[DEBUG] Response received in {:?}", duration);
                        println!("[DEBUG] Response length: {} chars", result.response.len());
                        println!("[DEBUG] Used tools: {}", result.used_tools);
                        if result.used_tools {
                            println!("[DEBUG] Tools called: {:?}", result.tools_called);
                        }
                        println!("[DEBUG] Success: {}", result.success);

                        if result.response.trim().is_empty() {
                            println!("❌ [ERROR] Empty response received!");
                            println!("💡 This might be the empty response bug we're debugging");
                        } else {
                            println!("{}", result.response);
                        }
                    }
                    Err(e) => {
                        let duration = start_time.elapsed();
                        println!("❌ Error after {:?}: {}", duration, e);
                        println!("[DEBUG] Full error: {:?}", e);

                        // Provide helpful hints for common errors
                        let error_str = e.to_string();
                        if error_str.contains("credentials") {
                            println!("💡 Check your AWS credentials");
                        } else if error_str.contains("context") || error_str.contains("too long") {
                            println!("💡 Message may be too long - try a shorter message");
                        } else if error_str.contains("ValidationException") {
                            println!("💡 This might be the ValidationException bug - check message formatting");
                        }
                    }
                }
            }
            Err(e) => {
                println!("\n❌ Input error: {}", e);
                break;
            }
        }
    }

    println!("\nThanks for trying Stood Interactive Chat!");
    Ok(())
}
