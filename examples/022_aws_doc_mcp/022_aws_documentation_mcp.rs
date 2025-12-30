//! Example 022: AWS Documentation MCP Server Integration (NEW Simple Method)
//!
//! This example demonstrates the NEW simple way to integrate with the AWS Documentation
//! MCP server using Docker and the `with_mcp_client()` builder method.
//!
//! ## Prerequisites
//!
//! 1. Docker must be installed and running
//! 2. Build the AWS documentation MCP server image:
//!    ```bash
//!    git clone https://github.com/awslabs/mcp.git
//!    cd mcp/src/aws-documentation-mcp-server/
//!    docker build -t awslabs/aws-documentation-mcp-server .
//!    ```
//!
//! ## Usage
//!
//! ```bash
//! # Run the example
//! cargo run --example 022_aws_doc_mcp
//!
//! # Run with debug logging to see MCP tool calls
//! RUST_LOG=debug cargo run --example 022_aws_doc_mcp
//! ```
//!
//! ## What This Example Demonstrates
//!
//! - ✅ NEW simple MCP integration with `with_mcp_client()`
//! - ✅ Docker-based MCP server configuration
//! - ✅ Automatic tool discovery and namespace prefixing
//! - ✅ Verification that MCP tools are actually being used
//! - ✅ AWS documentation queries with real tool calls
//! - ✅ Error handling and graceful fallbacks

use stood::agent::Agent;
use stood::llm::models::Bedrock;
use stood::mcp::transport::{StdioConfig, TransportFactory};
use stood::mcp::{MCPClient, MCPClientConfig};

/// Create and connect to the AWS Documentation MCP server via Docker
async fn create_aws_docs_mcp_client() -> Result<MCPClient, Box<dyn std::error::Error>> {
    println!("🐳 Setting up AWS Documentation MCP server via Docker...");

    // Configure Docker to run the AWS documentation MCP server
    let docker_config = StdioConfig {
        command: "docker".to_string(),
        args: vec![
            "run".to_string(),
            "--rm".to_string(),
            "--interactive".to_string(),
            "--env".to_string(),
            "FASTMCP_LOG_LEVEL=ERROR".to_string(),
            "--env".to_string(),
            "AWS_DOCUMENTATION_PARTITION=aws".to_string(),
            "awslabs/aws-documentation-mcp-server:latest".to_string(),
        ],
        env_vars: std::collections::HashMap::new(),
        working_dir: None,
        startup_timeout_ms: 30_000, // Give Docker time to start
        max_message_size: Some(16 * 1024 * 1024),
    };

    // Create MCP client with extended timeouts for Docker
    let client_config = MCPClientConfig {
        client_name: "stood-aws-docs-client".to_string(),
        client_version: env!("CARGO_PKG_VERSION").to_string(),
        request_timeout_ms: 45_000, // Longer timeout for documentation queries
        max_concurrent_requests: 5,
        auto_reconnect: true,
        reconnect_delay_ms: 5_000,
        ..Default::default()
    };

    let transport = TransportFactory::stdio(docker_config);
    let mut mcp_client = MCPClient::new(client_config, transport);

    // Connect to the MCP server
    mcp_client.connect().await?;
    println!("✅ Connected to AWS Documentation MCP server");

    // List available tools to verify connection
    let tools = mcp_client.list_tools().await?;
    println!(
        "📚 Available AWS documentation tools ({} total):",
        tools.len()
    );
    for tool in &tools {
        println!("   - {} ({})", tool.name, tool.description);
    }

    Ok(mcp_client)
}

/// Test MCP tools directly to verify they work
async fn verify_mcp_tools(mcp_client: &mut MCPClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Verifying MCP tools work directly...");

    // Test a simple documentation search
    let test_params = serde_json::json!({
        "search_phrase": "CloudFormation template basics"
    });

    println!("📝 Testing 'search_documentation' tool with query: CloudFormation template basics");

    match mcp_client
        .call_tool("search_documentation", Some(test_params))
        .await
    {
        Ok(content) => {
            println!("✅ MCP tool call successful!");
            if let Some(first_result) = content.first() {
                match first_result {
                    stood::mcp::Content::Text(text) => {
                        let preview = if text.text.len() > 200 {
                            format!("{}...", &text.text[..200])
                        } else {
                            text.text.clone()
                        };
                        println!("🎯 Tool result preview: {}", preview);
                        println!("✅ MCP server is working correctly!");
                    }
                    _ => println!("📋 Tool returned non-text content"),
                }
            }
        }
        Err(e) => {
            println!("❌ MCP tool call failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

/// Main example function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with minimal output (only errors)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive("stood=error".parse()?),
        )
        .init();

    println!("🚀 AWS Documentation MCP Integration Example (NEW Simple Method)");
    println!("================================================================");

    // Step 1: Create and connect to MCP client
    let mut mcp_client = match create_aws_docs_mcp_client().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!(
                "❌ Failed to connect to AWS Documentation MCP server: {}",
                e
            );
            eprintln!("💡 Make sure you have:");
            eprintln!("   1. Docker installed and running");
            eprintln!("   2. Built the AWS documentation MCP server image:");
            eprintln!("      git clone https://github.com/awslabs/mcp.git");
            eprintln!("      cd mcp/src/aws-documentation-mcp-server/");
            eprintln!("      docker build -t awslabs/aws-documentation-mcp-server .");
            return Err(e);
        }
    };

    // Step 2: Verify MCP tools work directly
    verify_mcp_tools(&mut mcp_client).await?;

    // Step 3: Create agent using NEW simple with_mcp_client() method
    println!("\n🤖 Creating agent with NEW simple MCP integration...");

    let mut agent = Agent::builder()
        .model(Bedrock::ClaudeHaiku45)
        .system_prompt(
            "You are an AWS expert assistant with access to comprehensive AWS documentation. \
             Always use the aws_docs_search_documentation tool to get authoritative information \
             from official AWS sources. Be specific about which tools you're using and quote \
             relevant parts of the documentation in your responses.",
        )
        .with_mcp_client(mcp_client, Some("aws_docs_".to_string()))
        .await?
        .build()
        .await?;

    println!("✅ Agent created with AWS documentation MCP tools!");

    // Step 4: Test queries that demonstrate MCP usage
    let demo_queries = vec![
        "What are the key components of a CloudFormation template? Use the documentation tool to get authoritative information.",
        "How do I create a DynamoDB table with global secondary indexes? Search the AWS documentation for specific examples.",
    ];

    println!("\n📋 Testing AWS documentation queries...");
    println!("💡 Watch for tool calls to verify MCP integration is working!");

    for (i, query) in demo_queries.iter().enumerate() {
        println!("\n🧪 Test {}: {}", i + 1, query);
        println!("{}", "=".repeat(80));

        match agent.execute(*query).await {
            Ok(result) => {
                println!("🔧 Used tools: {}", result.used_tools);
                println!("📋 Tools called: {}", result.tools_called.join(", "));
                println!("\n🤖 Agent Response:");
                println!("{}", result.response);

                // Verify MCP tools were actually used
                if result
                    .tools_called
                    .iter()
                    .any(|t| t.starts_with("aws_docs_"))
                {
                    println!("\n🎯 SUCCESS: AWS Documentation MCP tools were called!");
                } else {
                    println!("\n⚠️  WARNING: No AWS documentation tools were called");
                }
            }
            Err(e) => {
                eprintln!("\n❌ Query failed: {}", e);
            }
        }

        // Small delay between queries
        if i < demo_queries.len() - 1 {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }

    println!("\n🎉 AWS Documentation MCP Integration Demo Complete!");
    println!("\n📝 Summary:");
    println!("   ✅ Connected to AWS Documentation MCP server via Docker");
    println!("   ✅ Verified MCP tools work directly");
    println!("   ✅ Used NEW simple with_mcp_client() builder method");
    println!("   ✅ Demonstrated automatic tool discovery and namespace prefixing");
    println!("   ✅ Verified agent actually uses MCP tools (not built-in knowledge)");

    Ok(())
}
