#!/bin/bash
# Stood Telemetry Demo Stop Script
# This script stops the complete observability stack

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCKER_COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"

echo -e "${BLUE}🛑 Stood Telemetry Demo Stop${NC}"
echo -e "${BLUE}=============================${NC}"

# Check if Docker Compose file exists
check_compose_file() {
    if [ ! -f "$DOCKER_COMPOSE_FILE" ]; then
        echo -e "${RED}❌ Docker Compose file not found: $DOCKER_COMPOSE_FILE${NC}"
        echo -e "${YELLOW}💡 Run './setup-telemetry.sh' first to create the configuration${NC}"
        exit 1
    fi
}

# Check if services are running
check_services_running() {
    echo -e "${CYAN}🔍 Checking if services are running...${NC}"
    
    cd "$SCRIPT_DIR"
    
    # Determine which compose command to use
    if docker compose version &> /dev/null; then
        COMPOSE_CMD="docker compose"
    else
        COMPOSE_CMD="docker-compose"
    fi
    
    # Check if any containers are running
    RUNNING_CONTAINERS=$($COMPOSE_CMD ps -q 2>/dev/null || true)
    
    if [ -z "$RUNNING_CONTAINERS" ]; then
        echo -e "${YELLOW}ℹ️  No telemetry services are currently running${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✅ Found running telemetry services${NC}"
    return 0
}

# Stop all services
stop_services() {
    echo -e "${CYAN}🛑 Stopping telemetry services...${NC}"
    
    cd "$SCRIPT_DIR"
    
    # Stop and remove containers
    echo -e "${YELLOW}   Stopping containers...${NC}"
    $COMPOSE_CMD down
    
    echo -e "${GREEN}✅ All services stopped${NC}"
}

# Stop specific service
stop_specific_service() {
    local service_name="$1"
    
    echo -e "${CYAN}🛑 Stopping $service_name...${NC}"
    
    cd "$SCRIPT_DIR"
    
    # Stop specific service
    $COMPOSE_CMD stop "$service_name"
    
    echo -e "${GREEN}✅ $service_name stopped${NC}"
}

# Remove containers and volumes
cleanup_all() {
    echo -e "${CYAN}🧹 Cleaning up containers and volumes...${NC}"
    
    cd "$SCRIPT_DIR"
    
    # Stop and remove containers, networks, and volumes
    echo -e "${YELLOW}   Removing containers, networks, and volumes...${NC}"
    $COMPOSE_CMD down -v --remove-orphans
    
    # Remove unused networks and volumes
    echo -e "${YELLOW}   Cleaning up Docker resources...${NC}"
    docker network prune -f 2>/dev/null || true
    docker volume prune -f 2>/dev/null || true
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

# Show running services status
show_status() {
    echo -e "${CYAN}📋 Current Service Status${NC}"
    echo -e "${CYAN}=========================${NC}"
    
    cd "$SCRIPT_DIR"
    
    if docker compose version &> /dev/null; then
        docker compose ps
    else
        docker-compose ps
    fi
}

# Show help
show_help() {
    echo -e "${BLUE}Stood Telemetry Demo Stop Script${NC}"
    echo -e "${BLUE}=================================${NC}"
    echo ""
    echo -e "${YELLOW}Usage:${NC}"
    echo -e "  $0 [command] [service]"
    echo ""
    echo -e "${YELLOW}Commands:${NC}"
    echo -e "  ${GREEN}stop${NC}         Stop all telemetry services (default)"
    echo -e "  ${GREEN}cleanup${NC}      Stop services and remove volumes"
    echo -e "  ${GREEN}status${NC}       Show current service status"
    echo -e "  ${GREEN}service${NC}      Stop a specific service"
    echo -e "  ${GREEN}help${NC}         Show this help message"
    echo ""
    echo -e "${YELLOW}Available Services:${NC}"
    echo -e "  ${GREEN}otel-collector${NC}   OpenTelemetry Collector"
    echo -e "  ${GREEN}prometheus${NC}       Prometheus metrics server"
    echo -e "  ${GREEN}grafana${NC}          Grafana dashboard"
    echo -e "  ${GREEN}jaeger${NC}           Jaeger tracing server"
    echo -e "  ${GREEN}node-exporter${NC}    Node Exporter for system metrics"
    echo -e "  ${GREEN}loki${NC}             Loki log aggregation"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo -e "  $0                    # Stop all services"
    echo -e "  $0 stop              # Stop all services"
    echo -e "  $0 cleanup           # Stop services and remove volumes"
    echo -e "  $0 service grafana   # Stop only Grafana"
    echo -e "  $0 status            # Show service status"
    echo ""
    echo -e "${YELLOW}Notes:${NC}"
    echo -e "  • Use ${CYAN}cleanup${NC} to completely remove all data (metrics, logs, etc.)"
    echo -e "  • Use ${CYAN}stop${NC} to preserve data for next startup"
    echo -e "  • To start services again, run ${CYAN}./setup-telemetry.sh${NC}"
}

# Show completion message
show_completion() {
    echo -e "\n${PURPLE}🎯 Telemetry Services Stopped${NC}"
    echo -e "${PURPLE}==============================${NC}"
    echo -e "${GREEN}✅ All telemetry services have been stopped${NC}"
    echo -e "\n${YELLOW}💡 Next Steps:${NC}"
    echo -e "   • To start services again: ${CYAN}./setup-telemetry.sh${NC}"
    echo -e "   • To view service status: ${CYAN}./stop-telemetry.sh status${NC}"
    echo -e "   • To cleanup all data: ${CYAN}./stop-telemetry.sh cleanup${NC}"
}

# Main execution
main() {
    # Check prerequisites
    check_compose_file
    
    case "${1:-stop}" in
        stop)
            if check_services_running; then
                stop_services
                show_completion
            else
                echo -e "${YELLOW}💡 To start services: ${CYAN}./setup-telemetry.sh${NC}"
            fi
            ;;
        cleanup)
            if check_services_running; then
                cleanup_all
                show_completion
            else
                echo -e "${YELLOW}💡 To start services: ${CYAN}./setup-telemetry.sh${NC}"
            fi
            ;;
        service)
            if [ -z "$2" ]; then
                echo -e "${RED}❌ Service name required${NC}"
                show_help
                exit 1
            fi
            if check_services_running; then
                stop_specific_service "$2"
            else
                echo -e "${YELLOW}💡 To start services: ${CYAN}./setup-telemetry.sh${NC}"
            fi
            ;;
        status)
            show_status
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