# 🚀 Stood Agent Library - Grafana Dashboard Guide

This guide explains the comprehensive Grafana dashboard for monitoring the Stood Agent Library telemetry.

## 📊 Dashboard Overview

The dashboard provides complete observability for your Stood agent with the following sections:

### 1. 📊 Request & Performance Overview
**Key Performance Indicators (KPIs)**
- **Request Rate**: Real-time requests per second
- **P95 Request Latency**: 95th percentile response time in milliseconds  
- **Success Rate**: Percentage of successful requests
- **Concurrent Requests**: Number of active requests

### 2. 📈 Request & Latency Trends
**Time Series Analysis**
- **Request Rate by Status**: Shows success/error trends over time
- **Request Latency Percentiles**: P50, P95, P99 latency distribution

### 3. 🔧 Tool Execution Metrics
**Tool Performance Monitoring**
- **Tool Call Rate**: Calls per second by tool name and status
- **Tool Execution Duration**: P95 execution time for each tool

### 4. 🤖 Token Usage & AI Model Metrics
**AI/LLM Performance Tracking**
- **Token Usage Rate**: Input/output tokens per second
- **Tokens per Request Distribution**: Token consumption patterns
- **Total Tokens Processed**: Cumulative token usage
- **Total Model Invocations**: Number of AI model calls
- **Total Agent Cycles**: Complete agentic reasoning cycles
- **Tool Calls (24h)**: Tool usage over 24 hours

### 5. 📊 Detailed Analytics & Heatmaps
**Advanced Visualization**
- **Request Duration Heatmap**: Visual latency distribution over time
- **Tool Usage Distribution**: Pie chart of tool usage patterns

### 6. 🚨 Error Analysis & Health Monitoring
**Error Tracking & Alerting**
- **Error Rate**: Request failures over time
- **Tool Error Rate**: Tool execution failures by tool type

## 🎯 Key Metrics Explained

### Request Metrics
- `stood_agent_requests_total` - Total requests processed
- `stood_agent_request_duration_seconds` - Request processing time
- `stood_agent_concurrent_requests` - Active request count

### Token Metrics
- `stood_agent_tokens_input_total` - Input tokens consumed
- `stood_agent_tokens_output_total` - Output tokens generated  
- `stood_agent_tokens_total` - Total tokens processed
- `stood_agent_tokens_per_request` - Token distribution per request

### Tool Metrics
- `stood_agent_tool_calls_total` - Tool execution count
- `stood_agent_tool_execution_duration_seconds` - Tool execution time

### Model Metrics
- `stood_agent_model_invocations_total` - AI model calls
- `stood_agent_cycles_total` - Complete agent cycles

## 🔧 Usage Instructions

### Accessing the Dashboard
1. Open Grafana: http://localhost:3000
2. Login: admin/admin
3. Navigate to "Stood Telemetry" folder
4. Select "🚀 Stood Agent Library - Comprehensive Telemetry Dashboard"

### Time Range Selection
- Use the time picker in the top-right to adjust the observation window
- Default: Last 1 hour
- Recommended for development: Last 15 minutes
- Recommended for production: Last 24 hours

### Filtering and Drilling Down
- Click on legend items to filter specific metrics
- Use the interval variable (top-left) to adjust aggregation windows
- Hover over charts for detailed values

## 📈 Performance Baselines

### Healthy Performance Indicators
- **Success Rate**: > 95%
- **P95 Latency**: < 2000ms for agent requests
- **Tool Success Rate**: > 90%
- **Error Rate**: < 5%

### Alert Thresholds (Recommended)
- Request latency P95 > 5000ms
- Success rate < 90%
- Error rate > 10%
- Tool failure rate > 20%

## 🛠️ Troubleshooting

### No Data Showing
1. Verify Prometheus is scraping metrics: http://localhost:9090
2. Check OTLP collector status: `docker logs stood-otel-collector`
3. Confirm agent is recording metrics (look for 📊 log messages)

### Partial Data
1. Check time range - metrics may be outside the selected window
2. Verify all metrics are being exported: `curl http://localhost:9090/api/v1/label/__name__/values`
3. Check for rate expression issues (requires at least 2 data points)

### Performance Issues
1. Increase dashboard refresh interval for high-volume scenarios
2. Adjust aggregation intervals (use $interval variable)
3. Consider data retention policies in Prometheus

## 🎨 Customization

### Adding Custom Metrics
1. Add new panels using Grafana's panel editor
2. Use existing metrics as templates
3. Follow the naming convention: `stood_agent_*`

### Creating Alerts
1. Set up alert rules based on the performance baselines
2. Configure notification channels (Slack, email, etc.)
3. Use the dashboard panels as alert query references

## 📚 Related Documentation

- [Prometheus Query Guide](https://prometheus.io/docs/prometheus/latest/querying/basics/)
- [Grafana Panel Documentation](https://grafana.com/docs/grafana/latest/panels/)
- [OpenTelemetry Metrics](https://opentelemetry.io/docs/concepts/signals/metrics/)

## 🚀 Advanced Features

### Variables
- `$interval`: Adjustable time aggregation window
- Future: Add service instance filtering, environment selection

### Annotations
- Deployment markers
- Incident tracking
- Performance optimization events

### Export/Import
- Dashboard JSON can be exported for version control
- Share dashboard configuration across environments
- Import into different Grafana instances