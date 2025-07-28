#!/bin/bash

# Test script to verify enhanced ValidationException logging
echo "Testing enhanced ValidationException logging..."

# Clean up old logs
rm -f logs/stood.log.*

# Run the CLI with debug mode to generate detailed logs
echo "Running CLI with debug mode to test logging..."
RUST_LOG=debug cargo run --example agentic_cli -- --debug agentic "Create a very long conversation that might trigger context limits by generating a lot of text and tool usage"

echo ""
echo "Checking log file for enhanced logging..."
if [ -f logs/stood.log.$(date +%Y-%m-%d) ]; then
    echo "Log file found. Checking for enhanced features..."
    
    echo ""
    echo "=== Looking for Bedrock request details ==="
    grep -A 2 -B 2 "Full request body" logs/stood.log.$(date +%Y-%m-%d) | head -20
    
    echo ""
    echo "=== Looking for conversation structure logs ==="
    grep "Conversation structure\|Message.*role=" logs/stood.log.$(date +%Y-%m-%d) | head -10
    
    echo ""
    echo "=== Looking for ValidationException patterns ==="
    grep -i "validation\|context overflow\|recovery" logs/stood.log.$(date +%Y-%m-%d) | head -10
    
    echo ""
    echo "=== Looking for pretty JSON formatting ==="
    grep -A 5 "Pretty.*JSON" logs/stood.log.$(date +%Y-%m-%d) | head -20
    
else
    echo "No log file found at logs/stood.log.$(date +%Y-%m-%d)"
fi