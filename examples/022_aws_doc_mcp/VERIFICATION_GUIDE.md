# Verifying MCP Tool Usage vs. Agent Knowledge (NEW Simple Method)

This guide shows you how to confirm that your Stood agent is actually calling MCP tools rather than using its built-in knowledge when using the NEW `with_mcp_client()` method.

## ✨ NEW Simple Method Benefits

The NEW `with_mcp_client()` approach automatically:
- ✅ Discovers and registers MCP tools
- ✅ Applies namespace prefixing
- ✅ Validates MCP client connections
- ✅ Integrates tools with agent's unified tool system

## 🔍 Built-in Verification Features

The updated example (022_aws_doc_mcp) includes automatic verification:

### Step 1: Direct MCP Tool Testing
```rust
// The example directly tests MCP tools work
verify_mcp_tools(&mut mcp_client).await?;
```

**What you'll see:**
```
🔍 Verifying MCP tools work directly...
📝 Testing 'search_documentation' tool with query: CloudFormation template basics
✅ MCP tool call successful!
🎯 Tool result preview: AWS CloudFormation templates are...
✅ MCP server is working correctly!
```

### Step 2: Agent Creation with NEW Method
```rust
// One line to add MCP tools to agent
let mut agent = Agent::builder()
    .model(Bedrock::Claude35Haiku)
    .system_prompt("You are an AWS expert...")
    .with_mcp_client(mcp_client, Some("aws_docs_".to_string())).await?  // 🎯 NEW!
    .build().await?;
```

### Step 3: Automatic Tool Usage Verification
```rust
// Agent execution with built-in verification
match agent.execute(query).await {
    Ok(result) => {
        println!("🔧 Used tools: {}", result.used_tools);
        println!("📋 Tools called: {}", result.tools_called.join(", "));
        
        // Automatic verification
        if result.tools_called.iter().any(|t| t.starts_with("aws_docs_")) {
            println!("🎯 SUCCESS: AWS Documentation MCP tools were called!");
        } else {
            println!("⚠️  WARNING: No AWS documentation tools were called");
        }
    }
}
```

## 🔧 Quick Verification Methods

### Method 1: Run the Example with Debug Logging
```bash
# Run with debug logging to see all tool calls
RUST_LOG=debug cargo run --example 022_aws_doc_mcp
```

**What to Look For:**
```
[INFO  stood::mcp::client] Connecting to MCP server...
[INFO  stood::mcp::client] Discovered tool: search_documentation
[DEBUG stood::tools::mcp_adapter] MCP TOOL 'aws_docs_search_documentation' received parameters: {...}
[INFO  stood::tools::mcp_adapter] MCP TOOL INVOCATION: Calling 'aws_docs_search_documentation'
🎯 SUCCESS: AWS Documentation MCP tools were called!
```

### Method 2: Look for Success Messages
The example provides clear indicators:

**✅ Successful MCP Usage:**
```
🎯 SUCCESS: AWS Documentation MCP tools were called!
```

**⚠️ Warning - No MCP Usage:**
```
⚠️  WARNING: No AWS documentation tools were called
```

### Method 3: Check Tool Names in Output
MCP tools have namespace prefixes:
- `aws_docs_search_documentation` = MCP tool ✅
- `search_documentation` (without prefix) = Built-in knowledge ❌

## 🐛 Troubleshooting

### Issue 1: "No MCP tools were called" Warning
**Possible Causes:**
1. Agent using built-in knowledge instead of tools
2. MCP tools not properly registered
3. Tool call failed silently

**Solutions:**
```bash
# 1. Check with debug logging
RUST_LOG=debug cargo run --example 022_aws_doc_mcp

# 2. Look for these log messages:
#    "✅ Connected to AWS Documentation MCP server"
#    "📚 Available AWS documentation tools (X total):"
#    "✅ Agent created with AWS documentation MCP tools!"

# 3. Ensure Docker image exists
docker images | grep awslabs/aws-documentation-mcp-server
```

### Issue 2: Connection Failures
**Problem**: Cannot connect to MCP server
**Solution**: 
```bash
# Build Docker image first
./docker_mcp_setup.sh

# Verify setup
./verify_setup.sh
```

### Issue 3: Tool Parameter Errors
**Problem**: MCP tool calls fail with parameter validation
**Solution**: The NEW example handles this automatically by using proper parameter schemas discovered from the MCP server.

## 🎯 Verification Checklist

Run through this checklist to confirm MCP integration:

- [ ] **Docker Image Built**: `docker images | grep awslabs/aws-documentation-mcp-server`
- [ ] **Example Compiles**: `cargo check --example 022_aws_doc_mcp`
- [ ] **MCP Connection**: Look for "✅ Connected to AWS Documentation MCP server"
- [ ] **Tool Discovery**: Look for "📚 Available AWS documentation tools"
- [ ] **Direct Tool Test**: Look for "✅ MCP tool call successful!"
- [ ] **Agent Creation**: Look for "✅ Agent created with AWS documentation MCP tools!"
- [ ] **Tool Usage**: Look for "🎯 SUCCESS: AWS Documentation MCP tools were called!"

## 🚀 Advanced Verification

### Compare with and without MCP Server
1. **With MCP server running**: You should see specific AWS documentation content
2. **Without MCP server**: Agent falls back to general knowledge

### Check Tool Call Timing
MCP tool calls typically take 2-5 seconds due to Docker container communication, while built-in knowledge is instant.

### Verify Content Sources
MCP tools return content with specific AWS documentation formatting and URLs that built-in knowledge wouldn't have.

## 📝 Summary

The NEW `with_mcp_client()` method makes verification much easier:
- ✅ Automatic connection validation
- ✅ Built-in tool usage verification
- ✅ Clear success/warning messages
- ✅ Proper namespace handling
- ✅ Direct tool testing before agent usage

No more manual tool adapter creation or complex debugging - the verification is built right into the example!