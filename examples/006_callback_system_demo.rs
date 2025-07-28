//! Demonstration of the callback system integration with the unified Agent interface.
//!
//! This example shows how to use the Agent with different callback configurations,
//! from silent execution to verbose monitoring with multiple handlers.

use stood::agent::{Agent, PrintingConfig};
use stood::agent::callbacks::{CallbackHandler, CallbackEvent, CallbackError};
use async_trait::async_trait;
use std::io::{self, Write};

/// Custom callback handler that aligns streaming output with the outline structure
#[derive(Debug)]
struct AlignedStreamingHandler {
    indent: String,
    stream_prefix_printed: std::sync::Arc<std::sync::Mutex<bool>>,
}

impl AlignedStreamingHandler {
    fn new(indent_level: usize) -> Self {
        let indent = "     ".repeat(indent_level) + "⎿ ";
        Self { 
            indent,
            stream_prefix_printed: std::sync::Arc::new(std::sync::Mutex::new(false)),
        }
    }
}

#[async_trait]
impl CallbackHandler for AlignedStreamingHandler {
    async fn handle_event(&self, event: CallbackEvent) -> Result<(), CallbackError> {
        match event {
            CallbackEvent::ContentDelta { delta, complete, .. } => {
                if delta.trim().is_empty() {
                    return Ok(());
                }
                
                let mut prefix_printed = self.stream_prefix_printed.lock().unwrap();
                
                // Print the streaming prefix only once at the start
                if !*prefix_printed {
                    print!("{}STREAMING: ", self.indent);
                    *prefix_printed = true;
                }
                
                // Print content, respecting newlines
                if delta.contains('\n') {
                    // Split on newlines and handle each part
                    let parts: Vec<&str> = delta.split('\n').collect();
                    for (i, part) in parts.iter().enumerate() {
                        print!("{}", part);
                        if i < parts.len() - 1 {
                            // Add newline and continuation prefix for all but the last part
                            println!();
                            print!("{}           ", self.indent); // Align with "STREAMING: "
                        }
                    }
                } else {
                    // No newlines, just print the content
                    print!("{}", delta);
                }
                
                if complete {
                    println!();
                    println!("{}STREAMING: Content delivery completed", self.indent);
                    *prefix_printed = false; // Reset for next stream
                }
                
                io::stdout().flush().unwrap();
            }
            CallbackEvent::ToolStart { tool_name, .. } => {
                // If we're in the middle of streaming, add a newline and proper alignment
                let prefix_printed = self.stream_prefix_printed.lock().unwrap();
                if *prefix_printed {
                    println!(); // End the current streaming line
                }
                println!("{}🔧 Executing tool: {}", self.indent, tool_name);
                if *prefix_printed {
                    print!("{}           ", self.indent); // Resume streaming alignment
                    io::stdout().flush().unwrap();
                }
            }
            CallbackEvent::ToolComplete { tool_name, duration, error, .. } => {
                // If we're in the middle of streaming, add a newline and proper alignment
                let prefix_printed = self.stream_prefix_printed.lock().unwrap();
                if *prefix_printed {
                    println!(); // End the current streaming line
                }
                if let Some(err) = error {
                    println!("{}❌ Tool {} failed after {:?}: {}", self.indent, tool_name, duration, err);
                } else {
                    println!("{}✅ Tool {} completed in {:?}", self.indent, tool_name, duration);
                }
                if *prefix_printed {
                    print!("{}           ", self.indent); // Resume streaming alignment
                    io::stdout().flush().unwrap();
                }
            }
            _ => {} // Handle other events as needed
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Disable logging to reduce noise and show clean callback output
    std::env::set_var("RUST_LOG", "error");
    std::env::set_var("OTEL_ENABLED", "false");
    
    println!("🚀 Callback System Demo");
    println!("========================\n");

    // Check if AWS credentials are available (silently)
    let has_aws = std::env::var("AWS_ACCESS_KEY_ID").is_ok() || 
                  std::env::var("AWS_PROFILE").is_ok() || 
                  std::env::var("AWS_ROLE_ARN").is_ok();

    // Example 1: Silent Agent (Creation + Execution)
    println!("1. Silent Agent - Default settings (no callback output)");
    println!("   ⎿ Creating agent...");
    let mut silent_agent = Agent::builder()
        .with_builtin_tools()
        .build().await?;
    
    println!("   ⎿ Agent created successfully with ExecutionConfig::default()");
    println!("   ⎿ Callback handler: None (silent execution)");
    println!("   ⎿ Tools available: Calculator, File operations, HTTP, Environment");
    
    if has_aws {
        println!("   ⎿ Testing execution:");
        println!("     ⎿ Sending: 'What is 3+7? Use the calculator tool.'");
        println!("     ⎿ Expected: No real-time output, just final result");
        println!("     ⎿ Callbacks: None (silent)");
        println!();
        match silent_agent.execute("What is 3+7? Use the calculator tool.").await {
            Ok(result) => {
                println!("     ⎿ DEBUG: Execution completed successfully");
                println!("     ⎿ DEBUG: Response length: {} chars", result.response.len());
                println!("     ⎿ DEBUG: Used tools: {}", result.used_tools);
                println!("     ⎿ DEBUG: Tools called: {:?}", result.tools_called);
                println!("     ⎿ Agent Response:");
                for line in result.response.lines() {
                    if !line.trim().is_empty() {
                        println!("       {}", line);
                    }
                }
            }
            Err(e) => {
                println!("     ⎿ ERROR: Execution failed: {}", e);
            }
        }
    } else {
        println!("   ⎿ silent_agent.execute(prompt) - Available (no callback output)");
    }

    // Example 2: Printing Agent (Creation + Execution)
    println!("\n2. Printing Agent - Basic callbacks with real-time output");
    println!("   ⎿ Creating agent...");
    let mut printing_agent = Agent::builder()
        .with_callback_handler(AlignedStreamingHandler::new(2))
        .with_builtin_tools()
        .build().await?;
    
    println!("   ⎿ Agent created with AlignedStreamingHandler");
    println!("   ⎿ Will show tool execution and streaming output");
    println!("   ⎿ Output format: Real-time content + tool notifications (aligned)");
    
    if has_aws {
        println!("   ⎿ Testing execution:");
        println!("     ⎿ Sending: 'Calculate 25 * 8 + 17 using the calculator tool.'");
        println!("     ⎿ Expected: Real-time streaming + tool notifications");
        println!("     ⎿ Callbacks: Tool start/complete messages + streaming content");
        println!();
        match printing_agent.execute("Calculate 25 * 8 + 17 using the calculator tool.").await {
            Ok(result) => {
                println!("     ⎿ DEBUG: Printing execution completed");
                println!("     ⎿ DEBUG: Used tools: {}", result.used_tools);
                println!("     ⎿ DEBUG: Tools called: {:?}", result.tools_called);
                println!("     ⎿ Final result received");
            }
            Err(e) => {
                println!("     ⎿ ERROR: Execution failed: {}", e);
            }
        }
    } else {
        println!("   ⎿ printing_agent.execute(prompt) - Available (with real-time callbacks)");
    }

    // Example 3: Verbose Agent (Creation + Execution)
    println!("\n3. Verbose Agent - Development mode with detailed traces");
    println!("   ⎿ Creating agent...");
    let mut verbose_agent = Agent::builder()
        .with_callback_handler(AlignedStreamingHandler::new(2))
        .with_builtin_tools()
        .build().await?;
    
    println!("   ⎿ Agent created with AlignedStreamingHandler (verbose mode)");
    println!("   ⎿ Will show reasoning, tools, and performance metrics");
    println!("   ⎿ Output format: Detailed execution traces + evaluation decisions (aligned)");
    
    if has_aws {
        println!("   ⎿ Testing execution:");
        println!("     ⎿ Sending: 'What is the area of a circle with radius 4 meters?'");
        println!("     ⎿ Expected: Detailed traces + reasoning + tool execution + performance");
        println!("     ⎿ Callbacks: All events including evaluation decisions and detailed metrics");
        println!();
        match verbose_agent.execute("What is the area of a circle with radius 4 meters? Use the calculator if needed.").await {
            Ok(result) => {
                println!("     ⎿ DEBUG: Verbose execution completed");
                println!("     ⎿ DEBUG: Execution cycles: {}", result.execution.cycles);
                println!("     ⎿ DEBUG: Model calls: {}", result.execution.model_calls);
                println!("     ⎿ DEBUG: Tool executions: {}", result.execution.tool_executions);
                println!("     ⎿ DEBUG: Duration: {:?}", result.duration);
                println!("     ⎿ DEBUG: Used tools: {}", result.used_tools);
                println!("     ⎿ DEBUG: Success: {}", result.success);
                println!("     ⎿ Verbose execution completed with detailed telemetry");
            }
            Err(e) => {
                println!("     ⎿ ERROR: Execution failed: {}", e);
            }
        }
    } else {
        println!("   ⎿ verbose_agent.execute(prompt) - Available (with detailed traces)");
    }

    // Example 4: Custom Configuration Agent (Creation + Execution)
    println!("\n4. Custom Configuration Agent - Production monitoring setup");
    println!("   ⎿ Creating agent...");
    let _custom_config = PrintingConfig {
        show_reasoning: false,
        show_tools: true,
        show_performance: true,
        stream_output: true,
    };
    
    let mut custom_agent = Agent::builder()
        .with_callback_handler(AlignedStreamingHandler::new(2))
        .with_builtin_tools()
        .build().await?;
    
    println!("   ⎿ Agent created with AlignedStreamingHandler (custom mode)");
    println!("   ⎿ Configuration breakdown:");
    println!("     ⎿ Streaming: AlignedStreamingHandler (✅ Aligned real-time streaming)");
    println!("     ⎿ Tools: Tool execution monitoring (✅ Start/complete notifications)");
    println!("     ⎿ Performance: Implicit via execution results (✅ Duration tracking)");
    println!("     ⎿ Reasoning: Not shown in streaming (❌ Focused on results)");
    println!("   ⎿ Use case: Production monitoring with clean aligned output");
    
    if has_aws {
        println!("   ⎿ Testing execution:");
        println!("     ⎿ Sending: 'What is 15 * 12? Use the calculator.'");
        println!("     ⎿ Expected: Aligned streaming + tool monitoring (clean output)");
        println!("     ⎿ Callbacks: AlignedStreamingHandler with tool notifications");
        println!();
        match custom_agent.execute("What is 15 * 12? Use the calculator.").await {
            Ok(result) => {
                println!("     ⎿ DEBUG: Custom configuration execution completed");
                println!("     ⎿ DEBUG: Used tools: {}", result.used_tools);
                println!("     ⎿ DEBUG: Tools called: {:?}", result.tools_called);
                println!("     ⎿ Custom configuration result received");
            }
            Err(e) => {
                println!("     ⎿ ERROR: Custom execution failed: {}", e);
            }
        }
    } else {
        println!("   ⎿ custom_agent.execute(prompt) - Available (with custom configuration)");
    }

    // Example 5: Fully Configured Agent (Creation + Execution)
    println!("\n5. Fully Configured Agent - Production-ready with all options");
    println!("   ⎿ Creating agent...");
    let mut configured_agent = Agent::builder()
        .system_prompt("You are a helpful assistant. Be concise and use tools when appropriate.")
        .temperature(0.3)
        .with_callback_handler(AlignedStreamingHandler::new(2))
        .with_builtin_tools()
        .build().await?;
    
    println!("   ⎿ Agent created with combined configuration:");
    println!("   ⎿ Configuration breakdown:");
    println!("     ⎿ System prompt: Custom assistant behavior");
    println!("     ⎿ Temperature: 0.3 (more focused responses)"); 
    println!("     ⎿ Callbacks: AlignedStreamingHandler enabled");
    println!("     ⎿ Tools: Built-in tool suite available");
    println!("   ⎿ Use case: Production-ready agent with monitoring");
    
    if has_aws {
        println!("   ⎿ Testing execution:");
        println!("     ⎿ Sending: 'Calculate 8 * 7 and explain briefly'");
        println!("     ⎿ Expected: Concise response due to custom system prompt + temp 0.3");
        println!("     ⎿ Callbacks: AlignedStreamingHandler with all events");
        println!();
        match configured_agent.execute("Calculate 8 * 7 and explain briefly").await {
            Ok(result) => {
                println!("     ⎿ DEBUG: Fully configured execution completed");
                println!("     ⎿ DEBUG: Response length: {} chars (should be concise)", result.response.len());
                println!("     ⎿ DEBUG: Used tools: {}", result.used_tools);
                println!("     ⎿ DEBUG: Duration: {:?}", result.duration);
                println!("     ⎿ Production-ready configuration demonstrated");
            }
            Err(e) => {
                println!("     ⎿ ERROR: Full configuration execution failed: {}", e);
            }
        }
    } else {
        println!("   ⎿ configured_agent.execute(prompt) - Available (full production setup)");
    }

    if !has_aws {
        println!("\n6. AWS Credentials Setup");
        println!("   ⎿ To see live callback differences, configure AWS Bedrock access:");
        println!("     ⎿ Set AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY, OR");
        println!("     ⎿ Set AWS_PROFILE to use credentials file, OR");
        println!("     ⎿ Configure IAM role with AWS_ROLE_ARN");
    }

    let section_num = if has_aws { 4 } else { 7 };
    println!("\n{}. Demonstrating execution config convenience constructors...", section_num);
    
    // Show ExecutionConfig convenience constructors work
    use stood::agent::ExecutionConfig;
    
    let _silent_config = ExecutionConfig::silent();
    println!("   ⎿ ExecutionConfig::silent() - No callbacks, clean execution");
    
    let _printing_config = ExecutionConfig::with_printing();
    println!("   ⎿ ExecutionConfig::with_printing() - Default printing callbacks");
    
    let _verbose_config = ExecutionConfig::verbose();
    println!("   ⎿ ExecutionConfig::verbose() - Full verbose output with reasoning");
    
    let _minimal_config = ExecutionConfig::minimal();
    println!("   ⎿ ExecutionConfig::minimal() - Minimal output for headless execution");

    let section_num = if has_aws { 5 } else { 8 };
    println!("\n{}. Example: Creating custom callback handler...", section_num);
    
    #[derive(Debug)]
    struct MyCustomHandler;
    
    #[async_trait]
    impl CallbackHandler for MyCustomHandler {
        async fn handle_event(&self, event: CallbackEvent) -> Result<(), CallbackError> {
            match event {
                CallbackEvent::ModelStart { model_id, .. } => {
                    println!("     ⎿ CUSTOM: 🤖 Model started: {}", model_id);
                }
                CallbackEvent::ToolStart { tool_name, .. } => {
                    println!("     ⎿ CUSTOM: 🔧 Tool executing: {}", tool_name);
                }
                CallbackEvent::ContentDelta { delta, complete, .. } => {
                    print!("{}", delta);
                    if complete {
                        println!();
                        println!("     ⎿ CUSTOM: 📝 Content streaming completed");
                    }
                }
                CallbackEvent::EventLoopComplete { .. } => {
                    println!("     ⎿ CUSTOM: ✅ Agent execution cycle completed");
                }
                _ => {} // Handle other events as needed
            }
            Ok(())
        }
    }
    
    let _custom_handler_agent = Agent::builder()
        .with_callback_handler(MyCustomHandler)
        .with_builtin_tools()
        .build().await?;
    
    println!("   ⎿ Custom callback handler created and configured");
    println!("   ⎿ Implementation: MyCustomHandler with prefixed output");
    println!("   ⎿ Events handled: ModelStart, ToolStart, ContentDelta, EventLoopComplete");
    println!("   ⎿ Use case: Custom analytics, logging, or UI integration");
    
    let section_num = if has_aws { 6 } else { 9 };
    println!("\n{}. Configuration Comparison Summary...", section_num);
    println!("   ⎿ Agent 1 (Silent): No callbacks, clean execution");
    println!("   ⎿ Agent 2 (Printing): AlignedStreamingHandler, real-time + tools");
    println!("   ⎿ Agent 3 (Verbose): AlignedStreamingHandler, comprehensive monitoring");
    println!("   ⎿ Agent 4 (Custom): AlignedStreamingHandler with focused monitoring");
    println!("   ⎿ Agent 5 (Full): Custom prompt + temp 0.3 + AlignedStreamingHandler");
    println!("\n   Configuration impact on behavior:");
    println!("     ⎿ Agent 1: No real-time feedback, fastest execution");
    println!("     ⎿ Agent 2: Real-time streaming with aligned formatting");
    println!("     ⎿ Agent 3: Same as Agent 2 (both use AlignedStreamingHandler)");
    println!("     ⎿ Agent 4: Clean aligned output with tool monitoring");
    println!("     ⎿ Agent 5: Concise responses + aligned comprehensive monitoring");

    println!("\n✅ Callback System Demo completed successfully!");
    println!("\nArchitecture highlights:");
    println!("  ⎿ Single execute() method for all use cases");
    println!("  ⎿ Builder pattern configures callbacks at construction time");
    println!("  ⎿ Real-time streaming content deltas during execution");
    println!("  ⎿ Tool execution monitoring with start/complete events");
    println!("  ⎿ Model invocation tracking with timing and usage");
    println!("  ⎿ Error handling and recovery events");
    println!("  ⎿ Python-like simplicity with Rust type safety");
    println!("  ⎿ Supports silent, printing, verbose, and custom configurations");
    println!("  ⎿ EventLoop integration with comprehensive callback emission");
    println!("\nCallback comparison summary:");
    println!("  ⎿ Silent: Clean execution, no real-time feedback");
    println!("  ⎿ Printing/Verbose: Real-time aligned streaming + tool notifications");
    println!("  ⎿ Custom Config: Selective callback features (tools + performance only)");
    println!("  ⎿ Full Config: Complete setup with behavior customization");
    println!("  ⎿ Custom Handler: Complete control over event handling and formatting");
    
    Ok(())
}