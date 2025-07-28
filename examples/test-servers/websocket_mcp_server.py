#!/usr/bin/env python3
"""
Simple WebSocket MCP Server for testing
Run with: python websocket_mcp_server.py
"""

import asyncio
import json
import logging
import websockets
from datetime import datetime

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class WebSocketMCPServer:
    def __init__(self):
        self.capabilities = {
            "tools": {}
        }
        
    async def handle_request(self, request):
        """Handle MCP protocol requests"""
        method = request.get("method")
        params = request.get("params", {})
        req_id = request.get("id")
        
        logger.info(f"Received request: {method}")
        
        if method == "initialize":
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": self.capabilities,
                    "serverInfo": {
                        "name": "websocket-demo-mcp-server",
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
                            "name": "websocket_search",
                            "description": "Search for information via WebSocket MCP server",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": {
                                        "type": "string",
                                        "description": "The search query"
                                    }
                                },
                                "required": ["query"],
                                "additionalProperties": False
                            }
                        },
                        {
                            "name": "websocket_time",
                            "description": "Get current time from WebSocket server",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "additionalProperties": False
                            }
                        }
                    ]
                }
            }
        
        elif method == "tools/call":
            tool_name = params.get("name")
            arguments = params.get("arguments", {})
            
            logger.info(f"Tool call: {tool_name} with arguments: {arguments}")
            
            # Validate that arguments is a dict/object
            if not isinstance(arguments, dict):
                return {
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "error": {
                        "code": -32602,
                        "message": f"Invalid parameters: arguments must be a JSON object, got {type(arguments)}"
                    }
                }
            
            if tool_name == "websocket_search":
                query = arguments.get("query", "")
                if not query:
                    return {
                        "jsonrpc": "2.0",
                        "id": req_id,
                        "error": {
                            "code": -32602,
                            "message": "Invalid parameters: 'query' is required for websocket_search"
                        }
                    }
                result_text = f"🔍 WEBSOCKET MCP SEARCH for '{query}': Found comprehensive results via WebSocket connection. Server located relevant information about {query} from distributed sources. [Response from WebSocket MCP Server]"
            elif tool_name == "websocket_time":
                result_text = f"⏰ WEBSOCKET MCP TIME: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')} [Timestamp from WebSocket MCP Server]"
            else:
                return {
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "error": {
                        "code": -32601,
                        "message": f"Method not found: Unknown tool '{tool_name}'"
                    }
                }
            
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
    
    async def handle_client(self, websocket, path):
        """Handle WebSocket client connection"""
        logger.info(f"Client connected from {websocket.remote_address}")
        
        try:
            async for message in websocket:
                try:
                    # Handle ping/pong frames automatically (websockets library handles this)
                    # Only process text messages as JSON
                    if isinstance(message, str):
                        request = json.loads(message)
                        response = await self.handle_request(request)
                        
                        if response:
                            response_json = json.dumps(response)
                            await websocket.send(response_json)
                            logger.info(f"Sent response: {response.get('result', {}).get('tools', 'N/A')}")
                    else:
                        # Skip binary frames (ping/pong are handled automatically)
                        continue
                        
                except json.JSONDecodeError as e:
                    logger.error(f"Invalid JSON received: {e}")
                    error_response = {
                        "jsonrpc": "2.0",
                        "id": None,
                        "error": {
                            "code": -32700,
                            "message": "Parse error"
                        }
                    }
                    await websocket.send(json.dumps(error_response))
                except Exception as e:
                    logger.error(f"Error handling request: {e}")
                    error_response = {
                        "jsonrpc": "2.0",
                        "id": None,
                        "error": {
                            "code": -32603,
                            "message": f"Internal error: {str(e)}"
                        }
                    }
                    await websocket.send(json.dumps(error_response))
                    
        except websockets.exceptions.ConnectionClosed:
            logger.info("Client disconnected")
        except Exception as e:
            logger.error(f"Connection error: {e}")

async def main():
    """Start the WebSocket MCP server"""
    server = WebSocketMCPServer()
    
    # Start server on localhost:8765
    host = "localhost"
    port = 8765
    
    logger.info(f"Starting WebSocket MCP server on ws://{host}:{port}")
    
    async with websockets.serve(server.handle_client, host, port):
        logger.info(f"✅ WebSocket MCP server ready at ws://{host}:{port}")
        logger.info("Waiting for connections...")
        
        # Keep server running
        await asyncio.Future()  # Run forever

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        logger.info("Server shutting down...")