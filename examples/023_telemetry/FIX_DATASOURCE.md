# 🔧 Fix Grafana Datasource Issue

## Problem
Dashboard shows "Data source not available" because the Prometheus datasource UID doesn't match.

## Solution Steps

### Step 1: Configure Prometheus Datasource
1. **Open Grafana**: http://localhost:3000 (admin/admin)
2. **Go to**: Configuration (⚙️) → Data sources
3. **Add or Edit Prometheus datasource**

### Step 2: Prometheus Configuration
```
Name: Prometheus
Type: Prometheus
URL: http://localhost:9090
Access: Server (default)
HTTP Method: POST
```

### Step 3: Save and Test
- Click **"Save & test"**
- Should show ✅ "Data source is working"

### Step 4: Update Dashboard
After configuring the datasource:

**Option A: Re-import Dashboard**
1. Go to Dashboard → Import
2. Use the JSON file again
3. When prompted, select the correct Prometheus datasource

**Option B: Edit Existing Dashboard**
1. Open the dashboard
2. Edit any panel (click title → Edit)
3. In the query section, change datasource to "Prometheus"
4. Save the panel
5. Repeat for all panels (or save dashboard and reload)

### Step 5: Verify Metrics
Test with a simple query:
```
stood_agent_requests_total
```
You should see metric data if everything is working.

## Alternative URLs to Try
If `http://localhost:9090` doesn't work, try:
- `http://prometheus:9090` (if running in Docker network)
- `http://127.0.0.1:9090`
- Check your actual Prometheus URL with: `docker ps | grep prometheus`

## Quick Test
Once configured, this query should return data:
```
rate(stood_agent_requests_total[5m])
```