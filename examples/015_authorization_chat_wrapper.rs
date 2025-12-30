//! Authorization Chat Wrapper - Tool Authorization via Callbacks
//!
//! This example demonstrates how to implement tool authorization using the callback
//! system without modifying the core Stood library. The wrapper intercepts tool
//! execution and prompts the user for approval before allowing tools to run.
//!
//! Key features:
//! - ✅ User approval for tool execution
//! - 🔒 Works with both local and built-in tools
//! - 🎯 No core library modifications needed
//! - 💬 Interactive authorization prompts
//!
//! Usage:
//! ```bash
//! # Run the authorization chat wrapper
//! cargo run --example 010_authorization_chat_wrapper
//! ```

use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

use stood::agent::callbacks::error::CallbackError;
use stood::agent::callbacks::{CallbackEvent, CallbackHandler};
use stood::agent::Agent;
use stood::tool;
use stood::tools::builtin::{CalculatorTool, CurrentTimeTool};

/// Custom tool for demonstration
#[tool]
/// Read the contents of a file (requires authorization)
async fn secure_file_read(filename: String) -> Result<String, String> {
    match std::fs::read_to_string(&filename) {
        Ok(contents) => Ok(format!("File '{}' contents:\n{}", filename, contents)),
        Err(e) => Err(format!("Failed to read file '{}': {}", filename, e)),
    }
}

/// Custom tool for demonstration
#[tool]
/// Write content to a file (requires authorization)
async fn secure_file_write(filename: String, content: String) -> Result<String, String> {
    match std::fs::write(&filename, &content) {
        Ok(_) => Ok(format!(
            "Successfully wrote {} bytes to '{}'",
            content.len(),
            filename
        )),
        Err(e) => Err(format!("Failed to write to file '{}': {}", filename, e)),
    }
}

/// Authorization wrapper that intercepts tool calls
#[derive(Clone)]
struct AuthorizationWrapper {
    /// Track whether we're in interactive mode
    interactive: bool,
    /// Cache of user decisions for this session
    decision_cache: Arc<Mutex<std::collections::HashMap<String, bool>>>,
}

impl AuthorizationWrapper {
    fn new() -> Self {
        Self {
            interactive: true,
            decision_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Request user approval for tool execution
    async fn request_user_approval(
        &self,
        tool_name: &str,
        input: &serde_json::Value,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if !self.interactive {
            // In non-interactive mode, auto-approve for demo purposes
            return Ok(true);
        }

        // Check cache first
        let cache = self.decision_cache.lock().await;
        if let Some(&decision) = cache.get(tool_name) {
            println!(
                "\n🔄 Using cached decision for tool '{}': {}",
                tool_name,
                if decision { "APPROVED" } else { "DENIED" }
            );
            return Ok(decision);
        }
        drop(cache);

        // Display authorization prompt
        println!("\n{}", "=".repeat(60));
        println!("🔒 TOOL AUTHORIZATION REQUEST");
        println!("{}", "=".repeat(60));
        println!("Tool: {}", tool_name);
        println!(
            "Input: {}",
            serde_json::to_string_pretty(input).unwrap_or_else(|_| "Invalid JSON".to_string())
        );
        println!("{}", "-".repeat(60));
        print!("Allow this tool to execute? (y/n/a=always/d=deny always): ");
        io::stdout().flush().unwrap();

        // Read user input
        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();
        let response = response.trim().to_lowercase();

        let (approved, cache_decision) = match response.as_str() {
            "y" | "yes" => (true, false),
            "n" | "no" => (false, false),
            "a" | "always" => (true, true),
            "d" | "deny" => (false, true),
            _ => {
                println!("⚠️  Invalid response. Denying by default.");
                (false, false)
            }
        };

        // Cache decision if requested
        if cache_decision {
            let mut cache = self.decision_cache.lock().await;
            cache.insert(tool_name.to_string(), approved);
            println!("📝 Decision cached for future requests in this session.");
        }

        println!("{}", "=".repeat(60));
        println!();

        Ok(approved)
    }
}

#[async_trait::async_trait]
impl CallbackHandler for AuthorizationWrapper {
    async fn handle_event(&self, event: CallbackEvent) -> std::result::Result<(), CallbackError> {
        match event {
            CallbackEvent::ToolStart {
                tool_name, input, ..
            } => {
                println!("\n🔧 Tool execution requested: {}", tool_name);

                // Request user approval
                if !self
                    .request_user_approval(&tool_name, &input)
                    .await
                    .map_err(|e| {
                        CallbackError::ExecutionFailed(format!("Authorization error: {}", e))
                    })?
                {
                    // User denied - return error to prevent execution
                    return Err(CallbackError::ExecutionFailed(format!(
                        "User denied execution of tool '{}'",
                        tool_name
                    )));
                }

                println!("✅ Tool execution APPROVED: {}", tool_name);
                Ok(())
            }
            CallbackEvent::ToolComplete {
                tool_name,
                error,
                duration,
                ..
            } => {
                if let Some(error) = error {
                    println!(
                        "❌ Tool '{}' failed: {} (duration: {:?})",
                        tool_name, error, duration
                    );
                } else {
                    println!(
                        "✅ Tool '{}' completed successfully (duration: {:?})",
                        tool_name, duration
                    );
                }
                Ok(())
            }
            _ => Ok(()), // Ignore other events
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize minimal logging (errors only)
    tracing_subscriber::fmt().with_env_filter("error").init();

    println!("🔒 Authorization Chat Wrapper Demo");
    println!("This example demonstrates tool authorization via callbacks.");
    println!();

    // Create the authorization wrapper
    let auth_wrapper = Arc::new(AuthorizationWrapper::new());

    // Create agent with built-in tools and authorization callback
    let mut agent = Agent::builder()
        .tools(vec![
            Box::new(CalculatorTool::new()) as Box<dyn stood::tools::Tool>,
            Box::new(CurrentTimeTool::new()) as Box<dyn stood::tools::Tool>,
            secure_file_read(),
            secure_file_write(),
        ])
        .with_callback_handler(auth_wrapper.as_ref().clone())
        .system_prompt("You are a helpful assistant with access to tools. Always ask before using file operations or calculations.")
        .build()
        .await?;

    println!("✅ Agent initialized with authorization wrapper");
    println!();

    // Interactive chat loop
    println!("💬 Interactive Chat (type 'exit' to quit)");
    println!("Try asking me to:");
    println!("  - Calculate something (e.g., 'What is 25 * 17?')");
    println!("  - Check the current time");
    println!("  - Read a file (e.g., 'Read the Cargo.toml file')");
    println!("  - Write to a file (e.g., 'Write hello world to test.txt')");
    println!();
    println!("When tools are requested, you'll be prompted to authorize each one!");
    println!();

    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("exit") {
            println!("👋 Goodbye!");
            break;
        }

        // Execute with the agent - callbacks will handle authorization
        match agent.execute(input).await {
            Ok(response) => {
                println!("\n🤖 Assistant: {}", response.response);
            }
            Err(e) => {
                if e.to_string().contains("User denied") {
                    println!("\n⛔ Execution blocked: {}", e);
                    println!("The assistant cannot proceed without the requested tool.");
                } else {
                    println!("\n❌ Error: {}", e);
                }
            }
        }
        println!();
    }

    // Display session summary
    let cache = auth_wrapper.decision_cache.lock().await;
    if !cache.is_empty() {
        println!("\n📊 Session Authorization Summary:");
        println!("{}", "-".repeat(40));
        for (tool, approved) in cache.iter() {
            println!("  {} {}", if *approved { "✅" } else { "❌" }, tool);
        }
        println!("{}", "-".repeat(40));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authorization_wrapper() {
        // Test in non-interactive mode
        let wrapper_test = AuthorizationWrapper {
            interactive: false,
            decision_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
        };

        // Should auto-approve in non-interactive mode
        let result = wrapper_test
            .request_user_approval("test_tool", &serde_json::json!({"test": "value"}))
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_callback_handler() {
        // Test tool start event
        let event = CallbackEvent::ToolStart {
            tool_name: "calculator".to_string(),
            tool_use_id: "test_id".to_string(),
            input: serde_json::json!({"expression": "2+2"}),
        };

        // In non-interactive mode, this should succeed
        let wrapper_test = AuthorizationWrapper {
            interactive: false,
            decision_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
        };
        let result = wrapper_test.handle_event(event).await;
        assert!(result.is_ok());
    }
}
