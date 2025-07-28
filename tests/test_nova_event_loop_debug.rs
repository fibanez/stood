use std::env;
use stood::{
    agent::{Agent, LogLevel},
    llm::models::Bedrock,
    tool,
};

#[tool]
async fn simple_add(a: f64, b: f64) -> Result<f64, String> {
    println!("🔧 Tool called: simple_add({}, {})", a, b);
    Ok(a + b)
}

#[tokio::test]
async fn test_nova_event_loop_minimal() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Nova Event Loop Debug Test");
    println!("===========================");

    // Disable telemetry completely - use correct env var
    env::set_var("OTEL_ENABLED", "false");
    env::set_var("RUST_LOG", "stood=trace");
    tracing_subscriber::fmt()
        .with_env_filter("stood=trace")
        .with_target(true)
        .try_init()
        .ok();

    // Configure providers
    use stood::llm::registry::{PROVIDER_REGISTRY, ProviderRegistry};
    ProviderRegistry::configure().await?;

    // Check Bedrock availability
    if !PROVIDER_REGISTRY.is_configured(stood::llm::traits::ProviderType::Bedrock).await {
        eprintln!("❌ AWS Bedrock not available - skipping test");
        return Ok(());
    }

    println!("✅ Bedrock provider configured");

    // Create simple tool
    let tools = vec![simple_add()];

    // Create agent without event loop initialization
    println!("\n🔨 Creating agent with Nova...");
    let mut agent = Agent::builder()
        .model(Bedrock::NovaLite)
        .system_prompt("You are a helpful assistant. Use the simple_add tool when asked to add numbers.")
        .with_streaming(false)
        .tools(tools)
        .with_log_level(LogLevel::Trace)
        .build()
        .await?;

    println!("🤖 Agent created successfully");

    // Set a shorter timeout for the event loop
    use tokio::time::{timeout, Duration};
    
    println!("\n📞 Calling agent.execute() with timeout...");
    let result = timeout(
        Duration::from_secs(10),
        agent.execute("Please add 5 and 3 using the simple_add tool.")
    ).await;

    match result {
        Ok(Ok(response)) => {
            println!("\n✅ Response received:");
            println!("Content: {}", response.response);
            println!("Used tools: {}", response.used_tools);
            println!("Tools called: {:?}", response.tools_called);
        }
        Ok(Err(e)) => {
            println!("\n❌ Execute failed: {}", e);
            return Err(e.into());
        }
        Err(_) => {
            println!("\n⏰ Timeout after 10 seconds!");
            println!("Agent appears to be hanging in the event loop");
            return Err("Timeout in event loop".into());
        }
    }

    Ok(())
}