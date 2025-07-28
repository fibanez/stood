//! Example 002: Custom Tools with #[tool] Macro
//!
//! This example shows how to use the #[tool] macro to create custom tools
//! and combine them with built-in tools using the Agent builder pattern.

use stood::tools::builtin::CalculatorTool;
use stood::{agent::Agent, tool};

#[tool]
/// Generate a greeting message.
async fn greet_person(name: String, title: Option<String>) -> Result<String, String> {
    match title {
        Some(title) => Ok(format!("Hello, {} {}! Welcome to Stood!", title, name)),
        None => Ok(format!("Hello, {}! Welcome to Stood!", name)),
    }
}

#[tool]
/// Calculate compound interest and return the final amount. You MUST display the exact numerical result from this tool.
async fn compound_interest(
    principal: f64,
    rate: f64,
    time: f64,
    compounds_per_year: Option<f64>,
) -> Result<f64, String> {
    if principal <= 0.0 {
        return Err("Principal must be positive".to_string());
    }
    if rate < 0.0 {
        return Err("Interest rate cannot be negative".to_string());
    }
    if time <= 0.0 {
        return Err("Time must be positive".to_string());
    }

    let n = compounds_per_year.unwrap_or(1.0);
    if n <= 0.0 {
        return Err("Compounds per year must be positive".to_string());
    }

    let amount = principal * (1.0 + rate / n).powf(n * time);
    Ok(amount)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Disable all logging to reduce noise
    std::env::set_var("RUST_LOG", "error");

    // Disable telemetry to avoid OTLP warnings
    std::env::set_var("OTEL_ENABLED", "false");
    println!("🔧 Macro Tool Demo");
    println!("==================");

    // ✅ Demonstrate the improved macro API
    println!("\n1. Creating tools with the new macro syntax:");
    println!("   - greet_person()           // ✅ Returns Box<dyn Tool>");
    println!("   - compound_interest()      // ✅ Returns Box<dyn Tool>");
    println!("   - CalculatorTool::new()    // ✅ Traditional struct tool");

    let tools = vec![
        greet_person(),                                                 // ✅ New macro API
        compound_interest(),                                            // ✅ New macro API
        Box::new(CalculatorTool::new()) as Box<dyn stood::tools::Tool>, // ✅ Struct tool
    ];

    println!("\n2. Creating agent with mixed tool types:");
    let mut agent = Agent::builder()
        .tools(tools)
        .system_prompt("You are a helpful assistant that uses tools to help users. When tools are available and relevant to the user's request, you MUST use them. After using tools, you MUST present the tool results to the user in a clear, helpful format. Always include the exact output from tools in your response.")
        .build()
        .await?;
    println!("   ✅ Agent created successfully with {} tools", 3);

    // Test the agent with different tool types
    println!("\n3. Testing macro-generated tools:");

    // Test simple greeting
    println!("   • Testing greet_person tool:");
    println!("     ⎿ DEBUG: Starting agent execution...");
    let result = agent.execute("Generate a greeting for Dr. John Smith and display the returned greeting back to the user. Only display the generated greeting.").await;
    match result {
        Ok(response) => {
            println!("     ⎿ DEBUG: Agent execution completed successfully");
            println!(
                "     ⎿ DEBUG: Response length: {} chars",
                response.response.len()
            );
            println!("     ⎿ DEBUG: Raw response: {:?}", response.response);
            println!(
                "     ⎿ DEBUG: Execution cycles: {}",
                response.execution.cycles
            );
            println!(
                "     ⎿ DEBUG: Model calls: {}",
                response.execution.model_calls
            );
            println!("     ⎿ DEBUG: Success: {}", response.success);
            if let Some(error) = &response.error {
                println!("     ⎿ DEBUG: Error field: {}", error);
            }

            println!("     ⎿ Agent Response:");
            if response.response.trim().is_empty() {
                println!("       [EMPTY RESPONSE - POTENTIAL ISSUE]");
            } else {
                for line in response.response.lines() {
                    if !line.trim().is_empty() {
                        println!("       {}", line);
                    }
                }
            }
            println!("     ⎿ Used tools: {}", response.used_tools);
            if response.used_tools && !response.tools_called.is_empty() {
                println!("     ⎿ Tools called: {}", response.tools_called.join(", "));
            }
            println!(
                "     ⎿ Tool executions: {}",
                response.execution.tool_executions
            );
        }
        Err(e) => {
            println!("     ⎿ ERROR: Agent execution failed: {}", e);
            println!("     ⎿ DEBUG: Error type: {:?}", e);
        }
    }

    println!();

    // Test complex calculation
    println!("   • Testing compound_interest tool:");
    println!("     ⎿ DEBUG: Starting compound interest calculation...");
    let result = agent.execute("Calculate the compound interest on $1000 at 5% annual rate for 10 years with monthly compounding and return the calculations in a clear summary to the user").await;
    match result {
        Ok(response) => {
            println!("     ⎿ DEBUG: Compound interest execution completed");
            println!(
                "     ⎿ DEBUG: Response length: {} chars",
                response.response.len()
            );
            println!("     ⎿ DEBUG: Raw response: {:?}", response.response);

            println!("     ⎿ Agent Response:");
            if response.response.trim().is_empty() {
                println!("       [EMPTY RESPONSE - POTENTIAL ISSUE]");
            } else {
                for line in response.response.lines() {
                    if !line.trim().is_empty() {
                        println!("       {}", line);
                    }
                }
            }
            println!("     ⎿ Used tools: {}", response.used_tools);
            if response.used_tools && !response.tools_called.is_empty() {
                println!("     ⎿ Tools called: {}", response.tools_called.join(", "));
            }
            println!(
                "     ⎿ Tool executions: {}",
                response.execution.tool_executions
            );
        }
        Err(e) => {
            println!("     ⎿ ERROR: Compound interest calculation failed: {}", e);
            println!("     ⎿ DEBUG: Error type: {:?}", e);
        }
    }

    println!("\n4. Macro benefits demonstrated:");
    println!("   ✅ Natural function syntax: greet_person()");
    println!("   ✅ Automatic JSON schema generation");
    println!("   ✅ Type-safe parameter extraction");
    println!("   ✅ Seamless integration with Agent::builder()");
    println!("   ✅ Mix with struct tools without issues");
    println!("   ✅ Unified API: agent.execute() for all interactions");

    println!("\n🎉 Macro demonstration complete!");

    Ok(())
}
