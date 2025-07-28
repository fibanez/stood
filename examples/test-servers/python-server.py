#!/usr/bin/env python3
"""
Minimal Python MCP Test Server

A simple Model Context Protocol server implementation for testing the Rust MCP client.
Supports basic tools: echo, add, and get_time for comprehensive protocol testing.
"""

import asyncio
import json
import sys
import datetime
from typing import Dict, Any, List


class MCPTestServer:
    """Minimal MCP server for testing protocol compliance"""
    
    def __init__(self):
        self.server_info = {
            "name": "python-test-server",
            "version": "1.0.0"
        }
        self.capabilities = {
            "tools": {"listChanged": True}
        }
        self.tools = [
            {
                "name": "echo",
                "description": "Echo back the input text",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "Text to echo back"
                        }
                    },
                    "required": ["text"]
                }
            },
            {
                "name": "add",
                "description": "Add two numbers together",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "a": {
                            "type": "number",
                            "description": "First number"
                        },
                        "b": {
                            "type": "number",
                            "description": "Second number"
                        }
                    },
                    "required": ["a", "b"]
                }
            },
            {
                "name": "get_time",
                "description": "Get the current time",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "additionalProperties": False
                }
            }
        ]

    async def handle_initialize(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Handle initialize request"""
        return {
            "jsonrpc": "2.0",
            "id": request["id"],
            "result": {
                "protocolVersion": "2025-03-26",
                "capabilities": self.capabilities,
                "serverInfo": self.server_info
            }
        }

    async def handle_list_tools(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Handle tools/list request"""
        return {
            "jsonrpc": "2.0",
            "id": request["id"],
            "result": {
                "tools": self.tools
            }
        }

    async def handle_call_tool(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Handle tools/call request"""
        params = request.get("params", {})
        tool_name = params.get("name")
        arguments = params.get("arguments", {})
        
        try:
            if tool_name == "echo":
                text = arguments.get("text", "")
                content = [{
                    "type": "text",
                    "text": f"Echo: {text}"
                }]
            elif tool_name == "add":
                a = arguments.get("a", 0)
                b = arguments.get("b", 0)
                result = a + b
                content = [{
                    "type": "text",
                    "text": f"Result: {result}"
                }]
            elif tool_name == "get_time":
                current_time = datetime.datetime.now().isoformat()
                content = [{
                    "type": "text",
                    "text": f"Current time: {current_time}"
                }]
            else:
                return {
                    "jsonrpc": "2.0",
                    "id": request["id"],
                    "error": {
                        "code": -32601,
                        "message": f"Unknown tool: {tool_name}"
                    }
                }
            
            return {
                "jsonrpc": "2.0",
                "id": request["id"],
                "result": {
                    "content": content
                }
            }
        except Exception as e:
            return {
                "jsonrpc": "2.0",
                "id": request["id"],
                "error": {
                    "code": -32603,
                    "message": f"Tool execution error: {str(e)}"
                }
            }

    async def handle_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Route request to appropriate handler"""
        method = request.get("method")
        
        if method == "initialize":
            return await self.handle_initialize(request)
        elif method == "tools/list":
            return await self.handle_list_tools(request)
        elif method == "tools/call":
            return await self.handle_call_tool(request)
        else:
            return {
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "error": {
                    "code": -32601,
                    "message": f"Method not found: {method}"
                }
            }

    async def run_stdio(self):
        """Run server over stdio transport"""
        print("Python MCP Test Server starting (stdio mode)", file=sys.stderr)
        
        while True:
            try:
                # Read line from stdin
                line = await asyncio.get_event_loop().run_in_executor(
                    None, sys.stdin.readline
                )
                
                if not line:
                    break
                
                line = line.strip()
                if not line:
                    continue
                
                # Parse JSON request
                try:
                    request = json.loads(line)
                except json.JSONDecodeError as e:
                    error_response = {
                        "jsonrpc": "2.0",
                        "id": None,
                        "error": {
                            "code": -32700,
                            "message": f"Parse error: {str(e)}"
                        }
                    }
                    print(json.dumps(error_response), flush=True)
                    continue
                
                # Handle request
                response = await self.handle_request(request)
                
                # Send response
                print(json.dumps(response), flush=True)
                
            except KeyboardInterrupt:
                break
            except Exception as e:
                print(f"Server error: {e}", file=sys.stderr)
                break
        
        print("Python MCP Test Server stopped", file=sys.stderr)


def main():
    """Main entry point"""
    server = MCPTestServer()
    
    try:
        asyncio.run(server.run_stdio())
    except KeyboardInterrupt:
        print("Server interrupted", file=sys.stderr)
    except Exception as e:
        print(f"Server failed: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()