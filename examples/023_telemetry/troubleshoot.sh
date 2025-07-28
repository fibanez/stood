#!/bin/bash
# Stood Telemetry Demo Troubleshooting Script
# This script diagnoses common issues and provides solutions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${BLUE}🔧 Stood Telemetry Demo Troubleshooting${NC}"
echo -e "${BLUE}=======================================${NC}"

# Test Docker availability
test_docker() {
    echo -e "\n${CYAN}🐳 Testing Docker...${NC}"
    
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}❌ Docker not installed${NC}"
        echo -e "${YELLOW}💡 Solution: Install Docker from https://docs.docker.com/get-docker/${NC}"
        return 1
    fi
    
    if ! docker info &> /dev/null; then
        echo -e "${RED}❌ Docker daemon not running${NC}"
        echo -e "${YELLOW}💡 Solution: Start Docker daemon${NC}"
        echo -e "   Linux: sudo systemctl start docker"
        echo -e "   macOS: open -a Docker"
        echo -e "   Windows: Start Docker Desktop"
        return 1
    fi
    
    if ! docker compose version &> /dev/null && ! command -v docker-compose &> /dev/null; then
        echo -e "${RED}❌ Docker Compose not available${NC}"
        echo -e "${YELLOW}💡 Solution: Install Docker Compose${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✅ Docker is working correctly${NC}"
    return 0
}

# Test port availability
test_ports() {
    echo -e "\n${CYAN}🔌 Testing port availability...${NC}"
    
    local ports=(3000 4319 4320 9090 9100 13133 16686)
    local blocked_ports=()
    
    for port in "${ports[@]}"; do
        if lsof -i :$port &> /dev/null; then
            blocked_ports+=($port)
            echo -e "${YELLOW}⚠️  Port $port is in use${NC}"
            
            # Show what's using the port
            local process=$(lsof -i :$port | tail -n1 | awk '{print $1, $2}')
            echo -e "   Used by: $process"
        fi
    done
    
    if [ ${#blocked_ports[@]} -eq 0 ]; then
        echo -e "${GREEN}✅ All required ports are available${NC}"
        return 0
    else
        echo -e "\n${RED}❌ Some ports are blocked${NC}"
        echo -e "${YELLOW}💡 Solutions:${NC}"
        echo -e "   1. Stop the telemetry stack: ./setup-telemetry.sh stop"
        echo -e "   2. Kill specific processes: sudo kill -9 PID"
        echo -e "   3. Use different ports (modify docker-compose.yml)"
        return 1
    fi
}

# Test AWS credentials
test_aws_credentials() {
    echo -e "\n${CYAN}🔐 Testing AWS credentials...${NC}"
    
    if [[ -n "$AWS_PROFILE" ]]; then
        echo -e "${BLUE}📋 Using AWS Profile: $AWS_PROFILE${NC}"
        if aws sts get-caller-identity --profile "$AWS_PROFILE" &> /dev/null; then
            local identity=$(aws sts get-caller-identity --profile "$AWS_PROFILE" 2>/dev/null)
            echo -e "${GREEN}✅ AWS Profile is valid${NC}"
            echo -e "   Account: $(echo $identity | jq -r .Account 2>/dev/null || echo "N/A")"
            echo -e "   User: $(echo $identity | jq -r .Arn 2>/dev/null || echo "N/A")"
            return 0
        else
            echo -e "${RED}❌ AWS Profile is invalid or expired${NC}"
            echo -e "${YELLOW}💡 Solution: Check your profile configuration${NC}"
            echo -e "   aws configure list --profile $AWS_PROFILE"
            return 1
        fi
    elif [[ -n "$AWS_ACCESS_KEY_ID" && -n "$AWS_SECRET_ACCESS_KEY" ]]; then
        echo -e "${BLUE}📋 Using AWS environment variables${NC}"
        if aws sts get-caller-identity &> /dev/null; then
            echo -e "${GREEN}✅ AWS credentials are valid${NC}"
            return 0
        else
            echo -e "${RED}❌ AWS environment credentials are invalid${NC}"
            echo -e "${YELLOW}💡 Solution: Check your environment variables${NC}"
            echo -e "   AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID:0:10}..."
            return 1
        fi
    else
        echo -e "${YELLOW}⚠️  No AWS credentials detected${NC}"
        echo -e "${YELLOW}💡 Solution: Configure AWS credentials${NC}"
        echo -e "   Option 1: export AWS_PROFILE=your-profile"
        echo -e "   Option 2: export AWS_ACCESS_KEY_ID=xxx AWS_SECRET_ACCESS_KEY=yyy"
        echo -e "   Option 3: aws configure"
        return 1
    fi
}

# Test Rust/Cargo
test_rust() {
    echo -e "\n${CYAN}🦀 Testing Rust/Cargo...${NC}"
    
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Cargo/Rust not installed${NC}"
        echo -e "${YELLOW}💡 Solution: Install Rust from https://rustup.rs/${NC}"
        return 1
    fi
    
    local rust_version=$(rustc --version)
    local cargo_version=$(cargo --version)
    
    echo -e "${GREEN}✅ Rust/Cargo is available${NC}"
    echo -e "   $rust_version"
    echo -e "   $cargo_version"
    
    # Test compilation
    echo -e "${BLUE}📦 Testing compilation...${NC}"
    cd "$SCRIPT_DIR"
    if cargo check --bin telemetry_demo &> /dev/null; then
        echo -e "${GREEN}✅ Demo compiles successfully${NC}"
        return 0
    else
        echo -e "${RED}❌ Compilation failed${NC}"
        echo -e "${YELLOW}💡 Solution: Check dependencies and feature flags${NC}"
        echo -e "   Try: cargo clean && cargo check --bin telemetry_demo"
        return 1
    fi
}

# Test telemetry stack services
test_telemetry_services() {
    echo -e "\n${CYAN}📊 Testing telemetry services...${NC}"
    
    local services=(
        "OpenTelemetry Collector:http://localhost:13133"
        "Prometheus:http://localhost:9090/-/ready"
        "Grafana:http://localhost:3000/api/health"
        "Jaeger:http://localhost:16686"
    )
    
    local failed_services=()
    
    for service_info in "${services[@]}"; do
        local name="${service_info%:*}"
        local url="${service_info#*:}"
        
        if curl -s "$url" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ $name is responding${NC}"
        else
            echo -e "${RED}❌ $name is not responding${NC}"
            failed_services+=("$name")
        fi
    done
    
    if [ ${#failed_services[@]} -eq 0 ]; then
        echo -e "${GREEN}✅ All telemetry services are healthy${NC}"
        return 0
    else
        echo -e "\n${RED}❌ Some services are not responding${NC}"
        echo -e "${YELLOW}💡 Solutions:${NC}"
        echo -e "   1. Start the stack: ./setup-telemetry.sh start"
        echo -e "   2. Check service logs: docker logs stood-otel-collector"
        echo -e "   3. Restart services: ./setup-telemetry.sh restart"
        return 1
    fi
}

# Test telemetry data flow
test_telemetry_data_flow() {
    echo -e "\n${CYAN}📈 Testing telemetry data flow...${NC}"
    
    # Check if OpenTelemetry collector is receiving data
    local otel_metrics_url="http://localhost:8889/metrics"
    if curl -s "$otel_metrics_url" | grep -q "otelcol_receiver"; then
        echo -e "${GREEN}✅ OpenTelemetry Collector is receiving data${NC}"
    else
        echo -e "${YELLOW}⚠️  OpenTelemetry Collector metrics not found${NC}"
        echo -e "${YELLOW}💡 This is normal if no demo has been run yet${NC}"
    fi
    
    # Check if Prometheus is scraping data
    local prometheus_targets_url="http://localhost:9090/api/v1/targets"
    if curl -s "$prometheus_targets_url" | grep -q "\"health\":\"up\""; then
        echo -e "${GREEN}✅ Prometheus is scraping targets${NC}"
    else
        echo -e "${YELLOW}⚠️  Prometheus targets may be down${NC}"
        echo -e "${YELLOW}💡 Check: http://localhost:9090/targets${NC}"
    fi
    
    return 0
}

# Show system information
show_system_info() {
    echo -e "\n${CYAN}💻 System Information${NC}"
    echo -e "${CYAN}====================${NC}"
    
    echo -e "${BLUE}Operating System:${NC}"
    uname -a
    
    echo -e "\n${BLUE}Memory Usage:${NC}"
    free -h 2>/dev/null || vm_stat | head -5
    
    echo -e "\n${BLUE}Disk Space:${NC}"
    df -h . 2>/dev/null || diskutil info / | grep "Free Space"
    
    echo -e "\n${BLUE}Docker Info:${NC}"
    if command -v docker &> /dev/null && docker info &> /dev/null; then
        docker info | grep -E "(Server Version|Total Memory|CPUs|Operating System)" || echo "Docker info not available"
    else
        echo "Docker not available"
    fi
    
    echo -e "\n${BLUE}Environment Variables:${NC}"
    echo "AWS_REGION=${AWS_REGION:-"(not set)"}"
    echo "AWS_PROFILE=${AWS_PROFILE:-"(not set)"}"
    echo "OTEL_ENABLED=${OTEL_ENABLED:-"(not set)"}"
    echo "RUST_LOG=${RUST_LOG:-"(not set)"}"
}

# Show Docker logs
show_docker_logs() {
    echo -e "\n${CYAN}📋 Docker Container Logs${NC}"
    echo -e "${CYAN}=========================${NC}"
    
    cd "$SCRIPT_DIR"
    
    local containers=("stood-otel-collector" "stood-prometheus" "stood-grafana" "stood-jaeger")
    
    for container in "${containers[@]}"; do
        echo -e "\n${BLUE}--- $container ---${NC}"
        if docker ps | grep -q "$container"; then
            docker logs --tail 10 "$container" 2>/dev/null || echo "No logs available"
        else
            echo "Container not running"
        fi
    done
}

# Generate diagnostic report
generate_diagnostic_report() {
    echo -e "\n${CYAN}📄 Generating diagnostic report...${NC}"
    
    local report_file="$SCRIPT_DIR/diagnostic-report.txt"
    
    {
        echo "Stood Telemetry Demo Diagnostic Report"
        echo "Generated: $(date)"
        echo "========================================"
        echo
        
        echo "SYSTEM INFORMATION"
        echo "=================="
        uname -a
        echo
        
        echo "DOCKER STATUS"
        echo "============="
        docker --version 2>/dev/null || echo "Docker not available"
        docker info 2>/dev/null | head -10 || echo "Docker daemon not running"
        echo
        
        echo "RUNNING CONTAINERS"
        echo "=================="
        docker ps 2>/dev/null || echo "Cannot list containers"
        echo
        
        echo "PORT USAGE"
        echo "=========="
        for port in 3000 4319 4320 9090 9100 13133 16686; do
            if lsof -i :$port 2>/dev/null; then
                echo "Port $port: In use"
            else
                echo "Port $port: Available"
            fi
        done
        echo
        
        echo "ENVIRONMENT VARIABLES"
        echo "===================="
        env | grep -E "(AWS_|OTEL_|RUST_|STOOD_)" || echo "No relevant environment variables set"
        echo
        
        echo "RECENT DOCKER LOGS"
        echo "=================="
        for container in stood-otel-collector stood-prometheus stood-grafana stood-jaeger; do
            echo "--- $container ---"
            docker logs --tail 5 "$container" 2>/dev/null || echo "Container not found or not running"
            echo
        done
        
    } > "$report_file"
    
    echo -e "${GREEN}✅ Diagnostic report saved to: $report_file${NC}"
}

# Main diagnostic function
run_full_diagnosis() {
    echo -e "${BLUE}🔍 Running full system diagnosis...${NC}"
    
    local tests=(
        "test_docker"
        "test_ports" 
        "test_rust"
        "test_aws_credentials"
        "test_telemetry_services"
        "test_telemetry_data_flow"
    )
    
    local passed=0
    local total=${#tests[@]}
    
    for test in "${tests[@]}"; do
        if $test; then
            ((passed++))
        fi
    done
    
    echo -e "\n${BLUE}📊 Diagnosis Summary${NC}"
    echo -e "${BLUE}===================${NC}"
    echo -e "Tests passed: ${GREEN}$passed${NC}/$total"
    
    if [ $passed -eq $total ]; then
        echo -e "${GREEN}🎉 All tests passed! Your system is ready for the telemetry demo.${NC}"
        echo -e "${CYAN}👉 Run: ./run-demo.sh${NC}"
    else
        echo -e "${YELLOW}⚠️  Some issues detected. Review the solutions above.${NC}"
        echo -e "${CYAN}👉 For detailed logs: ./troubleshoot.sh logs${NC}"
    fi
}

# Show help
show_help() {
    echo -e "${BLUE}Stood Telemetry Demo Troubleshooting${NC}"
    echo -e "${BLUE}====================================${NC}"
    echo ""
    echo -e "${YELLOW}Usage:${NC}"
    echo -e "  $0 [command]"
    echo ""
    echo -e "${YELLOW}Commands:${NC}"
    echo -e "  ${GREEN}diagnose${NC}    Run full system diagnosis (default)"
    echo -e "  ${GREEN}docker${NC}      Test Docker setup"
    echo -e "  ${GREEN}ports${NC}       Check port availability"
    echo -e "  ${GREEN}aws${NC}         Test AWS credentials"
    echo -e "  ${GREEN}rust${NC}        Test Rust/Cargo setup"
    echo -e "  ${GREEN}services${NC}    Test telemetry services"
    echo -e "  ${GREEN}logs${NC}        Show Docker container logs"
    echo -e "  ${GREEN}sysinfo${NC}     Show system information"
    echo -e "  ${GREEN}report${NC}      Generate diagnostic report"
    echo -e "  ${GREEN}help${NC}        Show this help message"
    echo ""
    echo -e "${YELLOW}Common Issues:${NC}"
    echo -e "  • Docker not running: Start Docker daemon"
    echo -e "  • Ports in use: ./setup-telemetry.sh stop"
    echo -e "  • AWS credentials: export AWS_PROFILE=your-profile"
    echo -e "  • Services not responding: ./setup-telemetry.sh restart"
}

# Main execution
main() {
    case "${1:-diagnose}" in
        diagnose|full)
            run_full_diagnosis
            ;;
        docker)
            test_docker
            ;;
        ports)
            test_ports
            ;;
        aws)
            test_aws_credentials
            ;;
        rust)
            test_rust
            ;;
        services)
            test_telemetry_services
            ;;
        logs)
            show_docker_logs
            ;;
        sysinfo)
            show_system_info
            ;;
        report)
            generate_diagnostic_report
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

# Run main function with all arguments
main "$@"