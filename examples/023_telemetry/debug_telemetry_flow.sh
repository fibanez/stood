#!/bin/bash
# Telemetry Data Flow Debug Script

echo "🔍 Debugging Telemetry Data Flow"
echo "================================"

# Step 1: Check telemetry stack health
echo -e "\n📋 Step 1: Telemetry Stack Health"
echo "OTEL Collector (4320):"
curl -s http://localhost:4320 > /dev/null 2>&1 && echo "✅ OTLP HTTP endpoint responsive" || echo "❌ OTLP HTTP endpoint not responding"

echo "OTEL Collector Health (13133):"
curl -s http://localhost:13133/ | grep -q "available" && echo "✅ Collector healthy" || echo "❌ Collector unhealthy"

echo "Prometheus (9090):"
curl -s http://localhost:9090/-/ready > /dev/null 2>&1 && echo "✅ Prometheus ready" || echo "❌ Prometheus not ready"

# Step 2: Check collector metrics endpoint
echo -e "\n📋 Step 2: Collector Metrics Endpoint"
echo "Checking OTEL collector internal metrics..."
curl -s http://localhost:8889/metrics | wc -l | awk '{print "Lines of metrics: " $1}'

# Step 3: Check Prometheus targets
echo -e "\n📋 Step 3: Prometheus Targets"
echo "Checking Prometheus scrape targets..."
curl -s "http://localhost:9090/api/v1/targets" | grep -o '"health":"[^"]*"' | sort | uniq -c

# Step 4: Look for any stood metrics
echo -e "\n📋 Step 4: Search for Stood Metrics"
echo "Searching Prometheus for 'stood' metrics..."
curl -s "http://localhost:9090/api/v1/label/__name__/values" | grep -i stood || echo "No stood metrics found"

# Step 5: Check collector logs for recent activity
echo -e "\n📋 Step 5: Recent Collector Activity"
echo "Recent OTEL collector logs:"
docker logs stood-otel-collector --tail 5 2>/dev/null | grep -E "(received|export|stood)" || echo "No recent stood activity in logs"

# Step 6: Test OTLP endpoint directly
echo -e "\n📋 Step 6: Test OTLP Endpoint"
echo "Testing direct connection to OTLP endpoint..."
timeout 2 bash -c "</dev/tcp/localhost/4320" && echo "✅ Port 4320 is open" || echo "❌ Port 4320 not accessible"

echo -e "\n🎯 Debug Summary Complete"
echo "If no 'stood' metrics found, the issue is likely:"
echo "1. Application not sending metrics to OTLP endpoint"
echo "2. Metrics not being processed by collector"  
echo "3. Collector not exporting to Prometheus correctly"