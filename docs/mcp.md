# Model Context Protocol (MCP) Integration

Stood provides simplified MCP integration through the Agent builder pattern, enabling seamless connection to external MCP servers for expanded tool capabilities.

## Quick Start

### Single MCP Server Integration

The simplest way to add MCP tools to your agent:

```rust
use stood::agent::Agent;
use stood::mcp::{MCPClient, MCPClientConfig};
use stood::mcp::transport::{TransportFactory, StdioConfig};
use stood::llm::models::Bedrock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure MCP client
    let config = StdioConfig {
        command: "uvx".to_string(),
        args: vec!["awslabs.core-mcp-server@latest".to_string()],
        env_vars: [("FASTMCP_LOG_LEVEL".to_string(), "ERROR".to_string())].into(),
        ..Default::default()
    };

    let transport = TransportFactory::stdio(config);
    let mut mcp_client = MCPClient::new(MCPClientConfig::default(), transport);
    mcp_client.connect().await?;

    // Create agent with MCP integration
    let mut agent = Agent::builder()
        .model(Bedrock::Claude35Haiku)
        .system_prompt("You are a helpful assistant with access to MCP tools.")
        .with_mcp_client(mcp_client, Some("aws_".to_string())).await?
        .build().await?;

    let response = agent.execute("List AWS services using your tools").await?;
    println!("{}", response.response);

    Ok(())
}
```

### Multiple MCP Servers

For integrating multiple MCP servers:

```rust
// Set up multiple MCP clients
let stdio_client = /* ... configure STDIO client ... */;
let ws_client = /* ... configure WebSocket client ... */;

// Connect all clients
stdio_client.connect().await?;
ws_client.connect().await?;

// Create agent with multiple MCP servers
let mut agent = Agent::builder()
    .model(Bedrock::Claude35Haiku)
    .system_prompt("You are an assistant with access to multiple MCP tool sets.")
    .with_mcp_clients(vec![
        (stdio_client, Some("aws_".to_string())),    // AWS tools with aws_ prefix
        (ws_client, Some("ws_".to_string())),        // WebSocket tools with ws_ prefix
    ]).await?
    .build().await?;
```

## Transport Options

### WebSocket Transport

For network-based MCP servers:

```rust
use stood::mcp::transport::{TransportFactory, WebSocketConfig};

let config = WebSocketConfig {
    url: "wss://api.example.com/mcp".to_string(),
    connect_timeout_ms: 10_000,
    ping_interval_ms: Some(30_000),
    ..Default::default()
};

let transport = TransportFactory::websocket(config);
```

### Stdio Transport  

For local process-based servers:

```rust
use stood::mcp::transport::{TransportFactory, StdioConfig};

let config = StdioConfig {
    command: "node".to_string(),
    args: vec!["mcp-server.js".to_string()],
    working_dir: Some("/path/to/server".to_string()),
    env_vars: [("MCP_ENV".to_string(), "production".to_string())].into(),
    ..Default::default()
};

let transport = TransportFactory::stdio(config);
```

## Key Features

### Automatic Tool Discovery
The builder methods automatically discover and register all available tools from MCP servers:

```rust
// Tools are discovered and registered automatically
let agent = Agent::builder()
    .with_mcp_client(mcp_client, Some("prefix_".to_string())).await?
    .build().await?;

// Agent now has access to all MCP server tools with the specified prefix
```

### Namespace Prefixing
Optional namespace prefixes prevent tool name conflicts when using multiple servers:

```rust
.with_mcp_client(aws_client, Some("aws_".to_string())).await?  // aws_list_buckets
.with_mcp_client(db_client, Some("db_".to_string())).await?    // db_query
```

### Connection Validation
Builder methods verify MCP clients are connected before use and provide helpful error messages if connection fails.

## Best Practices

### Always Connect First
Ensure `mcp_client.connect().await?` is called before adding to agent:

```rust
let mut mcp_client = MCPClient::new(config, transport);
mcp_client.connect().await?;  // Required before use

let agent = Agent::builder()
    .with_mcp_client(mcp_client, Some("prefix_".to_string())).await?
    .build().await?;
```

### Use Meaningful Namespaces
Provide descriptive namespace prefixes to avoid tool name conflicts:

```rust
.with_mcp_clients(vec![
    (aws_client, Some("aws_".to_string())),        // Clear purpose
    (database_client, Some("db_".to_string())),    // Descriptive
    (file_client, Some("file_".to_string())),      // Concise
]).await?
```

### Handle Connection Failures Gracefully
```rust
match mcp_client.connect().await {
    Ok(_) => {
        // Proceed with agent creation
        let agent = Agent::builder()
            .with_mcp_client(mcp_client, Some("prefix_".to_string())).await?
            .build().await?;
    }
    Err(e) => {
        println!("MCP connection failed: {}. Continuing without MCP tools.", e);
        // Create agent with built-in tools only
        let agent = Agent::builder()
            .build().await?;
    }
}
```

## Complete Examples

For complete working examples, see:
- [013_mcp_integration.rs](../examples/013_mcp_integration.rs) - Single MCP server integration
- [014_mcp_configuration_examples.rs](../examples/014_mcp_configuration_examples.rs) - Multiple servers and advanced configuration

## See Also

- [Architecture](architecture.md) - Overall system design
- [Tools](tools.md) - Tool development and integration
- [Examples](examples.md) - Complete usage examples