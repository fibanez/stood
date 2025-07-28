# 📊 Stood Telemetry Demo - Complete Observability Stack

This comprehensive demonstration showcases the full power of Stood's telemetry system with OpenTelemetry integration, Prometheus metrics, Grafana dashboards, and Jaeger distributed tracing.

## 🚀 What You'll Experience

This demo provides a complete observability stack that automatically tracks:

- **Agent Performance**: Cycle duration, success rates, token consumption
- **Tool Execution**: Tool selection patterns, execution times, success/failure rates  
- **Model Interactions**: Request/response latency, token usage, model parameters
- **Error Handling**: Validation exceptions, recovery attempts, error patterns
- **Distributed Tracing**: Complete request flows across agent → model → tools
- **Custom Metrics**: Business-specific KPIs and performance indicators

## 📋 Prerequisites

### System Requirements
- **Docker & Docker Compose**: For running the monitoring stack
- **Rust**: Latest stable version with Cargo
- **AWS Credentials**: For Bedrock API access
- **8GB RAM**: Recommended for running all services
- **Available Ports**: 3000, 4319, 4320, 9090, 9100, 13133, 16686

### Quick Prerequisites Check
```bash
# Check Docker
docker --version
docker-compose --version  # or: docker compose version

# Check Rust
rustc --version
cargo --version

# Check AWS credentials (should return your configured profile/keys)
aws sts get-caller-identity  # or check environment variables
echo $AWS_PROFILE
echo $AWS_ACCESS_KEY_ID
```

## ⚡ Quick Start (5 Minutes)

### 1. Navigate to Demo Directory
```bash
cd examples/docs/004_telemetry
```

### 2. Start the Telemetry Stack
```bash
./setup-telemetry.sh
```

This automated script will:
- ✅ Check prerequisites
- 🐳 Download and configure Docker images
- ⚙️ Set up Prometheus, Grafana, Jaeger, and OpenTelemetry Collector
- 📊 Create pre-configured dashboards and data sources
- 🌐 Start all services and wait for readiness

### 3. Configure AWS Credentials (if not already done)
```bash
# Option 1: AWS Profile
export AWS_PROFILE=your-profile

# Option 2: Direct credentials
export AWS_ACCESS_KEY_ID=your-access-key
export AWS_SECRET_ACCESS_KEY=your-secret-key
export AWS_REGION=us-west-2

# Option 3: Use existing profile/role (if on EC2/ECS)
```

### 4. Run the Telemetry Demo
```bash
# Run the demo (telemetry is now always enabled)
cargo run --bin telemetry_demo
```

### 5. Access Monitoring Services
Once the demo starts, open these URLs in your browser:

| Service | URL | Credentials |
|---------|-----|-------------|
| 📈 **Prometheus** | http://localhost:9090 | None |
| 📊 **Grafana** | http://localhost:3000 | admin/admin |
| 🔍 **Jaeger** | http://localhost:16686 | None |
| 🔧 **OpenTelemetry** | http://localhost:13133 | None |

## 📊 Understanding the Demo

### Demo Phases
The telemetry demo runs through four comprehensive phases:

#### 📊 **Phase 1: Basic Operations**
- Current time queries
- Weather information retrieval  
- Mathematical calculations
- Simple tool interactions

**What to Watch**: 
- Agent cycle metrics in Grafana
- Tool execution traces in Jaeger
- Request rates in Prometheus

#### 🧠 **Phase 2: Complex Multi-Step Reasoning**
- Financial compound interest calculations
- Text analysis with sentiment detection
- Multi-tool orchestration
- Chain-of-thought reasoning

**What to Watch**:
- Distributed traces showing tool chains
- Token consumption patterns
- Latency distributions

#### 🚨 **Phase 3: Error Handling & Recovery**
- Invalid input scenarios
- Tool failure simulation
- Error recovery mechanisms
- Validation exception handling

**What to Watch**:
- Error rate metrics
- Recovery attempt traces
- Failed vs successful operations

#### ⚡ **Phase 4: Performance Stress Testing**
- Concurrent operation execution
- Load testing scenarios
- Resource utilization monitoring
- Performance bottleneck identification

**What to Watch**:
- Concurrent request handling
- Resource utilization graphs
- Performance degradation patterns

## 🔍 Exploring Telemetry Data

### Prometheus Metrics
Navigate to **http://localhost:9090** and explore these key metrics:

```promql
# Agent performance
rate(stood_agent_cycles_total[5m])
histogram_quantile(0.95, rate(stood_agent_cycle_duration_seconds_bucket[5m]))

# Token consumption
rate(stood_model_tokens_input_total[5m])
rate(stood_model_tokens_output_total[5m])

# Tool execution
rate(stood_tool_executions_total[5m])
stood_tool_execution_duration_seconds

# Error tracking
rate(stood_agent_errors_total[5m])
rate(stood_validation_exceptions_total[5m])
```

### Grafana Dashboards
Navigate to **http://localhost:3000** (admin/admin) to access:

#### **Stood Agent Telemetry Dashboard**
Pre-configured dashboard showing:
- Agent cycle rates and duration
- P95 model request latency
- Token consumption trends
- Tool execution success rates
- Error rate monitoring

#### **Creating Custom Dashboards**
1. Click "+" → "Dashboard"
2. Add panels with Prometheus data source
3. Use the metrics listed above
4. Save and organize in "Stood Telemetry" folder

### Jaeger Distributed Tracing
Navigate to **http://localhost:16686** to explore:

#### **Finding Traces**
1. **Service**: Select "stood-telemetry-demo"
2. **Operation**: Choose specific operations like:
   - `agent.basic_operations_demo`
   - `agent.complex_reasoning_demo`
   - `model.inference`
   - `tool.calculator`, `tool.weather`, etc.
3. **Time Range**: Last 1 hour (or adjust based on when you ran the demo)

#### **Trace Analysis**
Each trace shows:
- **Timeline**: Complete request flow
- **Spans**: Individual operations (agent → model → tools)
- **Attributes**: Model parameters, token usage, tool inputs/outputs
- **Events**: Important milestones and errors
- **Dependencies**: Service interaction map

## 🛠️ Advanced Configuration

### Environment Variables
Customize the telemetry demo with these environment variables:

```bash
# OpenTelemetry Configuration
export OTEL_ENABLED=true                                    # Enable telemetry
export OTEL_SERVICE_NAME=stood-telemetry-demo              # Service name
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4319   # OTLP endpoint
export OTEL_CONSOLE_EXPORT=true                            # Console debug output

# Stood-specific Configuration  
export STOOD_TELEMETRY_DEBUG=true                          # Enable debug tracing
export STOOD_TELEMETRY_BATCH_SIZE=512                      # Batch size for efficiency
export STOOD_TELEMETRY_TIMEOUT=2000                        # Batch timeout (ms)

# AWS Configuration
export AWS_REGION=us-west-2                                # Bedrock region
export AWS_PROFILE=your-profile                            # AWS profile
```

### Custom Telemetry Integration
To integrate telemetry in your own Stood applications:

```rust
use stood::{
    telemetry::{TelemetryConfig, StoodTracer, init_logging, LoggingConfig},
    agent::{Agent, AgentConfig},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure comprehensive logging
    let logging_config = LoggingConfig {
        log_dir: std::path::PathBuf::from("./logs"),
        console_enabled: false,  // Keep stdout clean
        json_format: true,       // Structured logging
        enable_performance_tracing: true,
        ..Default::default()
    };
    let _logging_guard = init_logging(logging_config)?;
    
    // Initialize telemetry
    let telemetry_config = TelemetryConfig {
        enabled: true,
        otlp_endpoint: Some("http://localhost:4319".to_string()),
        service_name: "my-stood-app".to_string(),
        ..Default::default()
    };
    
    let tracer = StoodTracer::init(telemetry_config)?.unwrap();
    
    // Your agent code with automatic telemetry
    let agent = Agent::new(tools).await?;
    let result = agent.execute_agentic("Your query").await?;
    
    // Telemetry is automatically collected!
    Ok(())
}
```

## 🎯 Production Deployment

### Docker Compose for Production
For production deployment, modify the Docker Compose configuration:

```yaml
# docker-compose.prod.yml
version: '3.8'
services:
  otel-collector:
    image: otel/opentelemetry-collector-contrib:latest
    environment:
      - OTEL_EXPORTER_OTLP_HEADERS=authorization=Bearer your-token
    volumes:
      - ./otel-collector-prod.yaml:/etc/otel-collector-config.yaml
    # Add resource limits, healthchecks, etc.

  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus-prod.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    # Add alerting rules, federation, etc.
```

### AWS Integration
For AWS environments, consider:

#### **ECS/Fargate Deployment**
```json
{
  "family": "stood-app-with-telemetry",
  "taskDefinition": {
    "containerDefinitions": [
      {
        "name": "aws-otel-collector",
        "image": "amazon/aws-otel-collector:latest",
        "command": ["--config=/etc/ecs/ecs-cloudwatch-xray.yaml"]
      },
      {
        "name": "stood-app",
        "environment": [
          {
            "name": "OTEL_EXPORTER_OTLP_ENDPOINT",
            "value": "http://localhost:4319"
          }
        ]
      }
    ]
  }
}
```

#### **CloudWatch Integration**
```yaml
# aws-otel-config.yaml
exporters:
  awscloudwatchmetrics:
    namespace: StoodAgent
    region: us-west-2
  
  awsxray:
    region: us-west-2
```

## 🔧 Troubleshooting

### Common Issues

#### **"No AWS credentials found"**
```bash
# Check credentials
aws sts get-caller-identity

# Set credentials if needed
export AWS_PROFILE=your-profile
# OR
export AWS_ACCESS_KEY_ID=xxx AWS_SECRET_ACCESS_KEY=yyy
```

#### **"Port already in use"**
```bash
# Check what's using the port
sudo lsof -i :9090

# Stop the telemetry stack
./setup-telemetry.sh stop

# Kill specific processes if needed
sudo kill -9 PID
```

#### **"Docker daemon not running"**
```bash
# Start Docker (Ubuntu/Debian)
sudo systemctl start docker

# Start Docker (macOS)
open -a Docker

# Start Docker (Windows)
# Start Docker Desktop application
```

#### **"No telemetry data appearing"**
1. **Check OpenTelemetry Collector logs**:
   ```bash
   docker logs stood-otel-collector
   ```

2. **Verify environment variables**:
   ```bash
   echo $OTEL_ENABLED
   echo $OTEL_EXPORTER_OTLP_ENDPOINT
   ```

3. **Test collector connectivity**:
   ```bash
   curl http://localhost:13133/  # Health check
   curl http://localhost:8889/metrics  # Prometheus metrics
   ```

4. **Check application logs**:
   ```bash
   # Look for telemetry initialization messages
   tail -f logs/stood.log | grep -i telemetry
   ```

### Service Health Checks
```bash
# Check all services
./setup-telemetry.sh status

# Individual service checks
curl http://localhost:9090/-/ready      # Prometheus
curl http://localhost:3000/api/health   # Grafana  
curl http://localhost:16686/           # Jaeger
curl http://localhost:13133/           # OpenTelemetry
```

### Performance Tuning
If you experience performance issues:

```bash
# Reduce telemetry overhead
export OTEL_TRACES_SAMPLER=traceidratio
export OTEL_TRACES_SAMPLER_ARG=0.1  # 10% sampling

# Increase batch sizes
export STOOD_TELEMETRY_BATCH_SIZE=2048
export STOOD_TELEMETRY_TIMEOUT=5000

# Disable console export in production
export OTEL_CONSOLE_EXPORT=false
```

## 📚 Learning Resources

### Key Concepts to Understand
1. **OpenTelemetry**: Industry standard for observability
2. **Prometheus**: Time-series metrics collection
3. **Grafana**: Metrics visualization and dashboards  
4. **Jaeger**: Distributed tracing and request flow analysis
5. **GenAI Semantic Conventions**: AI-specific observability standards

### Useful Queries and Dashboards

#### **Prometheus Alerting Rules**
```yaml
groups:
  - name: stood-agent-alerts
    rules:
      - alert: HighErrorRate
        expr: rate(stood_agent_errors_total[5m]) / rate(stood_agent_cycles_total[5m]) > 0.05
        for: 2m
        annotations:
          summary: "High error rate detected"
          
      - alert: HighLatency
        expr: histogram_quantile(0.95, rate(stood_model_request_duration_seconds_bucket[5m])) > 10
        for: 5m
        annotations:
          summary: "High model request latency"
```

#### **Advanced Grafana Queries**
```promql
# Error rate percentage
(rate(stood_agent_errors_total[5m]) / rate(stood_agent_cycles_total[5m])) * 100

# Token cost estimation (assuming $0.25 per 1M input tokens, $1.25 per 1M output tokens)
(rate(stood_model_tokens_input_total[5m]) * 60 * 0.25 / 1000000) + 
(rate(stood_model_tokens_output_total[5m]) * 60 * 1.25 / 1000000)

# Tool success rate by tool type
rate(stood_tool_executions_total{status="success"}[5m]) / 
rate(stood_tool_executions_total[5m]) by (tool_name)
```

## 🧹 Cleanup

### Stop the Demo
```bash
# Stop just the demo application
Ctrl+C  # in the terminal running the demo

# Stop the entire telemetry stack
./setup-telemetry.sh stop
```

### Complete Cleanup
```bash
# Stop and remove all containers
./setup-telemetry.sh stop
docker system prune -f

# Remove volumes (will delete stored metrics/dashboards)
docker volume rm stood-telemetry_prometheus_data
docker volume rm stood-telemetry_grafana_data

# Remove generated configuration files
rm -rf prometheus/ grafana/ otel-collector/
rm docker-compose.yml
```

## 🤝 Next Steps

After exploring this demo, you can:

1. **Integrate telemetry into your own Stood applications**
2. **Set up production monitoring with AWS CloudWatch/X-Ray**
3. **Create custom dashboards for your specific use cases**
4. **Implement alerting rules for proactive monitoring**
5. **Explore advanced OpenTelemetry features like sampling and filtering**

## 📞 Support

If you encounter issues or have questions:

1. **Check the troubleshooting section above**
2. **Review logs**: `docker logs stood-otel-collector`
3. **Verify prerequisites**: Ensure Docker, AWS credentials, and ports are available
4. **GitHub Issues**: Report bugs or request features in the Stood repository

---

**🎉 Congratulations!** You've successfully set up a comprehensive observability stack for Stood agents. This foundation will help you monitor, debug, and optimize your AI applications in production environments.