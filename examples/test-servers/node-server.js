#!/usr/bin/env node
/**
 * Minimal Node.js MCP Test Server
 * 
 * A simple Model Context Protocol server implementation for testing the Rust MCP client.
 * Supports basic tools: echo, multiply, and get_env for comprehensive protocol testing.
 */

const readline = require('readline');

class MCPTestServer {
    constructor() {
        this.serverInfo = {
            name: 'nodejs-test-server',
            version: '1.0.0'
        };
        
        this.capabilities = {
            tools: { listChanged: true }
        };
        
        this.tools = [
            {
                name: 'echo',
                description: 'Echo back the input text with Node.js prefix',
                inputSchema: {
                    type: 'object',
                    properties: {
                        text: {
                            type: 'string',
                            description: 'Text to echo back'
                        }
                    },
                    required: ['text']
                }
            },
            {
                name: 'multiply',
                description: 'Multiply two numbers together',
                inputSchema: {
                    type: 'object',
                    properties: {
                        a: {
                            type: 'number',
                            description: 'First number'
                        },
                        b: {
                            type: 'number',
                            description: 'Second number'
                        }
                    },
                    required: ['a', 'b']
                }
            },
            {
                name: 'get_env',
                description: 'Get environment variable value',
                inputSchema: {
                    type: 'object',
                    properties: {
                        name: {
                            type: 'string',
                            description: 'Environment variable name'
                        }
                    },
                    required: ['name']
                }
            }
        ];
    }

    async handleInitialize(request) {
        return {
            jsonrpc: '2.0',
            id: request.id,
            result: {
                protocolVersion: '2025-03-26',
                capabilities: this.capabilities,
                serverInfo: this.serverInfo
            }
        };
    }

    async handleListTools(request) {
        return {
            jsonrpc: '2.0',
            id: request.id,
            result: {
                tools: this.tools
            }
        };
    }

    async handleCallTool(request) {
        const params = request.params || {};
        const toolName = params.name;
        const args = params.arguments || {};

        try {
            let content;

            switch (toolName) {
                case 'echo':
                    const text = args.text || '';
                    content = [{
                        type: 'text',
                        text: `Node.js Echo: ${text}`
                    }];
                    break;

                case 'multiply':
                    const a = args.a || 0;
                    const b = args.b || 0;
                    const result = a * b;
                    content = [{
                        type: 'text',
                        text: `Result: ${result}`
                    }];
                    break;

                case 'get_env':
                    const envName = args.name;
                    const envValue = process.env[envName];
                    content = [{
                        type: 'text',
                        text: envValue ? `${envName}=${envValue}` : `Environment variable '${envName}' not found`
                    }];
                    break;

                default:
                    return {
                        jsonrpc: '2.0',
                        id: request.id,
                        error: {
                            code: -32601,
                            message: `Unknown tool: ${toolName}`
                        }
                    };
            }

            return {
                jsonrpc: '2.0',
                id: request.id,
                result: {
                    content: content
                }
            };
        } catch (error) {
            return {
                jsonrpc: '2.0',
                id: request.id,
                error: {
                    code: -32603,
                    message: `Tool execution error: ${error.message}`
                }
            };
        }
    }

    async handleRequest(request) {
        const method = request.method;

        switch (method) {
            case 'initialize':
                return await this.handleInitialize(request);
            case 'tools/list':
                return await this.handleListTools(request);
            case 'tools/call':
                return await this.handleCallTool(request);
            default:
                return {
                    jsonrpc: '2.0',
                    id: request.id,
                    error: {
                        code: -32601,
                        message: `Method not found: ${method}`
                    }
                };
        }
    }

    runStdio() {
        console.error('Node.js MCP Test Server starting (stdio mode)');

        const rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout,
            terminal: false
        });

        rl.on('line', async (line) => {
            try {
                line = line.trim();
                if (!line) return;

                // Parse JSON request
                let request;
                try {
                    request = JSON.parse(line);
                } catch (error) {
                    const errorResponse = {
                        jsonrpc: '2.0',
                        id: null,
                        error: {
                            code: -32700,
                            message: `Parse error: ${error.message}`
                        }
                    };
                    console.log(JSON.stringify(errorResponse));
                    return;
                }

                // Handle request
                const response = await this.handleRequest(request);

                // Send response
                console.log(JSON.stringify(response));
            } catch (error) {
                console.error(`Server error: ${error.message}`);
            }
        });

        rl.on('close', () => {
            console.error('Node.js MCP Test Server stopped');
            process.exit(0);
        });

        // Handle SIGINT (Ctrl+C)
        process.on('SIGINT', () => {
            console.error('Server interrupted');
            rl.close();
        });
    }
}

function main() {
    const server = new MCPTestServer();
    
    try {
        server.runStdio();
    } catch (error) {
        console.error(`Server failed: ${error.message}`);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}