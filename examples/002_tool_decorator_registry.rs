//! Example 002: Tool Decorator Registry
//!
//! This example demonstrates how to use the #[tool] decorator to create tools
//! and register them with the ToolRegistry for direct tool execution.

use stood::tools::ToolRegistry;
use stood_macros::tool;
use serde_json::json;

#[tool]
/// Add two numbers together
async fn add_numbers(a: f64, b: f64) -> Result<f64, String> {
    Ok(a + b)
}

#[tool]
/// Greet someone with an optional title
async fn greet(name: String, title: Option<String>) -> Result<String, String> {
    match title {
        Some(title) => Ok(format!("Hello, {} {}!", title, name)),
        None => Ok(format!("Hello, {}!", name)),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing [tool] decorator with unified tool system...");

    // Create a tool registry
    let registry = ToolRegistry::new();

    // Register tools created by the decorator
    registry.register_tool(add_numbers()).await?;
    registry.register_tool(greet()).await?;

    println!("✅ Registered {} tools", registry.tool_names().await.len());

    // Test the add_numbers tool
    println!("\n🧮 Testing add_numbers tool:");
    let result = registry.execute_tool("add_numbers", Some(json!({
        "a": 5.5,
        "b": 3.2
    })), None).await?;

    println!("Result: {}", serde_json::to_string_pretty(&result.content)?);
    assert!(result.success);

    // Test the greet tool without title
    println!("\n👋 Testing greet tool (no title):");
    let result = registry.execute_tool("greet", Some(json!({
        "name": "Alice"
    })), None).await?;

    println!("Result: {}", serde_json::to_string_pretty(&result.content)?);
    assert!(result.success);

    // Test the greet tool with title
    println!("\n👋 Testing greet tool (with title):");
    let result = registry.execute_tool("greet", Some(json!({
        "name": "Smith",
        "title": "Dr"
    })), None).await?;

    println!("Result: {}", serde_json::to_string_pretty(&result.content)?);
    assert!(result.success);

    // Test error case
    println!("\n❌ Testing error case (missing parameter):");
    let result = registry.execute_tool("add_numbers", Some(json!({
        "a": 5.0
        // missing "b" parameter
    })), None).await;

    match result {
        Ok(_) => panic!("Expected error for missing parameter"),
        Err(e) => println!("Expected error: {}", e),
    }

    // Test tool schemas
    println!("\n📋 Tool schemas:");
    let schemas = registry.get_tool_schemas().await;
    for schema in schemas {
        println!("Tool: {}", schema["name"]);
        println!("Description: {}", schema["description"]);
        println!("Schema: {}", serde_json::to_string_pretty(&schema["input_schema"])?);
        println!();
    }

    println!("✅ All tests passed! The [tool] decorator works with the unified system.");
    Ok(())
}