# MCP Test Servers

This directory contains minimal Model Context Protocol (MCP) server implementations for testing the Stood library's MCP client functionality.

## Available Servers

### Python Test Server (`python-server.py`)
- **Tools**: `echo`, `add`, `get_time`
- **Transport**: stdio
- **Dependencies**: Python 3.11+ (standard library only)

### Node.js Test Server (`node-server.js`)
- **Tools**: `echo`, `multiply`, `get_env`
- **Transport**: stdio
- **Dependencies**: Node.js 14+

## Running Test Servers

### Direct Execution

```bash
# Python server
python3 python-server.py

# Node.js server
node node-server.js
```

### Docker Execution

```bash
# Build containers
docker-compose build

# Run Python server
docker-compose run --rm python-mcp-server

# Run Node.js server
docker-compose run --rm node-mcp-server
```

## Testing Protocol

Both servers implement the MCP protocol over stdio transport:

1. **Initialize**: Establish protocol version and capabilities
2. **List Tools**: Query available tools
3. **Call Tool**: Execute specific tools with parameters

### Example Usage

```bash
# Test Python server
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","clientInfo":{"name":"test-client","version":"1.0.0"}}}' | python3 python-server.py

# Test Node.js server
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | node node-server.js
```

## Integration with Stood

These servers are used by Stood's integration tests to validate real MCP protocol compliance:

- **Protocol validation**: JSON-RPC 2.0 message format
- **Transport testing**: stdio communication
- **Tool execution**: Real tool calls and responses
- **Error handling**: Protocol error scenarios

## Server Capabilities

| Server | Tool 1 | Tool 2 | Tool 3 | Special Features |
|--------|---------|---------|---------|------------------|
| Python | `echo` | `add` | `get_time` | Standard library only |
| Node.js | `echo` | `multiply` | `get_env` | Environment access |