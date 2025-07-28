#!/bin/bash
# Stood Telemetry Demo Setup Script
# This script deploys a complete observability stack with Prometheus, Grafana, and Jaeger

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
GRAFANA_CONFIG_DIR="$SCRIPT_DIR/grafana"
PROMETHEUS_CONFIG_DIR="$SCRIPT_DIR/prometheus"

echo -e "${BLUE}🚀 Stood Telemetry Demo Setup${NC}"
echo -e "${BLUE}=================================${NC}"

# Check prerequisites
check_prerequisites() {
    echo -e "${CYAN}📋 Checking prerequisites...${NC}"
    
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}❌ Docker is not installed. Please install Docker first.${NC}"
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        echo -e "${RED}❌ Docker Compose is not available. Please install Docker Compose.${NC}"
        exit 1
    fi
    
    if ! docker info &> /dev/null; then
        echo -e "${RED}❌ Docker daemon is not running. Please start Docker.${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Prerequisites check passed${NC}"
}

# Create directory structure
create_directories() {
    echo -e "${CYAN}📁 Creating configuration directories...${NC}"
    
    mkdir -p "$PROMETHEUS_CONFIG_DIR"
    mkdir -p "$GRAFANA_CONFIG_DIR/dashboards"
    mkdir -p "$GRAFANA_CONFIG_DIR/provisioning/dashboards"
    mkdir -p "$GRAFANA_CONFIG_DIR/provisioning/datasources"
    mkdir -p "$SCRIPT_DIR/otel-collector"
    mkdir -p "$SCRIPT_DIR/logs"
    
    echo -e "${GREEN}✅ Directories created${NC}"
}

# Create Prometheus configuration
create_prometheus_config() {
    echo -e "${CYAN}⚙️ Creating Prometheus configuration...${NC}"
    
    cat > "$PROMETHEUS_CONFIG_DIR/prometheus.yml" << 'EOF'
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  # - "first_rules.yml"
  # - "second_rules.yml"

scrape_configs:
  # Prometheus self-monitoring
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  # OpenTelemetry Collector metrics
  - job_name: 'otel-collector'
    static_configs:
      - targets: ['otel-collector:8889']
    scrape_interval: 10s
    metrics_path: '/metrics'

  # Stood application metrics (if exposed directly)
  - job_name: 'stood-demo'
    static_configs:
      - targets: ['host.docker.internal:8080']
    scrape_interval: 15s
    metrics_path: '/metrics'
    scrape_timeout: 10s

  # Node exporter for system metrics
  - job_name: 'node-exporter'
    static_configs:
      - targets: ['node-exporter:9100']

  # Additional scrape config for OTLP metrics
  - job_name: 'otlp-metrics'
    static_configs:
      - targets: ['otel-collector:8888']
    metrics_path: '/metrics'
EOF

    echo -e "${GREEN}✅ Prometheus configuration created${NC}"
}

# Create OpenTelemetry Collector configuration
create_otel_config() {
    echo -e "${CYAN}⚙️ Creating OpenTelemetry Collector configuration...${NC}"
    
    cat > "$SCRIPT_DIR/otel-collector/otel-collector-config.yaml" << 'EOF'
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s
    send_batch_size: 1024
    send_batch_max_size: 2048
  
  memory_limiter:
    limit_mib: 256
    check_interval: 1s
  
  # Add resource attributes
  resource:
    attributes:
      - key: deployment.environment
        value: demo
        action: upsert
      - key: service.namespace
        value: stood
        action: upsert

exporters:
  # Prometheus metrics exporter
  prometheus:
    endpoint: "0.0.0.0:8889"
    namespace: stood
    const_labels:
      environment: demo
    send_timestamps: true
    metric_expiration: 180m
    enable_open_metrics: true
  
  # OTLP exporter for Jaeger traces
  otlp/jaeger:
    endpoint: jaeger:4317
    tls:
      insecure: true
  
  # Debug exporter for debugging
  debug:
    verbosity: detailed
    sampling_initial: 5
    sampling_thereafter: 200


extensions:
  health_check:
    endpoint: 0.0.0.0:13133
  pprof:
    endpoint: 0.0.0.0:1777
  zpages:
    endpoint: 0.0.0.0:55679

service:
  extensions: [health_check, pprof, zpages]
  pipelines:
    traces:
      receivers: [otlp]
      processors: [memory_limiter, batch, resource]
      exporters: [otlp/jaeger, debug]
    
    metrics:
      receivers: [otlp]
      processors: [memory_limiter, batch, resource]
      exporters: [prometheus, debug]
    
    logs:
      receivers: [otlp]
      processors: [memory_limiter, batch, resource]
      exporters: [debug]
  
  telemetry:
    logs:
      level: "debug"
EOF

    echo -e "${GREEN}✅ OpenTelemetry Collector configuration created${NC}"
}

# Create Grafana datasource configuration
create_grafana_datasources() {
    echo -e "${CYAN}⚙️ Creating Grafana datasources...${NC}"
    
    cat > "$GRAFANA_CONFIG_DIR/provisioning/datasources/datasources.yml" << 'EOF'
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: true
    jsonData:
      httpMethod: POST
      manageAlerts: true
      prometheusType: Prometheus
      prometheusVersion: 2.40.0
      cacheLevel: 'High'
      disableRecordingRules: false
      incrementalQueryOverlapWindow: 10m

  - name: Jaeger
    type: jaeger
    access: proxy
    url: http://jaeger:16686
    editable: true
    jsonData:
      tracesToLogs:
        datasourceUid: 'loki'
        tags: ['job', 'instance', 'pod', 'namespace']
        mappedTags: [{ key: 'service.name', value: 'service' }]
        mapTagNamesEnabled: false
        spanStartTimeShift: '-1h'
        spanEndTimeShift: '1h'
      tracesToMetrics:
        datasourceUid: 'prometheus'
        tags: [{ key: 'service.name', value: 'service' }, { key: 'job' }]
        queries:
          - name: 'Sample query'
            query: 'sum(rate(traces_spanmetrics_latency_bucket{$$__tags}[5m]))'
      serviceMap:
        datasourceUid: 'prometheus'
      nodeGraph:
        enabled: true

  - name: Loki
    type: loki
    access: proxy
    url: http://loki:3100
    editable: true
    jsonData:
      maxLines: 1000
      derivedFields:
        - datasourceUid: jaeger
          matcherRegex: "trace_id=(\\w+)"
          name: TraceID
          url: '$${__value.raw}'
EOF

    echo -e "${GREEN}✅ Grafana datasources created${NC}"
}

# Create Grafana dashboard provisioning
create_grafana_dashboards_config() {
    echo -e "${CYAN}⚙️ Creating Grafana dashboard provisioning...${NC}"
    
    cat > "$GRAFANA_CONFIG_DIR/provisioning/dashboards/dashboards.yml" << 'EOF'
apiVersion: 1

providers:
  - name: 'stood-dashboards'
    orgId: 1
    folder: 'Stood Telemetry'
    type: file
    disableDeletion: false
    updateIntervalSeconds: 10
    allowUiUpdates: true
    options:
      path: /etc/grafana/provisioning/dashboards
EOF

    echo -e "${GREEN}✅ Grafana dashboard provisioning created${NC}"
}

# Create Stood Telemetry Dashboard
create_stood_dashboard() {
    echo -e "${CYAN}📊 Creating Stood Telemetry Dashboard...${NC}"
    
    cat > "$GRAFANA_CONFIG_DIR/dashboards/stood-telemetry.json" << 'EOF'
{
  "annotations": {
    "list": [
      {
        "builtIn": 1,
        "datasource": {
          "type": "grafana",
          "uid": "-- Grafana --"
        },
        "enable": true,
        "hide": true,
        "iconColor": "rgba(0, 211, 255, 1)",
        "name": "Annotations & Alerts",
        "type": "dashboard"
      }
    ]
  },
  "editable": true,
  "fiscalYearStartMonth": 0,
  "graphTooltip": 0,
  "id": null,
  "links": [],
  "liveNow": false,
  "panels": [
    {
      "datasource": {
        "type": "prometheus",
        "uid": "prometheus"
      },
      "fieldConfig": {
        "defaults": {
          "color": {
            "mode": "palette-classic"
          },
          "custom": {
            "axisCenteredZero": false,
            "axisColorMode": "text",
            "axisLabel": "",
            "axisPlacement": "auto",
            "barAlignment": 0,
            "drawStyle": "line",
            "fillOpacity": 0,
            "gradientMode": "none",
            "hideFrom": {
              "legend": false,
              "tooltip": false,
              "vis": false
            },
            "lineInterpolation": "linear",
            "lineWidth": 1,
            "pointSize": 5,
            "scaleDistribution": {
              "type": "linear"
            },
            "showPoints": "auto",
            "spanNulls": false,
            "stacking": {
              "group": "A",
              "mode": "none"
            },
            "thresholdsStyle": {
              "mode": "off"
            }
          },
          "mappings": [],
          "thresholds": {
            "mode": "absolute",
            "steps": [
              {
                "color": "green",
                "value": null
              },
              {
                "color": "red",
                "value": 80
              }
            ]
          }
        },
        "overrides": []
      },
      "gridPos": {
        "h": 8,
        "w": 12,
        "x": 0,
        "y": 0
      },
      "id": 1,
      "options": {
        "legend": {
          "calcs": [],
          "displayMode": "list",
          "placement": "bottom",
          "showLegend": true
        },
        "tooltip": {
          "mode": "single",
          "sort": "none"
        }
      },
      "targets": [
        {
          "datasource": {
            "type": "prometheus",
            "uid": "prometheus"
          },
          "editorMode": "code",
          "expr": "rate(stood_agent_cycles_total[5m])",
          "instant": false,
          "legendFormat": "Agent Cycles/sec",
          "range": true,
          "refId": "A"
        }
      ],
      "title": "Agent Cycle Rate",
      "type": "timeseries"
    },
    {
      "datasource": {
        "type": "prometheus",
        "uid": "prometheus"
      },
      "fieldConfig": {
        "defaults": {
          "color": {
            "mode": "thresholds"
          },
          "mappings": [],
          "thresholds": {
            "mode": "absolute",
            "steps": [
              {
                "color": "green",
                "value": null
              },
              {
                "color": "red",
                "value": 80
              }
            ]
          }
        },
        "overrides": []
      },
      "gridPos": {
        "h": 8,
        "w": 12,
        "x": 12,
        "y": 0
      },
      "id": 2,
      "options": {
        "orientation": "auto",
        "reduceOptions": {
          "values": false,
          "calcs": [
            "lastNotNull"
          ],
          "fields": ""
        },
        "showThresholdLabels": false,
        "showThresholdMarkers": true
      },
      "pluginVersion": "10.0.0",
      "targets": [
        {
          "datasource": {
            "type": "prometheus",
            "uid": "prometheus"
          },
          "editorMode": "code",
          "expr": "histogram_quantile(0.95, rate(stood_model_request_duration_seconds_bucket[5m]))",
          "instant": false,
          "legendFormat": "P95 Latency",
          "range": true,
          "refId": "A"
        }
      ],
      "title": "P95 Model Request Latency",
      "type": "gauge"
    },
    {
      "datasource": {
        "type": "prometheus",
        "uid": "prometheus"
      },
      "fieldConfig": {
        "defaults": {
          "color": {
            "mode": "palette-classic"
          },
          "custom": {
            "axisCenteredZero": false,
            "axisColorMode": "text",
            "axisLabel": "",
            "axisPlacement": "auto",
            "barAlignment": 0,
            "drawStyle": "line",
            "fillOpacity": 0,
            "gradientMode": "none",
            "hideFrom": {
              "legend": false,
              "tooltip": false,
              "vis": false
            },
            "lineInterpolation": "linear",
            "lineWidth": 1,
            "pointSize": 5,
            "scaleDistribution": {
              "type": "linear"
            },
            "showPoints": "auto",
            "spanNulls": false,
            "stacking": {
              "group": "A",
              "mode": "none"
            },
            "thresholdsStyle": {
              "mode": "off"
            }
          },
          "mappings": [],
          "thresholds": {
            "mode": "absolute",
            "steps": [
              {
                "color": "green",
                "value": null
              },
              {
                "color": "red",
                "value": 80
              }
            ]
          }
        },
        "overrides": []
      },
      "gridPos": {
        "h": 8,
        "w": 12,
        "x": 0,
        "y": 8
      },
      "id": 3,
      "options": {
        "legend": {
          "calcs": [],
          "displayMode": "list",
          "placement": "bottom",
          "showLegend": true
        },
        "tooltip": {
          "mode": "single",
          "sort": "none"
        }
      },
      "targets": [
        {
          "datasource": {
            "type": "prometheus",
            "uid": "prometheus"
          },
          "editorMode": "code",
          "expr": "rate(stood_model_tokens_input_total[5m])",
          "instant": false,
          "legendFormat": "Input Tokens/sec",
          "range": true,
          "refId": "A"
        },
        {
          "datasource": {
            "type": "prometheus",
            "uid": "prometheus"
          },
          "editorMode": "code",
          "expr": "rate(stood_model_tokens_output_total[5m])",
          "instant": false,
          "legendFormat": "Output Tokens/sec",
          "range": true,
          "refId": "B"
        }
      ],
      "title": "Token Consumption Rate",
      "type": "timeseries"
    },
    {
      "datasource": {
        "type": "prometheus",
        "uid": "prometheus"
      },
      "fieldConfig": {
        "defaults": {
          "color": {
            "mode": "palette-classic"
          },
          "custom": {
            "axisCenteredZero": false,
            "axisColorMode": "text",
            "axisLabel": "",
            "axisPlacement": "auto",
            "barAlignment": 0,
            "drawStyle": "line",
            "fillOpacity": 0,
            "gradientMode": "none",
            "hideFrom": {
              "legend": false,
              "tooltip": false,
              "vis": false
            },
            "lineInterpolation": "linear",
            "lineWidth": 1,
            "pointSize": 5,
            "scaleDistribution": {
              "type": "linear"
            },
            "showPoints": "auto",
            "spanNulls": false,
            "stacking": {
              "group": "A",
              "mode": "none"
            },
            "thresholdsStyle": {
              "mode": "off"
            }
          },
          "mappings": [],
          "thresholds": {
            "mode": "absolute",
            "steps": [
              {
                "color": "green",
                "value": null
              },
              {
                "color": "red",
                "value": 80
              }
            ]
          }
        },
        "overrides": []
      },
      "gridPos": {
        "h": 8,
        "w": 12,
        "x": 12,
        "y": 8
      },
      "id": 4,
      "options": {
        "legend": {
          "calcs": [],
          "displayMode": "list",
          "placement": "bottom",
          "showLegend": true
        },
        "tooltip": {
          "mode": "single",
          "sort": "none"
        }
      },
      "targets": [
        {
          "datasource": {
            "type": "prometheus",
            "uid": "prometheus"
          },
          "editorMode": "code",
          "expr": "rate(stood_tool_executions_total{status=\"success\"}[5m])",
          "instant": false,
          "legendFormat": "Successful Tool Executions/sec",
          "range": true,
          "refId": "A"
        },
        {
          "datasource": {
            "type": "prometheus",
            "uid": "prometheus"
          },
          "editorMode": "code",
          "expr": "rate(stood_tool_executions_total{status=\"error\"}[5m])",
          "instant": false,
          "legendFormat": "Failed Tool Executions/sec",
          "range": true,
          "refId": "B"
        }
      ],
      "title": "Tool Execution Rate",
      "type": "timeseries"
    }
  ],
  "refresh": "5s",
  "schemaVersion": 38,
  "style": "dark",
  "tags": ["stood", "telemetry", "observability"],
  "templating": {
    "list": []
  },
  "time": {
    "from": "now-15m",
    "to": "now"
  },
  "timepicker": {},
  "timezone": "",
  "title": "Stood Agent Telemetry",
  "uid": "stood-telemetry",
  "version": 1,
  "weekStart": ""
}
EOF

    echo -e "${GREEN}✅ Stood Telemetry Dashboard created${NC}"
}

# Create Docker Compose configuration
create_docker_compose() {
    echo -e "${CYAN}🐳 Creating Docker Compose configuration...${NC}"
    
    cat > "$DOCKER_COMPOSE_FILE" << 'EOF'
services:
  # OpenTelemetry Collector
  otel-collector:
    image: otel/opentelemetry-collector-contrib:latest
    container_name: stood-otel-collector
    command: ["--config=/etc/otel-collector-config.yaml"]
    volumes:
      - ./otel-collector/otel-collector-config.yaml:/etc/otel-collector-config.yaml
    ports:
      - "4319:4317"   # OTLP gRPC receiver
      - "4320:4318"   # OTLP HTTP receiver
      - "8889:8889"   # Prometheus metrics export
      - "8888:8888"   # Prometheus metrics endpoint
      - "13133:13133" # Health check
      - "1777:1777"   # pprof
      - "55679:55679" # zpages
    networks:
      - stood-telemetry

  # Prometheus
  prometheus:
    image: prom/prometheus:latest
    container_name: stood-prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=200h'
      - '--web.enable-lifecycle'
      - '--web.enable-admin-api'
    volumes:
      - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    ports:
      - "9090:9090"
    networks:
      - stood-telemetry
    restart: unless-stopped

  # Grafana
  grafana:
    image: grafana/grafana:latest
    container_name: stood-grafana
    environment:
      - GF_SECURITY_ADMIN_USER=admin
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false
      - GF_INSTALL_PLUGINS=grafana-piechart-panel
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
    ports:
      - "3000:3000"
    networks:
      - stood-telemetry
    restart: unless-stopped

  # Jaeger
  jaeger:
    image: jaegertracing/all-in-one:latest
    container_name: stood-jaeger
    environment:
      - COLLECTOR_OTLP_ENABLED=true
    ports:
      - "16686:16686" # Jaeger UI
      - "14250:14250" # Jaeger gRPC receiver
    networks:
      - stood-telemetry
    restart: unless-stopped

  # Node Exporter for system metrics
  node-exporter:
    image: prom/node-exporter:latest
    container_name: stood-node-exporter
    command:
      - '--path.procfs=/host/proc'
      - '--path.rootfs=/rootfs'
      - '--path.sysfs=/host/sys'
      - '--collector.filesystem.mount-points-exclude=^/(sys|proc|dev|host|etc)($$|/)'
    volumes:
      - /proc:/host/proc:ro
      - /sys:/host/sys:ro
      - /:/rootfs:ro
    ports:
      - "9101:9100"
    networks:
      - stood-telemetry
    restart: unless-stopped

  # Loki for log aggregation
  loki:
    image: grafana/loki:latest
    container_name: stood-loki
    ports:
      - "3100:3100"
    command: -config.file=/etc/loki/local-config.yaml
    networks:
      - stood-telemetry
    restart: unless-stopped

networks:
  stood-telemetry:
    driver: bridge

volumes:
  prometheus_data:
  grafana_data:
EOF

    echo -e "${GREEN}✅ Docker Compose configuration created${NC}"
}

# Start the telemetry stack
start_stack() {
    echo -e "${CYAN}🚀 Starting telemetry stack...${NC}"
    
    cd "$SCRIPT_DIR"
    
    # Use docker compose or docker-compose based on availability
    if docker compose version &> /dev/null; then
        COMPOSE_CMD="docker compose"
    else
        COMPOSE_CMD="docker-compose"
    fi
    
    echo -e "${YELLOW}📦 Pulling latest images...${NC}"
    $COMPOSE_CMD pull
    
    echo -e "${YELLOW}🏗️ Building and starting services...${NC}"
    $COMPOSE_CMD up -d
    
    echo -e "${GREEN}✅ Telemetry stack started successfully!${NC}"
}

# Wait for services to be ready
wait_for_services() {
    echo -e "${CYAN}⏳ Waiting for services to be ready...${NC}"
    
    # Wait for Prometheus
    echo -e "${YELLOW}   Waiting for Prometheus...${NC}"
    while ! curl -s http://localhost:9090/-/ready > /dev/null; do
        sleep 2
        echo -n "."
    done
    echo -e " ${GREEN}✅${NC}"
    
    # Wait for Grafana
    echo -e "${YELLOW}   Waiting for Grafana...${NC}"
    while ! curl -s http://localhost:3000/api/health > /dev/null; do
        sleep 2
        echo -n "."
    done
    echo -e " ${GREEN}✅${NC}"
    
    # Wait for Jaeger
    echo -e "${YELLOW}   Waiting for Jaeger...${NC}"
    while ! curl -s http://localhost:16686/ > /dev/null; do
        sleep 2
        echo -n "."
    done
    echo -e " ${GREEN}✅${NC}"
    
    # Wait for OpenTelemetry Collector
    echo -e "${YELLOW}   Waiting for OpenTelemetry Collector...${NC}"
    while ! curl -s http://localhost:13133/ > /dev/null; do
        sleep 2
        echo -n "."
    done
    echo -e " ${GREEN}✅${NC}"
    
    echo -e "${GREEN}🎉 All services are ready!${NC}"
}

# Display service information
show_service_info() {
    echo -e "\n${PURPLE}🌐 Telemetry Stack Services${NC}"
    echo -e "${PURPLE}=============================${NC}"
    echo -e "${GREEN}📈 Prometheus:${NC}        http://localhost:9090"
    echo -e "${GREEN}📊 Grafana:${NC}           http://localhost:3000 (admin/admin)"
    echo -e "${GREEN}🔍 Jaeger:${NC}            http://localhost:16686"
    echo -e "${GREEN}🔧 OpenTelemetry:${NC}     http://localhost:13133 (health)"
    echo -e "${GREEN}📋 Node Exporter:${NC}     http://localhost:9101/metrics"
    echo -e "${GREEN}📝 Loki:${NC}              http://localhost:3100"
    echo -e "\n${YELLOW}💡 Next Steps:${NC}"
    echo -e "   1. Run the Stood telemetry demo: ${CYAN}cargo run --bin telemetry_demo${NC}"
    echo -e "   2. View metrics in Prometheus"
    echo -e "   3. Explore the Stood Telemetry dashboard in Grafana"
    echo -e "   4. Examine distributed traces in Jaeger"
    echo -e "\n${YELLOW}🔄 To stop the stack:${NC} ${CYAN}./setup-telemetry.sh stop${NC}"
}

# Stop the telemetry stack
stop_stack() {
    echo -e "${CYAN}🛑 Stopping telemetry stack...${NC}"
    
    cd "$SCRIPT_DIR"
    
    if docker compose version &> /dev/null; then
        docker compose down
    else
        docker-compose down
    fi
    
    echo -e "${GREEN}✅ Telemetry stack stopped${NC}"
}

# Show help
show_help() {
    echo -e "${BLUE}Stood Telemetry Demo Setup Script${NC}"
    echo -e "${BLUE}==================================${NC}"
    echo ""
    echo -e "${YELLOW}Usage:${NC}"
    echo -e "  $0 [command]"
    echo ""
    echo -e "${YELLOW}Commands:${NC}"
    echo -e "  ${GREEN}start${NC}    Start the telemetry stack (default)"
    echo -e "  ${GREEN}stop${NC}     Stop the telemetry stack"
    echo -e "  ${GREEN}restart${NC}  Restart the telemetry stack"
    echo -e "  ${GREEN}status${NC}   Show stack status"
    echo -e "  ${GREEN}logs${NC}     Show logs from all services"
    echo -e "  ${GREEN}help${NC}     Show this help message"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo -e "  $0              # Start the stack"
    echo -e "  $0 start        # Start the stack"
    echo -e "  $0 stop         # Stop the stack"
    echo -e "  $0 logs         # View logs"
}

# Show stack status
show_status() {
    echo -e "${CYAN}📋 Telemetry Stack Status${NC}"
    echo -e "${CYAN}=========================${NC}"
    
    cd "$SCRIPT_DIR"
    
    if docker compose version &> /dev/null; then
        docker compose ps
    else
        docker-compose ps
    fi
}

# Show logs
show_logs() {
    echo -e "${CYAN}📋 Telemetry Stack Logs${NC}"
    echo -e "${CYAN}=======================${NC}"
    
    cd "$SCRIPT_DIR"
    
    if docker compose version &> /dev/null; then
        docker compose logs -f
    else
        docker-compose logs -f
    fi
}

# Main execution
main() {
    case "${1:-start}" in
        start)
            check_prerequisites
            create_directories
            create_prometheus_config
            create_otel_config
            create_grafana_datasources
            create_grafana_dashboards_config
            create_stood_dashboard
            create_docker_compose
            start_stack
            wait_for_services
            show_service_info
            ;;
        stop)
            stop_stack
            ;;
        restart)
            stop_stack
            sleep 3
            start_stack
            wait_for_services
            show_service_info
            ;;
        status)
            show_status
            ;;
        logs)
            show_logs
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