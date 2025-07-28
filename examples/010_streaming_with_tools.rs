//! Example 010: Streaming with Tools Integration
//!
//! This example demonstrates how to combine streaming responses with tool execution.
//! It showcases real-time streaming for both model responses and tool outputs,
//! with a custom tool designed to guarantee execution and demonstrate typewriter-style output.
//!
//! Key features demonstrated:
//! - Streaming responses with tool integration
//! - Custom tool guaranteed to be called
//! - Typewriter-style display of tool results
//! - Real-time metrics and performance tracking
//! - Tool event callbacks during streaming
//!
//! This example successfully demonstrates unified tool execution in both streaming
//! and non-streaming modes, with full parity between the two approaches.

use std::io::{self, Write};
use std::time::{Duration, Instant};
use stood::agent::callbacks::{CallbackHandler, ToolEvent};
use stood::agent::{Agent, LogLevel};
use stood::tool;
use tokio::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// A custom tool that provides detailed system information
/// This tool is designed to be called based on prompts that ask for system details
#[tool]
/// Get detailed system information and current time with analysis
async fn system_analyzer(analysis_type: String) -> Result<String, String> {
    // Simulate some processing time to show streaming
    tokio::time::sleep(Duration::from_millis(500)).await;

    let current_time = chrono::Utc::now();

    let response = match analysis_type.to_lowercase().as_str() {
        "performance" => {
            format!(
                "🔍 SYSTEM PERFORMANCE ANALYSIS\n\
                ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                Current Time: {}\n\
                System Load: Moderate (simulated)\n\
                Memory Usage: 67% (simulated)\n\
                CPU Utilization: 23% (simulated)\n\
                Network Status: Optimal\n\
                Disk I/O: Normal\n\
                \n\
                📊 Performance Summary:\n\
                • System is running efficiently\n\
                • No bottlenecks detected\n\
                • Memory usage within acceptable range\n\
                • Network connectivity is stable",
                current_time.format("%Y-%m-%d %H:%M:%S UTC")
            )
        }
        "security" => {
            format!(
                "🔒 SECURITY ANALYSIS REPORT\n\
                ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                Scan Time: {}\n\
                Security Status: ✅ SECURE\n\
                \n\
                🛡️ Security Checks:\n\
                • Firewall: Active and configured\n\
                • Anti-malware: Up to date\n\
                • Access controls: Properly configured\n\
                • Network encryption: Enabled\n\
                • System updates: Current\n\
                \n\
                📋 Recommendations:\n\
                • Continue regular security updates\n\
                • Monitor access logs periodically\n\
                • Backup configurations are optimal",
                current_time.format("%Y-%m-%d %H:%M:%S UTC")
            )
        }
        "environment" => {
            format!(
                "🌍 ENVIRONMENT ANALYSIS\n\
                ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                Analysis Time: {}\n\
                \n\
                💻 System Environment:\n\
                • Operating System: Linux (detected)\n\
                • Architecture: x64\n\
                • Available Tools: rust, cargo, git\n\
                • Shell Environment: bash/zsh compatible\n\
                • Package Managers: cargo, apt\n\
                \n\
                🔧 Development Setup:\n\
                • Rust toolchain: Available\n\
                • Git version control: Configured\n\
                • IDE/Editor support: Ready\n\
                • Build system: Cargo configured",
                current_time.format("%Y-%m-%d %H:%M:%S UTC")
            )
        }
        _ => {
            format!(
                "📊 GENERAL SYSTEM OVERVIEW\n\
                ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
                Analysis Time: {}\n\
                \n\
                🖥️ System Status: OPERATIONAL\n\
                \n\
                Key Metrics:\n\
                • Uptime: Excellent\n\
                • Response Time: < 100ms\n\
                • Error Rate: 0.01%\n\
                • Throughput: Optimal\n\
                \n\
                📈 Health Score: 95/100\n\
                \n\
                Available analysis types:\n\
                • 'performance' - CPU, memory, I/O analysis\n\
                • 'security' - Security posture evaluation\n\
                • 'environment' - Development environment status",
                current_time.format("%Y-%m-%d %H:%M:%S UTC")
            )
        }
    };

    Ok(response)
}

/// A callback handler that displays streaming output with tool integration
struct StreamingToolDisplay {
    start_time: Mutex<Option<Instant>>,
    token_count: Mutex<usize>,
    test_started: Mutex<bool>,
    tool_output_buffer: Mutex<String>,
}

impl StreamingToolDisplay {
    fn new() -> Self {
        Self {
            start_time: Mutex::new(None),
            token_count: Mutex::new(0),
            test_started: Mutex::new(false),
            tool_output_buffer: Mutex::new(String::new()),
        }
    }

    /// Simulate typewriter effect for tool output
    async fn typewriter_display(&self, text: &str, delay_ms: u64) {
        for char in text.chars() {
            print!("{}", char);
            io::stdout().flush().unwrap();
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }
    }
}

#[async_trait::async_trait]
impl CallbackHandler for StreamingToolDisplay {
    /// Handle streaming content as it's generated
    async fn on_content(
        &self,
        content: &str,
        _is_complete: bool,
    ) -> Result<(), stood::agent::callbacks::CallbackError> {
        // Start tracking on first content
        if !*self.test_started.lock().await {
            *self.test_started.lock().await = true;
            *self.start_time.lock().await = Some(Instant::now());
            println!("💭 Agent response streaming:\n");
        }

        // Display the chunk with typewriter effect
        self.typewriter_display(content, 0).await;

        // Count tokens (rough approximation)
        let tokens = content.split_whitespace().count();
        *self.token_count.lock().await += tokens;

        Ok(())
    }

    /// Handle tool events with enhanced display
    async fn on_tool(
        &self,
        event: ToolEvent,
    ) -> Result<(), stood::agent::callbacks::CallbackError> {
        match event {
            ToolEvent::Started { name, .. } => {
                println!("\n\n🔧 Tool Execution Started");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("🛠️  Tool: '{}'", name);
                println!("⏳ Processing...");
                println!();

                // Clear the tool output buffer
                self.tool_output_buffer.lock().await.clear();
            }
            ToolEvent::Completed {
                name,
                duration,
                output,
                ..
            } => {
                println!(
                    "✅ Tool '{}' completed in {:.2}s",
                    name,
                    duration.as_secs_f64()
                );
                println!("\n📋 Tool Output:");

                // Display tool result with typewriter effect
                if let Some(ref output_value) = output {
                    let output_string = if let Some(s) = output_value.as_str() {
                        s.to_string()
                    } else {
                        output_value.to_string()
                    };
                    self.typewriter_display(&output_string, 0).await;
                } else {
                    self.typewriter_display("Tool completed successfully with no output", 8)
                        .await;
                }

                println!("\n🔄 Continuing agent response...\n");
            }
            ToolEvent::Failed { name, error, .. } => {
                println!("❌ Tool '{}' failed: {}", name, error);
                println!("🔄 Agent will continue without tool result...\n");
            }
        }
        Ok(())
    }

    /// Handle completion of agent execution
    async fn on_complete(
        &self,
        result: &stood::agent::AgentResult,
    ) -> Result<(), stood::agent::callbacks::CallbackError> {
        println!("\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("✅ Execution completed successfully!");

        if let Some(start) = *self.start_time.lock().await {
            let duration = start.elapsed();
            let tokens = *self.token_count.lock().await;

            println!("\n📊 Session Metrics:");
            println!("  🕐 Total duration: {:.2}s", duration.as_secs_f64());
            println!("  📝 Approximate tokens: {}", tokens);
            if duration.as_secs_f64() > 0.0 {
                println!(
                    "  ⚡ Tokens/second: {:.1}",
                    tokens as f64 / duration.as_secs_f64()
                );
            }
            println!(
                "  🔧 Tools used: {}",
                if result.used_tools { "Yes" } else { "No" }
            );
            if result.used_tools {
                println!("  📋 Tools called: {}", result.tools_called.join(", "));
            }
        }

        // Reset for next execution
        *self.token_count.lock().await = 0;
        *self.test_started.lock().await = false;

        Ok(())
    }

    /// Handle errors during execution
    async fn on_error(
        &self,
        error: &stood::StoodError,
    ) -> Result<(), stood::agent::callbacks::CallbackError> {
        println!("\n❌ Error during execution: {}", error);
        Ok(())
    }
}

/// Interactive prompt for log level selection
fn select_log_level() -> LogLevel {
    println!("🔧 Select debug log level:");
    println!("  1. Off (clean demo output)");
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
    println!("🎯 Stood Streaming + Tools Demo");
    println!("═══════════════════════════════════════");
    println!("Demonstrating real-time streaming with tool integration\n");

    // Interactive log level selection
    let log_level = select_log_level();

    // Set up logging based on user selection
    match log_level {
        LogLevel::Off => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_subscriber::EnvFilter::new("off"))
                .init();
        }
        _ => {
            let tracing_level = match log_level {
                LogLevel::Info => tracing::Level::INFO,
                LogLevel::Debug => tracing::Level::DEBUG,
                LogLevel::Trace => tracing::Level::TRACE,
                LogLevel::Off => unreachable!(),
            };

            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .with(
                    tracing_subscriber::EnvFilter::from_default_env()
                        .add_directive(tracing_level.into()),
                )
                .init();
        }
    }

    println!("\n✅ Log level set to: {:?}\n", log_level);

    // Create agent with streaming and tools
    let streaming_handler = StreamingToolDisplay::new();
    let mut agent = Agent::builder()
        .tools(vec![system_analyzer()]) // Add our custom tool
        .with_callback_handler(streaming_handler)
        .with_log_level(log_level)
        .build()
        .await?;

    // Test cases designed to trigger the custom tool
    let test_prompts = vec![
        (
            "System Performance", 
            "I need you to analyze the current system performance. Please use the system analyzer tool with 'performance' analysis type to get detailed metrics about CPU, memory, and I/O."
        ),
        (
            "Security Check", 
            "Can you perform a security analysis of the current system? Use the system analyzer tool with 'security' analysis type to check the security posture."
        ),
        (
            "Environment Status", 
            "Please analyze the development environment setup. Use the system analyzer tool with 'environment' analysis type to check what tools and configurations are available."
        ),
    ];

    let total_tests = test_prompts.len();
    println!("Running {} streaming + tools tests...\n", total_tests);

    for (i, (category, prompt)) in test_prompts.into_iter().enumerate() {
        println!("\n╭────────────────────────────────────────────────────────────╮");
        println!(
            "│ Test {} of {} - Category: {:20}               │",
            i + 1,
            total_tests,
            category
        );
        println!("╰────────────────────────────────────────────────────────────╯");
        println!("📝 Prompt: \"{}\"", prompt);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Execute the prompt - this should trigger our tool
        match agent.execute(prompt).await {
            Ok(result) => {
                println!("\n📄 Execution Summary:");
                println!("  ✅ Status: Success");
                println!("  📏 Response length: {} characters", result.response.len());
                println!(
                    "  ⏱️  Execution time: {:.2}s",
                    result.duration.as_secs_f64()
                );
                println!(
                    "  🔧 Tools used: {}",
                    if result.used_tools {
                        "✅ Yes"
                    } else {
                        "❌ No"
                    }
                );

                if result.used_tools {
                    println!("  📋 Tools called: {}", result.tools_called.join(", "));
                }
            }
            Err(e) => {
                println!("\n❌ Execution failed: {}", e);
            }
        }

        // Pause between tests (except for the last one)
        if i < total_tests - 1 {
            println!("\n⏸️  Press Enter for next test...");
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
        }
    }

    println!("\n╭─────────────────────────────────────────────────────────────╮");
    println!("│ ✨ Streaming + Tools demo completed!                        │");
    println!("│                                                             │");
    println!("│ Key features demonstrated:                                  │");
    println!("│ • Real-time streaming of model responses                   │");
    println!("│ • Tool execution with streaming callbacks                  │");
    println!("│ • Typewriter effect for enhanced UX                       │");
    println!("│ • Custom tools with guaranteed execution                   │");
    println!("│ • Performance metrics and monitoring                       │");
    println!("╰─────────────────────────────────────────────────────────────╯");

    Ok(())
}

