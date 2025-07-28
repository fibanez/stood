#!/bin/bash
# WebSocket MCP server management script

SCRIPT_DIR="$(dirname "$0")"
PID_FILE="$SCRIPT_DIR/websocket_server.pid"

start_server() {
    echo "Starting WebSocket MCP server on localhost:8765..."
    
    # Kill any existing server
    pkill -f websocket_mcp_server 2>/dev/null
    rm -f "$PID_FILE"
    
    # Start new server with nohup in background
    cd "$SCRIPT_DIR"
    nohup python websocket_mcp_server.py > websocket_server.log 2>&1 &
    SERVER_PID=$!
    
    # Save PID to file
    echo $SERVER_PID > "$PID_FILE"
    
    echo "✅ WebSocket MCP server started (PID: $SERVER_PID)"
    echo "   URL: ws://localhost:8765"
    echo "   PID file: $PID_FILE"
    echo "   Log file: $SCRIPT_DIR/websocket_server.log"
    
    # Wait a moment for server to start
    sleep 2
    
    # Test if server is responding
    if lsof -i :8765 >/dev/null 2>&1; then
        echo "✅ Server is listening on port 8765"
        echo ""
        echo "To stop the server, run: $0 stop"
    else
        echo "❌ Server failed to start on port 8765"
        rm -f "$PID_FILE"
        exit 1
    fi
}

stop_server() {
    echo "Stopping WebSocket MCP server..."
    
    # Try to stop using PID file first
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if kill "$PID" 2>/dev/null; then
            echo "✅ Server stopped (PID: $PID)"
        else
            echo "⚠️  PID $PID not found, trying alternative method..."
            pkill -f websocket_mcp_server
        fi
        rm -f "$PID_FILE"
    else
        echo "⚠️  PID file not found, trying to kill by process name..."
        if pkill -f websocket_mcp_server; then
            echo "✅ Server stopped"
        else
            echo "❌ No WebSocket MCP server process found"
        fi
    fi
    
    # Verify port is free
    if lsof -i :8765 >/dev/null 2>&1; then
        echo "⚠️  Port 8765 still in use"
    else
        echo "✅ Port 8765 is now free"
    fi
}

status_server() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if kill -0 "$PID" 2>/dev/null; then
            echo "✅ WebSocket MCP server is running (PID: $PID)"
            echo "   URL: ws://localhost:8765"
        else
            echo "❌ PID file exists but process is not running"
            rm -f "$PID_FILE"
        fi
    else
        if lsof -i :8765 >/dev/null 2>&1; then
            echo "⚠️  Something is running on port 8765 but no PID file found"
        else
            echo "❌ WebSocket MCP server is not running"
        fi
    fi
}

show_logs() {
    LOG_FILE="$SCRIPT_DIR/websocket_server.log"
    if [ -f "$LOG_FILE" ]; then
        echo "📄 Server logs (last 20 lines):"
        echo "────────────────────────────────"
        tail -20 "$LOG_FILE"
        echo "────────────────────────────────"
        echo "To follow logs: tail -f $LOG_FILE"
    else
        echo "❌ Log file not found: $LOG_FILE"
    fi
}

show_help() {
    echo "WebSocket MCP Server Management"
    echo "Usage: $0 {start|stop|restart|status|logs|help}"
    echo ""
    echo "Commands:"
    echo "  start   - Start the WebSocket MCP server with nohup"
    echo "  stop    - Stop the WebSocket MCP server"
    echo "  restart - Restart the WebSocket MCP server"
    echo "  status  - Check if the server is running"
    echo "  logs    - Show recent server logs"
    echo "  help    - Show this help message"
    echo ""
    echo "The server runs on ws://localhost:8765"
    echo "Logs are saved to: websocket_server.log"
}

case "$1" in
    start)
        start_server
        ;;
    stop)
        stop_server
        ;;
    restart)
        stop_server
        echo ""
        start_server
        ;;
    status)
        status_server
        ;;
    logs)
        show_logs
        ;;
    help|--help|-h)
        show_help
        ;;
    "")
        echo "No command specified. Starting server..."
        start_server
        ;;
    *)
        echo "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac