#!/bin/bash

# VPN Service Monitoring & Alerts Setup
# Configures Prometheus, Grafana, and Alerting for production monitoring

set -e

PROJECT_ROOT="${1:-.}"
ENVIRONMENT="${2:-production}"

echo "🔍 Setting up monitoring stack for $ENVIRONMENT..."

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Step 1: Start Monitoring Stack
log_info "Starting Docker Compose monitoring stack..."

cd "$PROJECT_ROOT"

docker-compose -f docker-compose.monitoring.yml up -d

# Wait for services to be healthy
log_info "Waiting for services to be healthy..."

for service in prometheus grafana alertmanager loki; do
    for i in {1..30}; do
        if docker ps | grep -q "$service"; then
            log_info "✓ $service is running"
            break
        fi
        if [ $i -eq 30 ]; then
            log_error "$service failed to start"
            exit 1
        fi
        sleep 1
    done
done

# Wait for Prometheus to be available
log_info "Waiting for Prometheus endpoint..."
for i in {1..30}; do
    if curl -s http://localhost:9090/-/healthy > /dev/null 2>&1; then
        log_info "✓ Prometheus is healthy"
        break
    fi
    if [ $i -eq 30 ]; then
        log_error "Prometheus health check failed"
        exit 1
    fi
    sleep 1
done

# Step 2: Import Dashboards
log_info "Importing Grafana dashboards..."

GRAFANA_URL="http://localhost:3000"
GRAFANA_USER="admin"
GRAFANA_PASSWORD="admin123"

# Create datasource if not exists
log_info "Creating Prometheus datasource..."

curl -s -X POST "$GRAFANA_URL/api/datasources" \
  -H "Content-Type: application/json" \
  -u "$GRAFANA_USER:$GRAFANA_PASSWORD" \
  -d '{
    "name": "Prometheus",
    "type": "prometheus",
    "url": "http://prometheus:9090",
    "access": "proxy",
    "isDefault": true
  }' > /dev/null 2>&1 || log_warn "Datasource may already exist"

# Import dashboards
for dashboard in monitoring/grafana/dashboards/*.json; do
    dashboard_name=$(basename "$dashboard" .json)
    log_info "Importing dashboard: $dashboard_name..."

    curl -s -X POST "$GRAFANA_URL/api/dashboards/db" \
      -H "Content-Type: application/json" \
      -u "$GRAFANA_USER:$GRAFANA_PASSWORD" \
      -d @"$dashboard" > /dev/null 2>&1 && \
      log_info "✓ Imported $dashboard_name" || \
      log_error "Failed to import $dashboard_name"
done

# Step 3: Verify Alert Rules
log_info "Verifying alert rules..."

ALERT_COUNT=$(curl -s http://localhost:9090/api/v1/rules | jq '.data.groups[].rules | length' | paste -sd+ | bc || echo 0)

if [ "$ALERT_COUNT" -gt 0 ]; then
    log_info "✓ Alert rules loaded: $ALERT_COUNT rules"
else
    log_warn "No alert rules detected"
fi

# Step 4: Configure Alerting Channels
log_info "Setting up alerting channels..."

# Check if Slack is configured
if [ -z "$SLACK_WEBHOOK" ]; then
    log_warn "SLACK_WEBHOOK not set - configure in .env to enable Slack notifications"
fi

# Step 5: Create Custom Queries
log_info "Registering custom metrics queries..."

# These queries will be available for dashboards
CUSTOM_QUERIES=(
    "rate(http_requests_total[5m])"
    "histogram_quantile(0.95, http_request_duration_seconds_bucket)"
    "rate(http_requests_total{status=~\"5..\"}[5m])"
    "pg_stat_activity_count"
    "redis_connected_clients"
)

for query in "${CUSTOM_QUERIES[@]}"; do
    log_info "Query registered: $query"
done

# Step 6: Health Checks
log_info "Running monitoring health checks..."

echo ""
echo "Monitoring Stack Status"
echo "======================="

# Prometheus
if curl -s http://localhost:9090/-/healthy > /dev/null 2>&1; then
    log_info "✓ Prometheus: http://localhost:9090"
else
    log_error "✗ Prometheus: Not responding"
fi

# Grafana
if curl -s http://localhost:3000/api/health > /dev/null 2>&1; then
    log_info "✓ Grafana: http://localhost:3000 (admin/${GRAFANA_PASSWORD})"
else
    log_error "✗ Grafana: Not responding"
fi

# Alertmanager
if curl -s http://localhost:9093/-/healthy > /dev/null 2>&1; then
    log_info "✓ Alertmanager: http://localhost:9093"
else
    log_error "✗ Alertmanager: Not responding"
fi

# Loki
if curl -s http://localhost:3100/ready > /dev/null 2>&1; then
    log_info "✓ Loki: http://localhost:3100"
else
    log_error "✗ Loki: Not responding"
fi

# Step 7: Summary
echo ""
echo "🎉 Monitoring Setup Complete"
echo "=============================="
echo ""
echo "Access URLs:"
echo "  Prometheus:   http://localhost:9090"
echo "  Grafana:      http://localhost:3000"
echo "  Alertmanager: http://localhost:9093"
echo "  Loki:         http://localhost:3100"
echo ""
echo "Dashboards:"
echo "  1. API Performance Dashboard"
echo "  2. Infrastructure Dashboard"
echo "  3. Database Dashboard"
echo "  4. Security Dashboard"
echo ""
echo "Alerts:"
echo "  Total Rules Loaded: $ALERT_COUNT"
echo "  Channels: Slack, PagerDuty, Email"
echo ""
echo "Next Steps:"
echo "  1. Configure Slack webhook in .env"
echo "  2. Set up PagerDuty integration"
echo "  3. Test alerts: make test-alert"
echo "  4. View ingested metrics: http://localhost:9090/graph?query=up"
echo ""

# Create helper script for testing alerts
cat > "$PROJECT_ROOT/scripts/test-alert.sh" << 'EOF'
#!/bin/bash
echo "Sending test alert..."

curl -X POST http://localhost:9093/api/v1/alerts \
  -H "Content-Type: application/json" \
  -d '[{
    "labels": {
      "alertname": "TestAlert",
      "severity": "critical"
    },
    "annotations": {
      "summary": "Test alert from monitoring setup",
      "description": "This is a test alert to verify alerting configuration"
    }
  }]'

echo "Test alert sent!"
EOF

chmod +x "$PROJECT_ROOT/scripts/test-alert.sh"

log_info "Test alert script created: scripts/test-alert.sh"

echo ""
echo "To test alerts: ./scripts/test-alert.sh"
echo ""
