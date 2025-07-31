//! Example 024: Enterprise Prompt Builder Agent
//!
//! This example demonstrates an autonomous agent that helps build enterprise-level prompts.
//! The agent uses the built-in think tool for reasoning and a custom ask_user tool to
//! gather requirements interactively.
//!
//! Key features:
//! - Autonomous agent with enterprise prompt building system message
//! - Built-in think tool for step-by-step reasoning
//! - Custom ask_user tool for interactive requirements gathering
//! - Callbacks to display the agent's thinking process
//! - Print statements for major events (model calls, tool calls)
//! - Automatic execution with initial user message

use std::io::{self, Write};
use stood::agent::callbacks::{CallbackHandler, ToolEvent};
use stood::agent::Agent;
use stood::llm::models::Bedrock;
use stood::tool;
use tokio::sync::Mutex;
use rustyline::{Editor, Result as RustylineResult};

/// Smart input reader with bracketed paste mode support
fn read_smart_input(prompt: &str) -> RustylineResult<String> {
    // Enable bracketed paste mode for automatic paste detection
    let config = rustyline::Config::builder()
        .bracketed_paste(true)
        .build();
    
    let mut rl: Editor<(), rustyline::history::FileHistory> = Editor::with_config(config)?;
    
    let readline = rl.readline(prompt);
    match readline {
        Ok(line) => {
            // Count lines to detect paste operations
            let line_count = line.lines().count();
            if line_count > 1 {
                println!("📝 Pasted {} lines", line_count);
            }
            Ok(line)
        }
        Err(rustyline::error::ReadlineError::Interrupted) => {
            Err(rustyline::error::ReadlineError::Interrupted)
        }
        Err(rustyline::error::ReadlineError::Eof) => {
            Err(rustyline::error::ReadlineError::Eof)
        }
        Err(err) => Err(err),
    }
}

/// Custom tool for asking the user for input with paste detection
#[tool]
/// Ask the user a question and get their response with automatic paste detection
async fn ask_user(question: String) -> Result<String, String> {
    println!("\n🤔 Question:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{}", question);
    
    match read_smart_input("> ") {
        Ok(response) => {
            let trimmed = response.trim().to_string();
            Ok(trimmed)
        }
        Err(rustyline::error::ReadlineError::Interrupted) => {
            Err("User interrupted input (Ctrl+C)".to_string())
        }
        Err(rustyline::error::ReadlineError::Eof) => {
            Err("End of input reached (Ctrl+D)".to_string())
        }
        Err(e) => {
            Err(format!("Failed to read user input: {}", e))
        }
    }
}

/// Callback handler that displays the agent's thinking process
struct EnterprisePromptBuilderDisplay {
    thinking_active: Mutex<bool>,
    json_display_enabled: bool,
    first_model_call: Mutex<bool>,
}

impl EnterprisePromptBuilderDisplay {
    fn new(json_display_enabled: bool) -> Self {
        Self {
            thinking_active: Mutex::new(false),
            json_display_enabled,
            first_model_call: Mutex::new(true),
        }
    }

    /// Pretty print JSON with indentation and syntax highlighting
    fn pretty_print_json(&self, title: &str, json_value: &serde_json::Value) {
        if !self.json_display_enabled {
            return;
        }

        println!("\n📄 {} JSON:", title);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        match serde_json::to_string_pretty(json_value) {
            Ok(pretty_json) => {
                // Add some basic syntax highlighting with colors
                for line in pretty_json.lines() {
                    if line.trim_start().starts_with('"') && line.contains(':') {
                        // Key-value pairs in blue
                        println!("\x1b[34m{}\x1b[0m", line);
                    } else if line.trim().starts_with('"') {
                        // String values in green
                        println!("\x1b[32m{}\x1b[0m", line);
                    } else {
                        // Structure (brackets, braces) in default color
                        println!("{}", line);
                    }
                }
            }
            Err(_) => {
                // Fallback to simple display if pretty printing fails
                println!("{}", json_value);
            }
        }
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }

    /// Extract and display current message context (not full history)
    fn display_current_message_context(&self, messages: &stood::types::Messages, tools_available: usize) {
        if !self.json_display_enabled {
            return;
        }

        // Get the last few messages (current context)
        let all_messages = messages.iter().collect::<Vec<_>>();
        let context_messages = if all_messages.len() > 3 {
            // Show system + last 2 messages to avoid history spam
            let mut context = vec![];
            if let Some(first) = all_messages.first() {
                if first.role == stood::types::messages::MessageRole::System {
                    context.push(*first);
                }
            }
            context.extend(all_messages.iter().rev().take(2).rev());
            context
        } else {
            all_messages
        };

        // Create JSON representation of current context
        let json_messages: Vec<serde_json::Value> = context_messages
            .iter()
            .map(|msg| {
                serde_json::json!({
                    "role": format!("{:?}", msg.role).to_lowercase(),
                    "content": msg.content,
                    "content_length": msg.content.len()
                })
            })
            .collect();

        // Create a more informative request JSON with tool information
        let tool_info = if tools_available > 0 {
            serde_json::json!({
                "count": tools_available,
                "note": "Tool definitions not available in callback - contains built-in tools (calculator, file_read, file_write, etc.) and custom ask_user, think tools"
            })
        } else {
            serde_json::json!({
                "count": 0,
                "note": "No tools available"
            })
        };

        let request_json = serde_json::json!({
            "messages": json_messages,
            "tools": tool_info,
            "context": "current_message_context"
        });

        self.pretty_print_json("Model Request", &request_json);
    }
}

#[async_trait::async_trait]
impl CallbackHandler for EnterprisePromptBuilderDisplay {
    /// Handle streaming content from the model
    async fn on_content(
        &self,
        content: &str,
        _is_complete: bool,
    ) -> Result<(), stood::agent::callbacks::CallbackError> {
        // Hide content for first call, show for subsequent calls (unless thinking)
        let is_first_call = *self.first_model_call.lock().await;
        let is_thinking = *self.thinking_active.lock().await;
        
        if !is_first_call && !is_thinking {
            print!("{}", content);
            io::stdout().flush().unwrap();
        }
        Ok(())
    }

    /// Handle tool events
    async fn on_tool(
        &self,
        event: ToolEvent,
    ) -> Result<(), stood::agent::callbacks::CallbackError> {
        match event {
            ToolEvent::Started { name, .. } => {
                if name == "think" {
                    *self.thinking_active.lock().await = true;
                    println!("\n🧠 Agent is thinking...");
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                } else if name == "ask_user" {
                    println!("\n💬 Agent is preparing a question...");
                } else {
                    println!("\n🔧 Tool '{}' started", name);
                }
            }
            ToolEvent::Completed {
                name,
                duration,
                output,
                ..
            } => {
                if name == "think" {
                    *self.thinking_active.lock().await = false;
                    println!("✅ Thinking completed in {:.2}s", duration.as_secs_f64());

                    // Show the thinking content if available
                    if let Some(ref output_value) = output {
                        if let Some(guidance) = output_value.get("guidance") {
                            if let Some(guidance_str) = guidance.as_str() {
                                println!("💭 Agent's thoughts:");
                                println!("{}", guidance_str);
                            }
                        }
                    }
                    println!("\n🔄 Continuing agent response...");
                } else if name == "ask_user" {
                    println!("✅ Question answered in {:.2}s", duration.as_secs_f64());
                } else {
                    println!(
                        "✅ Tool '{}' completed in {:.2}s",
                        name,
                        duration.as_secs_f64()
                    );
                }
            }
            ToolEvent::Failed { name, error, .. } => {
                if name == "think" {
                    *self.thinking_active.lock().await = false;
                }
                println!("❌ Tool '{}' failed: {}", name, error);
            }
        }
        Ok(())
    }

    /// Handle execution completion
    async fn on_complete(
        &self,
        result: &stood::agent::AgentResult,
    ) -> Result<(), stood::agent::callbacks::CallbackError> {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🎯 Enterprise Prompt Builder Session Complete!");
        println!("📊 Execution Summary:");
        println!(
            "  ⏱️  Total duration: {:.2}s",
            result.duration.as_secs_f64()
        );
        println!(
            "  🔧 Tools used: {}",
            if result.used_tools { "Yes" } else { "No" }
        );
        if result.used_tools {
            println!("  📋 Tools called: {}", result.tools_called.join(", "));
        }
        println!("  📏 Response length: {} characters", result.response.len());
        Ok(())
    }

    /// Handle errors
    async fn on_error(
        &self,
        error: &stood::StoodError,
    ) -> Result<(), stood::agent::callbacks::CallbackError> {
        println!("\n❌ Error during execution: {}", error);
        Ok(())
    }

    /// Handle all events to capture JSON display for ModelStart and ModelComplete
    async fn handle_event(
        &self,
        event: stood::agent::callbacks::events::CallbackEvent,
    ) -> Result<(), stood::agent::callbacks::CallbackError> {
        use stood::agent::callbacks::events::CallbackEvent;
        use stood::agent::callbacks::ToolEvent;

        match event {
            CallbackEvent::ModelStart { messages, tools_available, .. } => {
                let mut is_first_call = self.first_model_call.lock().await;
                if *is_first_call {
                    *is_first_call = false;
                    // Still show JSON for first call if enabled, just with different message
                    if self.json_display_enabled {
                        println!("🤖 Initial model call starting...");
                        self.display_current_message_context(&messages, tools_available);
                    }
                } else if self.json_display_enabled {
                    println!("🤖 Model call starting...");
                    self.display_current_message_context(&messages, tools_available);
                }
                Ok(())
            }
            CallbackEvent::ModelComplete { response, duration, .. } => {
                // Always show JSON if enabled (including first call)
                if self.json_display_enabled {
                    let response_json = serde_json::json!({
                        "content": response,
                        "content_length": response.len(),
                        "duration_ms": duration.as_millis(),
                        "context": "model_response"
                    });
                    self.pretty_print_json("Model Response", &response_json);
                }
                Ok(())
            }
            // Handle other events using the simplified pattern
            CallbackEvent::ContentDelta { delta, complete, .. } => {
                self.on_content(&delta, complete).await
            }
            CallbackEvent::ToolStart { tool_name, input, .. } => {
                self.on_tool(ToolEvent::Started { name: tool_name, input }).await
            }
            CallbackEvent::ToolComplete { tool_name, output, error, duration, .. } => {
                if let Some(err) = error {
                    self.on_tool(ToolEvent::Failed { name: tool_name, error: err, duration }).await
                } else {
                    self.on_tool(ToolEvent::Completed { name: tool_name, output, duration }).await
                }
            }
            CallbackEvent::EventLoopComplete { result, .. } => {
                // Convert EventLoopResult to AgentResult for callback
                let agent_result = stood::agent::AgentResult::from(result, std::time::Duration::ZERO);
                self.on_complete(&agent_result).await
            }
            CallbackEvent::Error { error, .. } => {
                self.on_error(&error).await
            }
            _ => Ok(()), // Ignore other events
        }
    }
}

/// Interactive prompt for JSON display selection
fn select_json_display() -> bool {
    println!("🔧 Enable JSON display for model calls?");
    println!("  1. Yes - Show request/response JSON");
    println!("  2. No - Hide JSON details");
    print!("Enter your choice (1-2): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    match input.trim() {
        "1" => {
            println!("✅ JSON display enabled - you'll see model request/response JSON");
            true
        }
        "2" => {
            println!("✅ JSON display disabled - clean output mode");
            false
        }
        _ => {
            println!("Invalid choice, defaulting to disabled");
            false
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏢 Enterprise Prompt Builder Agent");
    println!("═══════════════════════════════════════");

    // Interactive JSON display selection
    let json_display_enabled = select_json_display();
    println!();

    // Check AWS credentials
    let has_aws = std::env::var("AWS_ACCESS_KEY_ID").is_ok()
        || std::env::var("AWS_PROFILE").is_ok()
        || std::env::var("AWS_ROLE_ARN").is_ok();

    if !has_aws {
        println!("⚠️ No AWS credentials found.");
        println!("To run this example, set up AWS credentials:");
        println!("   • Set AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY");
        println!("   • Or set AWS_PROFILE to use AWS credentials file");
        println!("   • Or configure IAM role with AWS_ROLE_ARN");
        return Ok(());
    }


    // Create the display handler with JSON display setting
    let display_handler = EnterprisePromptBuilderDisplay::new(json_display_enabled);

    // Create agent with enterprise prompt building capabilities
    println!("🔧 Creating agent with built-in tools and thinking capabilities...");
    let mut agent = Agent::builder()
        .model(Bedrock::Claude35Sonnet)
        .tools(vec![ask_user()]) // Add custom ask_user tool
        .with_builtin_tools() // This includes the think tool
        .with_callback_handler(display_handler)
        .with_task_evaluation("Review if we asked for sample data, did we create xml tags placeholders for the sample data, do we have all sections of a prompt, did we ask if the user wanted an evaluation prompt added")
        .build()
        .await?;


    // Send the initial user message to start the process
    let initial_message = 
            "Your task is to create an LLM prompt. Your purpose is to use your knowledge to identify questions to ask the user in order to build one prompt through systematic analysis and user collaboration.  We don't do it by using a role, we foucs strictly by defining the task we are trying to accomplish.

The structure and order of the prompt you are building contains the following information that you must gather:
1. 1 or 2 sentences that describe the task context and high level goal
2. The data section -> here you will include the xml markdown to insert the data - this may be dynamic content 
3. Detail task instruction, it may contain successs criteria, longer definition of what we described in section 1
4. Examples - we prefer to have multiple examples, relevant, and diferent from each other
5. Repetition of critical instructions
6. Description of tools that we will develop internall or exposed through MCP
7. Evaluation prompt - this will be a prompt that we will use for the model to verify we have all the information before we decide if we are done

            Your process:
            1. First, use the 'think' tool to analyze what information you need to build an effective enterprise prompt
            2. Use the 'ask_user' tool to gather specific requirements about each of the prompt sections
                You will first ask what the final outpu will look like so you have visibility on the ultimate goal
                These will be used optionally for examples, more importantly you will use them to think how to describe the task to the LLM
            3. Continue using 'think' and 'ask_user' iteratively to refine your understanding
            4. Ask the user what to do if we cannot complete a task or the information is not complete
            5. Ask the user about constraints and edge cases you think about so we can provide instructions in the prompt
            6. Think if there are other questions we may want to ask the user to make the prompt efficient
            4. Once you have sufficient information, build a comprehensive enterprise prompt using the structure we defined
            5. Present the final prompt with clear sections and explanations

            Be thorough, professional, and ensure the prompt you create will be suitable for enterprise use cases with proper structure, clarity, and business alignment. Ask one questions at a time, don't ask multiple pieces of information at once. 

Your goal is to identify the task, and generate a prompt - don't generate python code - focus on asking questions to the user using the ask user tool and only working on generating the best prompt based on you knowledge for the task. 

Start asking questions to identify the task we are trying to accomplish and gather all details from the prompt structure";

    // Note: Initial message is sent to agent but not displayed to user to avoid clutter

    // Execute the agent
    match agent.execute(initial_message).await {
        Ok(result) => {
            println!("\n🎉 Enterprise prompt building completed successfully!");
            println!("\n📋 Final Result:");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("{}", result.response);
        }
        Err(e) => {
            println!("\n❌ Agent execution failed: {}", e);
            println!("This could be due to:");
            println!("  • Network connectivity issues");
            println!("  • AWS service availability");
            println!("  • Authentication problems");
            println!("  • Rate limiting");
        }
    }

    println!("\n🏁 Enterprise Prompt Builder session ended.");
    println!("Thank you for using the Stood Enterprise Prompt Builder!");

    Ok(())
}
