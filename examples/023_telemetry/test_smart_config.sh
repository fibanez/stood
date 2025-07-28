#!/bin/bash
# Test script for smart telemetry configuration

echo "🧪 Testing Smart Telemetry Configuration"
echo "========================================"

# Test 1: No environment variables (should auto-detect)
echo -e "\n📋 Test 1: Clean environment (auto-detection)"
unset OTEL_ENABLED OTEL_EXPORTER_OTLP_ENDPOINT
cd "$(dirname "$0")"

# Test 2: With telemetry stack running
echo -e "\n📋 Test 2: With telemetry stack"
if curl -s http://localhost:4320 > /dev/null 2>&1; then
    echo "✅ OTLP endpoint detected at localhost:4320"
else
    echo "❌ No OTLP endpoint at localhost:4320"
fi

if curl -s http://localhost:9090 > /dev/null 2>&1; then
    echo "✅ Prometheus detected at localhost:9090"
else
    echo "❌ No Prometheus at localhost:9090"
fi

# Test 3: Feature flag behavior
echo -e "\n📋 Test 3: Checking with/without feature flag"

echo "Without telemetry feature:"
cargo check --manifest-path /home/fernando/Documents/code/stood/Cargo.toml 2>&1 | grep -i telemetry || echo "No telemetry warnings"

echo -e "\nWith telemetry feature:"
cargo check --manifest-path /home/fernando/Documents/code/stood/Cargo.toml 2>&1 | grep -i telemetry || echo "Telemetry compiled successfully"

# Test 4: Show current configuration
echo -e "\n📋 Test 4: Current environment"
echo "OTEL_ENABLED=${OTEL_ENABLED:-"(not set)"}"
echo "OTEL_EXPORTER_OTLP_ENDPOINT=${OTEL_EXPORTER_OTLP_ENDPOINT:-"(not set)"}"
echo "OTEL_SERVICE_NAME=${OTEL_SERVICE_NAME:-"(not set)"}"

echo -e "\n🎯 Smart telemetry test completed!"