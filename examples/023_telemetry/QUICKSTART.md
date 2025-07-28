# 🚀 Telemetry Demo - Quick Start (2 Minutes)

Get the Stood telemetry demo running in 2 minutes with these simple commands:

## ⚡ Prerequisites Check
```bash
# Ensure you have these installed:
docker --version         # ✅ Docker
cargo --version         # ✅ Rust/Cargo  
aws sts get-caller-identity  # ✅ AWS credentials (or set AWS_PROFILE)
```

## 🎯 Quick Start Commands

### 1. Navigate to Demo Directory
```bash
cd examples/docs/004_telemetry
```

### 2. Start Monitoring Stack (1 minute)
```bash
./setup-telemetry.sh
```
Wait for: `✅ All services are ready!`

### 3. Run the Demo (30 seconds)
```bash
./run-demo.sh
```

## 📊 View Results Immediately

Open these URLs while the demo runs:

| 📈 **Prometheus** | http://localhost:9090 |
| 📊 **Grafana** | http://localhost:3000 (admin/admin) |
| 🔍 **Jaeger** | http://localhost:16686 |

## 🎯 What to Look For

### In Grafana (http://localhost:3000):
1. Login: `admin` / `admin`
2. Go to "Dashboards" → "Stood Agent Telemetry" 
3. Watch real-time metrics appear as the demo runs

### In Jaeger (http://localhost:16686):
1. Service: Select "stood-telemetry-demo"
2. Click "Find Traces"
3. Click on any trace to see the detailed request flow

### In Prometheus (http://localhost:9090):
Try these queries:
```promql
rate(stood_agent_cycles_total[5m])
histogram_quantile(0.95, rate(stood_model_request_duration_seconds_bucket[5m]))
```

## 🔧 Common Quick Fixes

**AWS Credentials Error?**
```bash
export AWS_PROFILE=your-profile
# OR
export AWS_ACCESS_KEY_ID=xxx AWS_SECRET_ACCESS_KEY=yyy
```

**Port Already in Use?**
```bash
./setup-telemetry.sh stop
./setup-telemetry.sh start
```

**No Data in Grafana?**
- Wait 30 seconds for data to appear
- Refresh the dashboard
- Check that the demo is actually running

## 🛑 Stop Everything
```bash
# Stop demo: Ctrl+C
# Stop monitoring stack:
./setup-telemetry.sh stop
```

## 🎓 Next Steps
Once you see data flowing:
1. Read the full [README.md](README.md) for detailed explanations
2. Explore custom queries in Prometheus
3. Create your own Grafana dashboards
4. Examine distributed traces in Jaeger

**Total Time: ~2 minutes to see live telemetry data! 🎉**