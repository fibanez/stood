#!/bin/bash
# Stood Telemetry Demo Overview and Management Script
# This script provides a comprehensive overview and control interface for the telemetry demo

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

show_banner() {
    echo -e "${PURPLE}"
    cat << 'EOF'
╔═══════════════════════════════════════════════════════════════════════════════╗
║                          🚀 STOOD TELEMETRY DEMO                             ║
║                    Complete Observability Stack Demonstration                 ║
╚═══════════════════════════════════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

show_overview() {
    echo -e "${CYAN}📊 What This Demo Provides:${NC}"
    echo -e "   • ${GREEN}OpenTelemetry Integration${NC} - Industry-standard observability"
    echo -e "   • ${GREEN}Prometheus Metrics${NC} - Agent performance, token usage, tool execution"
    echo -e "   • ${GREEN}Grafana Dashboards${NC} - Beautiful real-time visualizations"
    echo -e "   • ${GREEN}Jaeger Tracing${NC} - Distributed traces across agent → model → tools"
    echo -e "   • ${GREEN}Error Recovery${NC} - ValidationException handling and recovery patterns"
    echo -e "   • ${GREEN}Performance Analysis${NC} - Latency, throughput, and resource utilization"
    echo ""
    echo -e "${CYAN}🎯 Demo Scenarios:${NC}"
    echo -e "   📊 ${YELLOW}Phase 1:${NC} Basic Operations (time, weather, calculations)"
    echo -e "   🧠 ${YELLOW}Phase 2:${NC} Complex Multi-Step Reasoning (financial analysis)"
    echo -e "   🚨 ${YELLOW}Phase 3:${NC} Error Handling & Recovery (validation failures)" 
    echo -e "   ⚡ ${YELLOW}Phase 4:${NC} Performance Stress Testing (concurrent operations)"
}

show_services() {
    echo -e "\n${CYAN}🌐 Telemetry Stack Services:${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    local services=(
        "📈:Prometheus:http://localhost:9090:Metrics collection and querying"
        "📊:Grafana:http://localhost:3000:Visualization dashboards (admin/admin)"
        "🔍:Jaeger:http://localhost:16686:Distributed tracing and request flow"
        "🔧:OpenTelemetry:http://localhost:13133:Collector health and status"
        "📋:Node Exporter:http://localhost:9100:System metrics collection"
        "📝:Loki:http://localhost:3100:Log aggregation service"
    )
    
    for service_info in "${services[@]}"; do
        IFS=':' read -r icon name url description <<< "$service_info"
        local status=""
        if curl -s "$url" > /dev/null 2>&1; then
            status="${GREEN}●${NC}"
        else
            status="${RED}●${NC}"
        fi
        echo -e "   $status $icon ${BLUE}$name${NC}: ${CYAN}$url${NC}"
        echo -e "      $description"
    done
}

show_quick_commands() {
    echo -e "\n${CYAN}⚡ Quick Commands:${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "   ${GREEN}🚀 Complete Setup:${NC}        ./demo-overview.sh setup"
    echo -e "   ${GREEN}🎯 Run Demo:${NC}               ./run-demo.sh"
    echo -e "   ${GREEN}📊 Start Stack:${NC}            ./setup-telemetry.sh"
    echo -e "   ${GREEN}🛑 Stop Stack:${NC}             ./setup-telemetry.sh stop"
    echo -e "   ${GREEN}🔍 Troubleshoot:${NC}           ./troubleshoot.sh"
    echo -e "   ${GREEN}📋 Check Status:${NC}           ./demo-overview.sh status"
    echo -e "   ${GREEN}🧹 Cleanup:${NC}                ./demo-overview.sh cleanup"
}

show_key_metrics() {
    echo -e "\n${CYAN}📈 Key Metrics to Watch:${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "   ${YELLOW}Agent Performance:${NC}"
    echo -e "   • rate(stood_agent_cycles_total[5m]) - Cycles per second"
    echo -e "   • histogram_quantile(0.95, rate(stood_agent_cycle_duration_seconds_bucket[5m])) - P95 latency"
    echo -e ""
    echo -e "   ${YELLOW}Token Economics:${NC}"
    echo -e "   • rate(stood_model_tokens_input_total[5m]) - Input tokens/sec"
    echo -e "   • rate(stood_model_tokens_output_total[5m]) - Output tokens/sec"
    echo -e ""
    echo -e "   ${YELLOW}Tool Execution:${NC}"
    echo -e "   • rate(stood_tool_executions_total{status=\"success\"}[5m]) - Successful tools/sec"
    echo -e "   • stood_tool_execution_duration_seconds - Tool latency"
    echo -e ""
    echo -e "   ${YELLOW}Error Monitoring:${NC}"
    echo -e "   • rate(stood_agent_errors_total[5m]) - Error rate"
    echo -e "   • rate(stood_validation_exceptions_total[5m]) - Context overflow events"
}

check_status() {
    echo -e "${CYAN}📋 System Status Check${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━${NC}"
    
    # Check Docker
    if command -v docker &> /dev/null && docker info &> /dev/null; then
        echo -e "${GREEN}✅ Docker:${NC} Running"
    else
        echo -e "${RED}❌ Docker:${NC} Not available"
    fi
    
    # Check AWS credentials
    if [[ -n "$AWS_PROFILE" ]] || [[ -n "$AWS_ACCESS_KEY_ID" ]]; then
        echo -e "${GREEN}✅ AWS Credentials:${NC} Configured"
    else
        echo -e "${YELLOW}⚠️  AWS Credentials:${NC} Not detected"
    fi
    
    # Check Rust
    if command -v cargo &> /dev/null; then
        echo -e "${GREEN}✅ Rust/Cargo:${NC} Available"
    else
        echo -e "${RED}❌ Rust/Cargo:${NC} Not installed"
    fi
    
    # Check services
    echo -e "\n${CYAN}Service Status:${NC}"
    local services=(
        "OpenTelemetry:http://localhost:13133"
        "Prometheus:http://localhost:9090/-/ready"
        "Grafana:http://localhost:3000/api/health"
        "Jaeger:http://localhost:16686"
    )
    
    for service_info in "${services[@]}"; do
        local name="${service_info%:*}"
        local url="${service_info#*:}"
        
        if curl -s "$url" > /dev/null 2>&1; then
            echo -e "   ${GREEN}✅ $name:${NC} Healthy"
        else
            echo -e "   ${RED}❌ $name:${NC} Not responding"
        fi
    done
}

run_complete_setup() {
    echo -e "${YELLOW}🚀 Running Complete Telemetry Demo Setup${NC}"
    echo -e "${YELLOW}=========================================${NC}"
    
    echo -e "\n${CYAN}Step 1: Starting telemetry stack...${NC}"
    if ./setup-telemetry.sh; then
        echo -e "${GREEN}✅ Telemetry stack started successfully${NC}"
    else
        echo -e "${RED}❌ Failed to start telemetry stack${NC}"
        exit 1
    fi
    
    echo -e "\n${CYAN}Step 2: Running system diagnosis...${NC}"
    if ./troubleshoot.sh diagnose; then
        echo -e "${GREEN}✅ System diagnosis completed${NC}"
    else
        echo -e "${YELLOW}⚠️  Some issues detected, but continuing...${NC}"
    fi
    
    echo -e "\n${GREEN}🎉 Setup Complete!${NC}"
    echo -e "${CYAN}👉 Next step: Run the demo with: ${YELLOW}./run-demo.sh${NC}"
    show_services
}

open_dashboards() {
    echo -e "${CYAN}📊 Opening monitoring dashboards...${NC}"
    
    local urls=(
        "http://localhost:9090"
        "http://localhost:3000"
        "http://localhost:16686"
    )
    
    for url in "${urls[@]}"; do
        if command -v xdg-open &> /dev/null; then
            xdg-open "$url" 2>/dev/null &
        elif command -v open &> /dev/null; then
            open "$url" 2>/dev/null &
        else
            echo -e "${YELLOW}Please open manually: $url${NC}"
        fi
    done
    
    echo -e "${GREEN}✅ Dashboard URLs opened in browser${NC}"
}

cleanup_demo() {
    echo -e "${YELLOW}🧹 Cleaning up telemetry demo...${NC}"
    
    # Stop containers
    echo -e "${CYAN}Stopping telemetry stack...${NC}"
    ./setup-telemetry.sh stop
    
    # Remove Docker volumes
    echo -e "${CYAN}Removing Docker volumes...${NC}"
    docker volume rm stood-telemetry_prometheus_data stood-telemetry_grafana_data 2>/dev/null || true
    
    # Clean up generated files
    echo -e "${CYAN}Cleaning generated files...${NC}"
    rm -rf prometheus/ grafana/ otel-collector/ logs/ target/
    rm -f docker-compose.yml diagnostic-report.txt
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

show_help() {
    show_banner
    show_overview
    
    echo -e "\n${CYAN}💡 Usage:${NC}"
    echo -e "   $0 [command]"
    
    echo -e "\n${CYAN}📋 Commands:${NC}"
    echo -e "   ${GREEN}overview${NC}     Show this complete overview (default)"
    echo -e "   ${GREEN}setup${NC}        Run complete setup (stack + diagnosis)"
    echo -e "   ${GREEN}status${NC}       Check system and service status"
    echo -e "   ${GREEN}services${NC}     Show service information and URLs"
    echo -e "   ${GREEN}metrics${NC}      Show key metrics to monitor"
    echo -e "   ${GREEN}open${NC}         Open monitoring dashboards in browser"
    echo -e "   ${GREEN}cleanup${NC}      Complete cleanup (stops everything)"
    echo -e "   ${GREEN}help${NC}         Show this help message"
    
    echo -e "\n${CYAN}🎯 Quick Start Workflow:${NC}"
    echo -e "   1. ${YELLOW}./demo-overview.sh setup${NC}     - Complete setup"
    echo -e "   2. ${YELLOW}./run-demo.sh${NC}                - Run the demo"
    echo -e "   3. ${YELLOW}./demo-overview.sh open${NC}      - Open dashboards"
    echo -e "   4. ${YELLOW}./demo-overview.sh cleanup${NC}   - Clean up when done"
    
    show_quick_commands
    show_services
    show_key_metrics
    
    echo -e "\n${PURPLE}📚 Documentation:${NC}"
    echo -e "   • ${CYAN}README.md${NC}     - Complete documentation"
    echo -e "   • ${CYAN}QUICKSTART.md${NC} - 2-minute quick start guide"
    echo -e "   • ${CYAN}troubleshoot.sh${NC} - Diagnostic and troubleshooting tools"
    
    echo -e "\n${GREEN}🎉 Ready to explore the power of Stood's telemetry system!${NC}"
}

# Main execution
main() {
    case "${1:-overview}" in
        overview|help|--help|-h)
            show_help
            ;;
        setup)
            run_complete_setup
            ;;
        status)
            check_status
            ;;
        services)
            show_services
            ;;
        metrics)
            show_key_metrics
            ;;
        open)
            open_dashboards
            ;;
        cleanup)
            cleanup_demo
            ;;
        *)
            echo -e "${RED}❌ Unknown command: $1${NC}"
            echo -e "${YELLOW}💡 Use: $0 help${NC}"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"