# Port Configuration - Stood Telemetry Demo

This document explains the port configuration for the Stood telemetry demo and why certain ports were chosen.

## Port Allocation

| Service | Port | Protocol | Purpose |
|---------|------|----------|---------|
| **Grafana** | 3000 | HTTP | Dashboard UI |
| **Prometheus** | 9090 | HTTP | Metrics collection UI |
| **Jaeger UI** | 16686 | HTTP | Tracing UI |
| **Jaeger gRPC** | 14250 | gRPC | Jaeger receiver |
| **Jaeger OTLP** | 4317, 4318 | gRPC/HTTP | OTLP endpoints for Jaeger |
| **OpenTelemetry Collector** | 4319→4317, 4320→4318 | gRPC/HTTP | External OTLP endpoints |
| **OpenTelemetry Health** | 13133 | HTTP | Health check endpoint |
| **Node Exporter** | 9100 | HTTP | System metrics |
| **Loki** | 3100 | HTTP | Log aggregation |

## Port Conflict Resolution

### Problem
Originally, both Jaeger and OpenTelemetry Collector were configured to use ports 4317/4318, causing a startup conflict:

```
Error response from daemon: failed to set up container networking: 
driver failed programming external connectivity on endpoint stood-otel-collector: 
Bind for 0.0.0.0:4317 failed: port is already allocated
```

### Solution
We resolved this by:

1. **Keeping Jaeger on original ports** (4317/4318) since it binds directly to these ports
2. **Mapping OpenTelemetry Collector** to external ports 4319/4320 while keeping internal ports 4317/4318
3. **Updating all client configurations** to use port 4319 for OTLP connections

### Configuration Details

#### Docker Port Mapping
```yaml
# Jaeger (unchanged)
jaeger:
  ports:
    - "4317:4317"   # OTLP gRPC receiver (Jaeger)
    - "4318:4318"   # OTLP HTTP receiver (Jaeger)

# OpenTelemetry Collector (updated)
otel-collector:
  ports:
    - "4319:4317"   # OTLP gRPC receiver (external)
    - "4320:4318"   # OTLP HTTP receiver (external)
```

#### Client Configuration
All Stood telemetry clients now connect to:
```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4319
```

#### Internal Communication
- OpenTelemetry Collector internally uses ports 4317/4318
- Jaeger receives traces from OpenTelemetry Collector via `jaeger:4317`
- External clients connect to OpenTelemetry Collector via `localhost:4319`

## Data Flow

```
Stood Agent → localhost:4319 → OpenTelemetry Collector → jaeger:4317 → Jaeger
                               ↓
                            Prometheus ← 8889
```

## Troubleshooting

### Check Port Usage
```bash
# Check if ports are in use
netstat -tulpn | grep :4317  # Should show Jaeger
netstat -tulpn | grep :4319  # Should show OpenTelemetry Collector

# Check service health
curl http://localhost:13133/  # OpenTelemetry Collector health
curl http://localhost:16686/  # Jaeger UI
```

### Port Conflicts
If you encounter port conflicts:

1. **Stop the telemetry stack**: `./setup-telemetry.sh stop`
2. **Check what's using ports**: `lsof -i :4317` (requires sudo)
3. **Kill conflicting processes**: `sudo kill -9 <PID>`
4. **Restart**: `./setup-telemetry.sh`

### Custom Port Configuration
To use different ports, modify:

1. `docker-compose.yml` - Update port mappings
2. `run-demo.sh` - Update `OTEL_EXPORTER_OTLP_ENDPOINT`
3. `telemetry_demo.rs` - Update default endpoint
4. `README.md` - Update documentation

## Security Notes

- All services bind to `0.0.0.0` for demo purposes
- In production, consider binding to specific interfaces
- Use TLS for production OTLP endpoints
- Implement authentication for Grafana/Prometheus in production