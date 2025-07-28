# Test MCP Servers

This directory contains test MCP servers for development and testing.

## WebSocket MCP Server

### Setup
```bash
cd examples/test-servers
pip install websockets
```

### Running the Server
```bash
python websocket_mcp_server.py
```

You should see:
```
INFO:__main__:Starting WebSocket MCP server on ws://localhost:8765
INFO:__main__:✅ WebSocket MCP server ready at ws://localhost:8765
INFO:__main__:Waiting for connections...
```

### Testing with Example 014

1. **Start the WebSocket server** in one terminal:
   ```bash
   cd examples/test-servers
   python websocket_mcp_server.py
   ```

2. **Run example 014** in another terminal:
   ```bash
   cargo run --example 014_mcp_configuration_examples
   ```

### Expected Output

When the WebSocket server is running, you should see:
```
2. WebSocket MCP Server:
✅ Connected to WebSocket MCP server
Available WebSocket MCP tools: ["websocket_search", "websocket_time"]
```

When the server is NOT running, you should see:
```
2. WebSocket MCP Server:
   Failed: WebSocket error: WebSocket connection failed: IO error: Connection refused (os error 111) (This is expected if the server isn't available)
```

## Available Tools

The WebSocket MCP server provides:

1. **websocket_search** - Search for information via WebSocket
2. **websocket_time** - Get current time from WebSocket server

Both tools return responses clearly marked with "WEBSOCKET MCP" to demonstrate the connection is working.