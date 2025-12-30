//! Example 005: Basic Callback System
//!
//! This example shows how to use printing callbacks to see real-time execution progress,
//! tool usage, and streaming content as it's generated.

use stood::agent::Agent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔊 Printing Callbacks Demo");
    println!("==========================\n");

    // Create agent with printing callbacks enabled
    println!("Creating agent with printing callbacks...");
    let mut agent = Agent::builder()
        .system_prompt("You are a helpful math assistant. Show your work when solving problems. When using the calculator tool, provide simple arithmetic expressions like '2+3', '10*5', '25*8+17', or '3.14159*25' - do not use symbols like π or mathematical functions.")
        .with_printing_callbacks()
        .with_builtin_tools()
        .build()
        .await?;

    println!("✅ Agent created with printing callbacks enabled\n");

    // Check if AWS credentials are available for live demo
    let has_aws = std::env::var("AWS_ACCESS_KEY_ID").is_ok()
        || std::env::var("AWS_PROFILE").is_ok()
        || std::env::var("AWS_ROLE_ARN").is_ok();

    if has_aws {
        println!("🔗 AWS credentials detected - running live demo with callbacks...\n");

        println!("📢 The output below shows real-time callback events:");
        println!("   • EventLoop start/completion");
        println!("   • Model invocations");
        println!("   • Tool executions");
        println!("   • Streaming content deltas");
        println!("   • Performance metrics");
        println!();

        println!("🧮 Executing: 'Calculate the area of a circle with radius 5 meters'");
        println!();

        match agent
            .execute("Calculate the area of a circle with radius 5 meters. Show your work.")
            .await
        {
            Ok(result) => {
                println!("\n✅ Execution completed successfully!");
                println!("📊 Execution Details:");
                println!("   • Cycles: {}", result.execution.cycles);
                println!("   • Tools used: {}", result.tools_called.len());
                println!("   • Duration: {:?}", result.duration);
                if !result.tools_called.is_empty() {
                    println!("   • Tools called: {}", result.tools_called.join(", "));
                }
            }
            Err(e) => {
                println!("\n⚠️ Execution failed: {}", e);
                println!("   This is normal in test environments without proper AWS setup");
            }
        }

        println!("\n🧮 Executing: 'What is 25 * 8 + 17?'");
        println!();

        match agent
            .execute("What is 25 * 8 + 17? Use the calculator tool.")
            .await
        {
            Ok(result) => {
                println!("\n✅ Second execution completed!");
                println!("📊 Performance: {:?}", result.duration);
            }
            Err(e) => {
                println!("\n⚠️ Second execution failed: {}", e);
            }
        }
    } else {
        println!("⚠️ No AWS credentials found.");
        println!("To see live callback output, set up AWS credentials:");
        println!("   • Set AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY");
        println!("   • Or set AWS_PROFILE to use AWS credentials file");
        println!("   • Or configure IAM role with AWS_ROLE_ARN");
        println!();
        println!("📋 With callbacks enabled, you would see:");
        println!("   🔄 EventLoopStart(uuid)");
        println!("   🔄 CycleStart(uuid, cycle_1)");
        println!("   🤖 ModelStart(ClaudeHaiku35, N tools)");
        println!("   📝 ContentDelta(chunk, complete: false)");
        println!("   📝 ContentDelta(chunk, complete: false)");
        println!("   📝 ContentDelta(chunk, complete: true)");
        println!("   🤖 ModelComplete(response_len, duration)");
        println!("   🔧 ToolStart(calculator)");
        println!("   🔧 ToolComplete(calculator, success: true)");
        println!("   ✅ EventLoopComplete(success: true, 1 cycles)");
    }

    println!("\n🎯 Callback Types Demonstrated:");
    println!("   • Event loop lifecycle events");
    println!("   • Model invocation start/completion");
    println!("   • Real-time content streaming deltas");
    println!("   • Tool execution monitoring");
    println!("   • Error handling and recovery");
    println!("   • Performance metrics collection");

    println!("\n📋 Other Callback Configurations Available:");
    println!("   • Agent::builder().with_verbose_callbacks() - Show reasoning");
    println!(
        "   • Agent::builder().with_performance_callbacks(tracing::Level::INFO) - Focus on metrics"
    );
    println!("   • Agent::builder().with_callback_handler(custom_handler) - Your own handler");
    println!("   • Agent::builder().with_composite_callbacks(vec![...]) - Multiple handlers");

    println!("\n✅ Printing Callbacks Demo completed!");

    Ok(())
}
