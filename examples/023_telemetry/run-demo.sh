#!/bin/bash
# Stood Telemetry Demo Runner
# This script compiles and runs the telemetry demo with proper configuration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${BLUE}🚀 Stood Telemetry Demo Runner${NC}"
echo -e "${BLUE}===============================${NC}"

# Check prerequisites
check_prerequisites() {
    echo -e "${CYAN}📋 Checking prerequisites...${NC}"
    
    # Check Rust/Cargo
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Cargo/Rust is not installed. Please install Rust first.${NC}"
        echo -e "${YELLOW}   Visit: https://rustup.rs/${NC}"
        exit 1
    fi
    
    # Check Docker stack
    if ! curl -s http://localhost:4320 > /dev/null 2>&1; then
        echo -e "${RED}❌ OpenTelemetry Collector not running at localhost:4320${NC}"
        echo -e "${YELLOW}   Run: ./setup-telemetry.sh${NC}"
        exit 1
    fi
    
    # Check AWS credentials
    if [[ -z "$AWS_PROFILE" && -z "$AWS_ACCESS_KEY_ID" ]]; then
        echo -e "${YELLOW}⚠️  No AWS credentials detected. The demo will run but Bedrock calls may fail.${NC}"
        echo -e "${YELLOW}   Set AWS_PROFILE or AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY${NC}"
    fi
    
    echo -e "${GREEN}✅ Prerequisites check passed${NC}"
}

# Set environment variables for optimal telemetry
setup_environment() {
    echo -e "${CYAN}⚙️ Configuring telemetry environment...${NC}"
    
    # OpenTelemetry configuration (now with smart auto-detection!)
    export OTEL_SERVICE_NAME=stood-telemetry-demo
    export OTEL_SERVICE_VERSION=1.0.0
    # OTEL_EXPORTER_OTLP_ENDPOINT is now auto-detected!
    # OTEL_ENABLED defaults to auto-detect
    export OTEL_CONSOLE_EXPORT=true
    export OTEL_RESOURCE_ATTRIBUTES="service.name=stood-telemetry-demo,service.version=1.0.0,deployment.environment=demo"
    
    # Stood-specific telemetry configuration
    export STOOD_TELEMETRY_ENABLED=true
    export STOOD_TELEMETRY_DEBUG=true
    export STOOD_TELEMETRY_BATCH_SIZE=256
    export STOOD_TELEMETRY_TIMEOUT=2000
    
    # Rust logging configuration
    export RUST_LOG="stood=debug,telemetry_demo=info,tracing=info,opentelemetry=debug"
    
    # AWS configuration defaults
    export AWS_REGION="${AWS_REGION:-us-west-2}"
    
    echo -e "${GREEN}✅ Environment configured for telemetry${NC}"
}

# Compile the demo
compile_demo() {
    echo -e "${CYAN}🔨 Compiling telemetry demo...${NC}"
    
    cd "$SCRIPT_DIR"
    
    # Clean previous builds for a fresh start
    cargo clean
    
    # Compile with always-on telemetry
    echo -e "${YELLOW}   Building with always-on telemetry...${NC}"
    if cargo build --bin telemetry_demo --release; then
        echo -e "${GREEN}✅ Compilation successful${NC}"
    else
        echo -e "${RED}❌ Compilation failed${NC}"
        exit 1
    fi
}

# Run the demo
run_demo() {
    echo -e "${CYAN}🎯 Starting telemetry demo...${NC}"
    
    cd "$SCRIPT_DIR"
    
    echo -e "${YELLOW}📊 Monitoring endpoints:${NC}"
    echo -e "   Prometheus: ${CYAN}http://localhost:9090${NC}"
    echo -e "   Grafana:    ${CYAN}http://localhost:3000${NC} (admin/admin)"
    echo -e "   Jaeger:     ${CYAN}http://localhost:16686${NC}"
    echo -e ""
    echo -e "${YELLOW}🔄 Starting demo... (Press Ctrl+C to stop)${NC}"
    echo -e "${BLUE}===============================================${NC}"
    
    # Run the demo with always-on telemetry
    cargo run --bin telemetry_demo --release
}

# Show help
show_help() {
    echo -e "${BLUE}Stood Telemetry Demo Runner${NC}"
    echo -e "${BLUE}===========================${NC}"
    echo ""
    echo -e "${YELLOW}Usage:${NC}"
    echo -e "  $0 [command]"
    echo ""
    echo -e "${YELLOW}Commands:${NC}"
    echo -e "  ${GREEN}run${NC}      Compile and run the demo (default)"
    echo -e "  ${GREEN}compile${NC}  Just compile the demo"
    echo -e "  ${GREEN}check${NC}    Check prerequisites"
    echo -e "  ${GREEN}env${NC}      Show environment configuration"
    echo -e "  ${GREEN}help${NC}     Show this help message"
    echo ""
    echo -e "${YELLOW}Prerequisites:${NC}"
    echo -e "  1. Run ${CYAN}./setup-telemetry.sh${NC} first"
    echo -e "  2. Configure AWS credentials"
    echo -e "  3. Ensure ports 4317, 9090, 3000, 16686 are available"
}

# Show environment configuration
show_environment() {
    echo -e "${CYAN}📋 Telemetry Environment Configuration${NC}"
    echo -e "${CYAN}======================================${NC}"
    
    setup_environment
    
    echo -e "${YELLOW}OpenTelemetry:${NC}"
    echo -e "  OTEL_ENABLED=${OTEL_ENABLED}"
    echo -e "  OTEL_SERVICE_NAME=${OTEL_SERVICE_NAME}"
    echo -e "  OTEL_EXPORTER_OTLP_ENDPOINT=${OTEL_EXPORTER_OTLP_ENDPOINT}"
    echo -e "  OTEL_CONSOLE_EXPORT=${OTEL_CONSOLE_EXPORT}"
    
    echo -e "${YELLOW}Stood Configuration:${NC}"
    echo -e "  STOOD_TELEMETRY_ENABLED=${STOOD_TELEMETRY_ENABLED}"
    echo -e "  STOOD_TELEMETRY_DEBUG=${STOOD_TELEMETRY_DEBUG}"
    echo -e "  STOOD_TELEMETRY_BATCH_SIZE=${STOOD_TELEMETRY_BATCH_SIZE}"
    
    echo -e "${YELLOW}AWS Configuration:${NC}"
    echo -e "  AWS_REGION=${AWS_REGION}"
    echo -e "  AWS_PROFILE=${AWS_PROFILE:-"(not set)"}"
    echo -e "  AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID:+"(set)"}${AWS_ACCESS_KEY_ID:-"(not set)"}"
    
    echo -e "${YELLOW}Rust Configuration:${NC}"
    echo -e "  RUST_LOG=${RUST_LOG}"
}

# Main execution
main() {
    case "${1:-run}" in
        run)
            check_prerequisites
            setup_environment
            compile_demo
            run_demo
            ;;
        compile)
            check_prerequisites  
            setup_environment
            compile_demo
            ;;
        check)
            check_prerequisites
            echo -e "${GREEN}🎉 All prerequisites satisfied!${NC}"
            ;;
        env)
            show_environment
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            echo -e "${RED}❌ Unknown command: $1${NC}"
            show_help
            exit 1
            ;;
    esac
}

# Handle interruption gracefully
trap 'echo -e "\n${YELLOW}🛑 Demo stopped by user${NC}"; exit 0' INT

# Run main function with all arguments
main "$@"