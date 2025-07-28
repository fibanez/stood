//! Custom streaming callbacks demonstration
//!
//! This example demonstrates how to implement custom callback handlers for
//! streaming responses. It shows real-time token counting, performance metrics,
//! and how to build your own streaming visualization.
//!
//! Key features demonstrated:
//! - Custom CallbackHandler implementation
//! - Real-time token counting and metrics
//! - Performance tracking (tokens/second)
//! - Proper use of Agent::builder() and agent.execute() patterns

use stood::agent::{Agent, LogLevel};
use stood::agent::callbacks::{CallbackHandler, ToolEvent};
use tokio::sync::Mutex;
use std::time::Instant;
use std::io::{self, Write};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// A callback handler that displays streaming output in real-time
struct StreamingDisplay {
    start_time: Mutex<Option<Instant>>,
    token_count: Mutex<usize>,
    test_started: Mutex<bool>,
}

impl StreamingDisplay {
    fn new() -> Self {
        Self {
            start_time: Mutex::new(None),
            token_count: Mutex::new(0),
            test_started: Mutex::new(false),
        }
    }
}

#[async_trait::async_trait]
impl CallbackHandler for StreamingDisplay {
    /// Handle streaming content as it's generated
    async fn on_content(&self, content: &str, _is_complete: bool) -> Result<(), stood::agent::callbacks::CallbackError> {
        // Start tracking on first content
        if !*self.test_started.lock().await {
            *self.test_started.lock().await = true;
            *self.start_time.lock().await = Some(Instant::now());
            println!("💭 Response streaming in real-time:\n");
        }
        
        // Display the chunk without newline to show streaming effect
        print!("{}", content);
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
        
        // Count tokens (rough approximation)
        let tokens = content.split_whitespace().count();
        *self.token_count.lock().await += tokens;
        
        Ok(())
    }

    /// Handle tool events
    async fn on_tool(&self, event: ToolEvent) -> Result<(), stood::agent::callbacks::CallbackError> {
        match event {
            ToolEvent::Started { name, .. } => {
                println!("\n🔧 Using tool '{}'", name);
            }
            ToolEvent::Completed { name, duration, .. } => {
                println!("  ✓ Tool '{}' completed in {:.2}s", name, duration.as_secs_f64());
            }
            ToolEvent::Failed { name, error, .. } => {
                println!("  ❌ Tool '{}' failed: {}", name, error);
            }
        }
        Ok(())
    }

    /// Handle completion of agent execution
    async fn on_complete(&self, _result: &stood::agent::AgentResult) -> Result<(), stood::agent::callbacks::CallbackError> {
        println!("\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        if let Some(start) = *self.start_time.lock().await {
            let duration = start.elapsed();
            let tokens = *self.token_count.lock().await;
            
            println!("✅ Execution completed");
            println!("📊 Streaming metrics:");
            println!("  - Stream duration: {:.2}s", duration.as_secs_f64());
            println!("  - Approximate tokens: {}", tokens);
            if duration.as_secs_f64() > 0.0 {
                println!("  - Tokens/second: {:.1}", tokens as f64 / duration.as_secs_f64());
            }
        }
        
        // Reset for next execution
        *self.token_count.lock().await = 0;
        *self.test_started.lock().await = false;
        
        Ok(())
    }

    /// Handle errors during execution
    async fn on_error(&self, error: &stood::StoodError) -> Result<(), stood::agent::callbacks::CallbackError> {
        println!("\n❌ Error during streaming: {}", error);
        Ok(())
    }
}

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
    println!("🎯 Stood Streaming E2E Demo");
    println!("Demonstrating real-time streaming with Agent::builder() and agent.execute()\n");

    // Interactive log level selection
    let log_level = select_log_level();
    
    // Set up logging based on user selection
    match log_level {
        LogLevel::Off => {
            // For "Off", suppress all logging including telemetry
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_subscriber::EnvFilter::new("off"))  // Turn off all logging
                .init();
        }
        _ => {
            let tracing_level = match log_level {
                LogLevel::Info => tracing::Level::INFO,
                LogLevel::Debug => tracing::Level::DEBUG,
                LogLevel::Trace => tracing::Level::TRACE,
                LogLevel::Off => unreachable!(), // Already handled above
            };
            
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing_level.into()))
                .init();
        }
    }

    println!("\n✅ Log level set to: {:?}\n", log_level);

    // Create agent using the builder pattern with streaming configuration
    let streaming_handler = StreamingDisplay::new();
    let mut agent = Agent::builder()
        .with_callback_handler(streaming_handler)
        .with_log_level(log_level)  // Set the selected log level
        .build()
        .await?;

    // Test cases with different prompts
    let test_prompts = vec![
        ("Haiku", "Write a haiku about Rust programming"),
        ("Science", "Explain quantum computing in 3 sentences"),
        ("Facts", "List 5 interesting facts about the ocean"),
        ("Creative", "Tell me a very short story about a robot learning to paint"),
    ];

    println!("Running {} streaming tests with agent.execute()...\n", test_prompts.len());

    for (i, (category, prompt)) in test_prompts.iter().enumerate() {
        println!("\n╭─────────────────────────────────────────────────────────────╮");
        println!("│ Test {} of {} - Category: {:14}                      │", i + 1, test_prompts.len(), category);
        println!("╰─────────────────────────────────────────────────────────────╯");
        println!("📝 Prompt: \"{}\"", prompt);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        // Execute using the simple agent.execute() method
        // All streaming configuration was done in the builder
        match agent.execute(*prompt).await {
            Ok(result) => {
                // The streaming already happened via callbacks
                // The result contains the complete response and metrics
                println!("\n📄 Summary:");
                println!("  - Response length: {} characters", result.response.len());
                println!("  - Execution time: {:.2}s", result.duration.as_secs_f64());
                println!("  - Success: {}", if result.success { "✅" } else { "❌" });
                
                if result.used_tools {
                    println!("  - Tools used: {}", result.tools_called.join(", "));
                }
            }
            Err(e) => {
                println!("\n❌ Execution failed: {}", e);
            }
        }

        // Pause between tests (except for the last one)
        if i < test_prompts.len() - 1 {
            println!("\n⏸️  Press Enter for next test...");
            use std::io::{self, BufRead};
            let stdin = io::stdin();
            let mut lines = stdin.lock().lines();
            let _ = lines.next();
        }
    }

    println!("\n╭─────────────────────────────────────────────────────────────╮");
    println!("│ ✨ Streaming demo completed!                                │");
    println!("│                                                             │");
    println!("│ Key patterns demonstrated:                                  │");
    println!("│ • Agent::builder() for configuration                        │");
    println!("│ • agent.execute() for simple execution                      │");
    println!("│ • Callback handlers for streaming                           │");
    println!("│ • Real-time token display                                  │");
    println!("╰─────────────────────────────────────────────────────────────╯");
    
    Ok(())
}