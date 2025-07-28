use std::env;
use stood::{
    agent::{Agent, LogLevel},
    llm::models::Bedrock,
    tools::builtin::FileReadTool,
};
use std::fs;

#[tokio::test]
async fn test_nova_file_read_agent_execution_flow() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Nova File Read Agent Execution Flow Debug Test");
    println!("================================================");

    // Disable telemetry
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

    // Create test file
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("nova_agent_debug_test.txt");
    let test_content = "Test content for Nova agent debugging";
    fs::write(&temp_path, test_content)?;
    println!("📁 Created test file: {}", temp_path.display());

    // Create agent with Nova
    let mut agent = Agent::builder()
        .model(Bedrock::NovaMicro)
        .system_prompt("You are a helpful assistant. When asked to read a file, use the file_read tool.")
        .tool(Box::new(FileReadTool::new()))
        .with_log_level(LogLevel::Trace)
        .build()
        .await?;

    println!("\n🤖 Agent created with Nova Micro");

    // Test with explicit tool instruction
    println!("\n=== Testing Nova Tool Execution Flow ===");
    let prompt = format!("Use the file_read tool to read the file at path '{}'", temp_path.display());
    println!("Prompt: {}", prompt);
    
    let response = agent.execute(&prompt).await?;
    
    println!("\n📊 Execution Results:");
    println!("Success: {}", response.success);
    println!("Used tools: {}", response.used_tools);
    println!("Tools called: {:?}", response.tools_called);
    println!("Response: {}", response.response);
    
    if let Some(error) = &response.error {
        println!("Error: {}", error);
    }

    // Clean up
    fs::remove_file(&temp_path).ok();

    // Verify the test results
    if !response.success {
        println!("\n❌ Nova agent execution failed");
        return Err(format!("Execution failed: {}", response.error.unwrap_or_default()).into());
    }

    if !response.used_tools {
        println!("\n❌ Nova agent did not use tools");
        return Err("Nova agent should have used the file_read tool".into());
    }

    if !response.response.contains(test_content) {
        println!("\n❌ Response doesn't contain file content");
        println!("Expected content: '{}'", test_content);
        println!("Actual response: '{}'", response.response);
        return Err("Response should contain the file content".into());
    }

    println!("\n✅ Nova file read agent test completed successfully");
    Ok(())
}

#[tokio::test]
async fn test_claude_file_read_agent_control() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Claude File Read Agent Control Test");
    println!("=====================================");

    // Disable telemetry
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

    // Create test file
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("claude_agent_debug_test.txt");
    let test_content = "Test content for Claude agent debugging";
    fs::write(&temp_path, test_content)?;
    println!("📁 Created test file: {}", temp_path.display());

    // Create agent with Claude
    let mut agent = Agent::builder()
        .model(Bedrock::Claude35Haiku)
        .system_prompt("You are a helpful assistant. When asked to read a file, use the file_read tool.")
        .tool(Box::new(FileReadTool::new()))
        .with_log_level(LogLevel::Trace)
        .build()
        .await?;

    println!("\n🤖 Agent created with Claude 3.5 Haiku");

    // Test with same prompt as Nova
    println!("\n=== Testing Claude Tool Execution Flow ===");
    let prompt = format!("Use the file_read tool to read the file at path '{}'", temp_path.display());
    println!("Prompt: {}", prompt);
    
    let response = agent.execute(&prompt).await?;
    
    println!("\n📊 Execution Results:");
    println!("Success: {}", response.success);
    println!("Used tools: {}", response.used_tools);
    println!("Tools called: {:?}", response.tools_called);
    println!("Response: {}", response.response);

    // Clean up
    fs::remove_file(&temp_path).ok();

    // Verify
    if !response.success || !response.used_tools || !response.response.contains(test_content) {
        println!("\n❌ Claude control test failed");
        return Err("Claude control test failed".into());
    }

    println!("\n✅ Claude file read agent test completed successfully");
    Ok(())
}