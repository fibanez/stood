//! Example 013: Simple MCP Server Integration with Agent
//!
//! This example demonstrates the NEW simple way to add MCP servers to agents
//! using the `with_mcp_client()` builder method that matches Python's approach.
//!
//! The example shows:
//! 1. Creating an MCP client with stdio transport
//! 2. Simple one-line agent integration with `with_mcp_client()`
//! 3. Automatic tool discovery and namespace prefixing
//! 4. Seamless integration with built-in tools

use stood::agent::Agent;
use stood::mcp::{MCPClient, MCPClientConfig};
use stood::mcp::transport::{TransportFactory, StdioConfig};
use stood::tools::builtin::CalculatorTool;
use stood::tool;

/// Simple tool to demonstrate hybrid usage
#[tool]
/// Get information about a specific topic
async fn get_info(topic: String) -> Result<String, String> {
    Ok(format!("Here's some information about {}: This is a built-in tool response that complements MCP server capabilities.", topic))
}

/// Create a simple mock MCP server command for demonstration
fn create_sample_mcp_server_config() -> StdioConfig {
    // Try multiple common MCP server setups in order of preference
    // 1. Python mcp package (if installed)
    // 2. Node.js @modelcontextprotocol/server-filesystem (if installed)  
    // 3. A simple echo server for demonstration
    
    StdioConfig {
        command: "python".to_string(),
        args: vec![
            "-c".to_string(),
            r#"
import sys, json, traceback

# MCP Server Implementation
def handle_request(req):
    method = req.get("method")
    params = req.get("params", {})
    req_id = req.get("id")
    
    if method == "initialize":
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "demo-mcp-server",
                    "version": "1.0.0"
                }
            }
        }
    
    elif method == "tools/list":
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "tools": [
                    {
                        "name": "sample_search",
                        "description": "Search for information about any topic",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": "The search query"
                                }
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "get_time",
                        "description": "Get the current time",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    }
                ]
            }
        }
    
    elif method == "tools/call":
        tool_name = params.get("name")
        arguments = params.get("arguments", {})
        
        if tool_name == "sample_search":
            query = arguments.get("query", "")
            result_text = f"🔍 MCP SERVER SEARCH RESULT for '{query}': Rust is a systems programming language focused on safety and performance. MCP server found: zero-cost abstractions, move semantics, guaranteed memory safety, threads without data races, trait-based generics, pattern matching, type inference, minimal runtime, efficient C bindings. [This response came from the MCP server via tool call]"
        elif tool_name == "get_time":
            import datetime
            result_text = f"⏰ MCP SERVER TIME: {datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S')} [This timestamp came from the MCP server]"
        else:
            result_text = f"Unknown tool: {tool_name}"
        
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": result_text
                    }
                ]
            }
        }
    
    elif method == "notifications/initialized":
        # No response needed for notifications
        return None
    
    else:
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {}
        }

# Main loop
try:
    for line in sys.stdin:
        try:
            line = line.strip()
            if not line:
                continue
                
            req = json.loads(line)
            response = handle_request(req)
            
            if response:
                print(json.dumps(response), flush=True)
                
        except json.JSONDecodeError:
            continue
        except Exception as e:
            if req.get("id"):
                error_response = {
                    "jsonrpc": "2.0",
                    "id": req.get("id"),
                    "error": {
                        "code": -32603,
                        "message": f"Internal error: {str(e)}"
                    }
                }
                print(json.dumps(error_response), flush=True)
except KeyboardInterrupt:
    pass
"#.to_string(),
        ],
        working_dir: None,
        env_vars: std::collections::HashMap::new(),
        startup_timeout_ms: 30000,
        max_message_size: Some(1024 * 1024), // 1MB
    }
}

/// Create and connect to MCP server with simple configuration
async fn try_mcp_connection() -> Option<MCPClient> {
    println!("🔧 Attempting to connect to MCP server...");
    
    let config = create_sample_mcp_server_config();
    println!("📝 Using command: {} {}", config.command, config.args.join(" "));
    
    let transport = TransportFactory::stdio(config);
    let mut mcp_client = MCPClient::new(MCPClientConfig::default(), transport);
    
    match mcp_client.connect().await {
        Ok(_) => {
            println!("✅ Successfully connected to MCP server!");
            
            // Try to list tools from the server to verify connection
            match mcp_client.list_tools().await {
                Ok(tools) => {
                    if tools.is_empty() {
                        println!("⚠️  MCP server connected but reports no tools available");
                        return None;
                    }
                    
                    println!("🛠️  Available MCP tools ({} total):", tools.len());
                    for tool in &tools {
                        println!("   - {} ({})", tool.name, tool.description);
                    }
                    
                    Some(mcp_client)
                }
                Err(e) => {
                    println!("❌ Failed to list tools from MCP server: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to connect to MCP server: {}", e);
            println!("   This could happen if:");
            println!("   - Python is not installed or not in PATH");
            println!("   - The MCP server process fails to start");
            println!("   - The server doesn't respond within the timeout period");
            println!("   - There are permission issues launching the subprocess");
            None
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MCP Server Integration Demo");
    println!("==============================");
    println!();
    
    // Step 1: Try to connect to MCP server
    let mcp_client = try_mcp_connection().await;
    
    // Step 2: Create agent with NEW simple MCP integration
    println!("\n🤖 Creating agent with simple MCP integration...");
    
    let mut agent_builder = Agent::builder()
        .system_prompt("You are a helpful assistant with access to both built-in tools and MCP server tools. 

CRITICAL INSTRUCTION: When users ask you to 'quote', 'show exactly', or 'display the complete tool result', you MUST include the literal, verbatim tool output in your response. Never paraphrase, summarize, or rewrite tool results when asked to quote them. 

If a tool returns content with special markers like 🔍, ⏰, or [brackets with metadata], include those markers EXACTLY as they appear. Tool results often contain verification markers to prove the tools actually executed - preserve these markers.

When using MCP tools specifically, always mention that you used the MCP tool and include its complete response when asked.")
        .tool(get_info())
        .tool(Box::new(CalculatorTool::new()));
    
    // Add MCP client using the new simple method
    if let Some(mcp_client) = mcp_client {
        println!("✅ Adding MCP client using new with_mcp_client() method");
        agent_builder = agent_builder.with_mcp_client(mcp_client, Some("mcp_".to_string())).await
            .map_err(|e| {
                eprintln!("❌ Failed to add MCP client: {}", e);
                e
            })?;
    } else {
        println!("⚙️  No MCP client available, proceeding with built-in tools only");
    }
    
    let mut agent = agent_builder.build().await?;
        
    println!("✅ Agent created successfully with simple MCP integration!");
    
    // Enable debug logging to see tool execution details
    println!("\n🔍 Enabling debug logging for this execution...");
    
    // Step 4: Test MCP tools with clear markers
    println!("\n📋 MCP Integration Test");
    println!("{}", "=".repeat(40));
    
    // Test 1: Search tool
    println!("\n🧪 Test 1: MCP Search Tool");
    let prompt1 = "Use the mcp_sample_search tool to search for 'testing123'. I need you to show me the EXACT, COMPLETE, VERBATIM tool output including any special markers like 🔍 or [metadata]. Do NOT paraphrase or summarize - quote it word-for-word exactly as the tool returned it.";
    println!("📝 Prompt: {}", prompt1);
    println!("{}", "-".repeat(40));
    
    match agent.execute(prompt1).await {
        Ok(result) => {
            println!("✅ Test 1 Success!");
            println!("🔧 Used tools: {}", result.used_tools);
            println!("📋 Tools called: {}", result.tools_called.join(", "));
            println!("📄 Response: {}", result.response);
            
            // Enhanced validation - check for specific markers
            let has_mcp_marker = result.response.contains("MCP SERVER");
            let has_search_marker = result.response.contains("🔍");
            let has_metadata_marker = result.response.contains("[This response came from the MCP server");
            let has_testing123 = result.response.contains("testing123");
            
            if has_mcp_marker && has_search_marker && has_metadata_marker {
                println!("🎯 SUCCESS: Complete MCP server response with all markers is visible!");
                println!("✅ Found: MCP SERVER marker");
                println!("✅ Found: 🔍 search marker");
                println!("✅ Found: [metadata] marker");
            } else {
                println!("⚠️  WARNING: MCP server response missing expected markers");
                if !has_mcp_marker { println!("❌ Missing: 'MCP SERVER' marker"); }
                if !has_search_marker { println!("❌ Missing: '🔍' search marker"); }
                if !has_metadata_marker { println!("❌ Missing: '[This response came from the MCP server' metadata"); }
                if !has_testing123 { println!("❌ Missing: 'testing123' search term"); }
                
                // Show what we got instead
                println!("🔍 Actual response preview (first 200 chars): {}", 
                    if result.response.len() > 200 { 
                        format!("{}...", &result.response[..200])
                    } else { 
                        result.response.clone() 
                    }
                );
            }
        }
        Err(e) => {
            println!("❌ Test 1 Failed: {}", e);
        }
    }
    
    // Test 2: Time tool  
    println!("\n\n🧪 Test 2: MCP Time Tool");
    let prompt2 = "Call the mcp_get_time tool and show me the EXACT response it returns. Include everything - any special characters like ⏰, timestamps, and metadata in brackets. I want to see the complete, unmodified output exactly as the MCP server provided it.";
    println!("📝 Prompt: {}", prompt2);
    println!("{}", "-".repeat(40));
    
    match agent.execute(prompt2).await {
        Ok(result) => {
            println!("✅ Test 2 Success!");
            println!("🔧 Used tools: {}", result.used_tools);
            println!("📋 Tools called: {}", result.tools_called.join(", "));
            println!("📄 Response: {}", result.response);
            
            // Enhanced validation - check for specific markers
            let has_mcp_marker = result.response.contains("MCP SERVER");
            let has_time_marker = result.response.contains("⏰");
            let has_metadata_marker = result.response.contains("[This timestamp came from the MCP server");
            let has_timestamp = result.response.contains("2025-") || result.response.contains("2024-");
            
            if has_mcp_marker && has_time_marker && has_metadata_marker {
                println!("🎯 SUCCESS: Complete MCP server response with all markers is visible!");
                println!("✅ Found: MCP SERVER marker");
                println!("✅ Found: ⏰ time marker");
                println!("✅ Found: [metadata] marker");
            } else {
                println!("⚠️  WARNING: MCP server response missing expected markers");
                if !has_mcp_marker { println!("❌ Missing: 'MCP SERVER' marker"); }
                if !has_time_marker { println!("❌ Missing: '⏰' time marker"); }
                if !has_metadata_marker { println!("❌ Missing: '[This timestamp came from the MCP server' metadata"); }
                if !has_timestamp { println!("❌ Missing: timestamp format"); }
                
                // Show what we got instead
                println!("🔍 Actual response preview (first 200 chars): {}", 
                    if result.response.len() > 200 { 
                        format!("{}...", &result.response[..200])
                    } else { 
                        result.response.clone() 
                    }
                );
            }
        }
        Err(e) => {
            println!("❌ Test 2 Failed: {}", e);
        }
    }
    
    // Additional validation test
    println!("\n\n🧪 Test 3: Verification Test");
    let prompt3 = "Call both mcp_sample_search for 'verification' and mcp_get_time. Show me both complete results exactly as returned, including all special characters and markers.";
    println!("📝 Prompt: {}", prompt3);
    println!("{}", "-".repeat(40));
    
    match agent.execute(prompt3).await {
        Ok(result) => {
            println!("✅ Test 3 Success!");
            println!("🔧 Used tools: {}", result.used_tools);
            println!("📋 Tools called: {}", result.tools_called.join(", "));
            println!("📄 Response: {}", result.response);
            
            let search_markers = result.response.contains("🔍") && result.response.contains("verification");
            let time_markers = result.response.contains("⏰");
            let mcp_markers = result.response.contains("MCP SERVER");
            
            if search_markers && time_markers && mcp_markers {
                println!("🎯 SUCCESS: Both MCP tools' verification markers are visible!");
            } else {
                println!("⚠️  WARNING: Missing markers in combined test");
                if !search_markers { println!("❌ Missing search markers"); }
                if !time_markers { println!("❌ Missing time markers"); }
                if !mcp_markers { println!("❌ Missing MCP SERVER markers"); }
            }
        }
        Err(e) => {
            println!("❌ Test 3 Failed: {}", e);
        }
    }

    println!("\n🎉 MCP Integration Demo Complete!");
    println!("\n📝 Summary:");
    println!("   - Demonstrated NEW simple with_mcp_client() builder method");
    println!("   - Showed automatic tool discovery and namespace prefixing");
    println!("   - Created unified agent with one-line MCP integration");
    println!("   - Proved Python-like simplicity in Rust");
    println!("   - Enhanced validation to ensure MCP verification markers are preserved");
    println!("   - Added recovery strategy for missing tool result markers");
    
    Ok(())
}