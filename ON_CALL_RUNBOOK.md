#!/bin/bash

# On-Call Runbook for VPN Service
# Quick reference guide for on-call engineers handling incidents

# Extract current date/time
current_time=$(date '+%Y-%m-%d %H:%M:%S')

echo "==============================================="
echo "VPN Service On-Call Runbook"
echo "Generated: $current_time"
echo "==============================================="
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    local service=$1
    local cmd=$2
    local result=$(eval "$cmd" 2>&1 | head -1)
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $service: $result"
    else
        echo -e "${RED}✗${NC} $service: FAILED"
    fi
}

echo "CURRENT SERVICE STATUS"
echo "======================="
echo ""

# Check API Health
print_status "API" "curl -s http://localhost:8080/health | jq -r '.status'"

# Check Database
print_status "Database" "curl -s http://localhost:8080/api/health/db | jq -r '.status'"

# Check Redis
print_status "Redis" "curl -s http://localhost:8080/api/health/redis | jq -r '.status'"

# Check Monitoring
print_status "Prometheus" "curl -s http://localhost:9090/-/healthy"
print_status "Grafana" "curl -s http://localhost:3000/api/health | jq -r '.database'"

echo ""
echo "QUICK COMMANDS"
echo "=============="
echo ""
echo "View API Logs:"
echo "  tail -f /tmp/vpn-api.log"
echo ""
echo "View Database Logs:"
echo "  docker logs vpn_postgres_1 -f"
echo ""
echo "View Redis Logs:"
echo "  docker logs vpn_redis_1 -f"
echo ""
echo "Database Shell:"
echo "  psql postgresql://postgres:postgres@localhost:5432/vpn_service"
echo ""
echo "Redis CLI:"
echo "  redis-cli -p 6379"
echo ""
echo "Restart API Service:"
echo "  systemctl restart vpn-api"
echo ""
echo "Check CPU Usage:"
echo "  top -o %CPU"
echo ""
echo "Check Memory Usage:"
echo "  free -h"
echo ""
echo "Check Disk Usage:"
echo "  df -h"
echo ""
echo "View Open Port Connections:"
echo "  netstat -tuln | grep LISTEN"
echo ""

echo "COMMON INCIDENT PROCEDURES"
echo "=========================="
echo ""

# Menu
echo "1. API Down"
echo "   a) Check logs: tail -f /tmp/vpn-api.log"
echo "   b) Check process: ps aux | grep vpn-api"
echo "   c) Check port (8080): netstat -tuln | grep 8080"
echo "   d) Restart: systemctl restart vpn-api"
echo "   e) Check if DB is responsive: psql -c 'SELECT 1'"
echo ""

echo "2. Database Connection Errors"
echo "   a) Check DB container: docker ps | grep postgres"
echo "   b) Check DB logs: docker logs vpn_postgres_1"
echo "   c) Test connection: psql postgresql://postgres:postgres@localhost/vpn_service -c 'SELECT 1'"
echo "   d) Check connection pool: curl http://localhost:9090/api/v1/query?query=pg_stat_activity_count"
echo ""

echo "3. High Memory Usage"
echo "   a) Identify process: ps aux --sort=-%mem | head -5"
echo "   b) Check container limits: docker stats"
echo "   c) Review memory metrics: curl 'http://localhost:9090/api/v1/query?query=container_memory_usage_bytes'"
echo "   d) If API is culprit, restart it: systemctl restart vpn-api"
echo ""

echo "4. High CPU Usage"
echo "   a) Identify hot process: top -o %CPU"
echo "   b) Check for runaway queries: docker exec vpn_postgres_1 psql -U postgres -d vpn_service -c 'SELECT * FROM pg_stat_statements ORDER BY total_time DESC LIMIT 10'"
echo "   c) Kill long-running query: SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE state = 'active' AND query_start < now() - interval '30 minutes'"
echo ""

echo "5. Disk Space Critical"
echo "   a) Check usage: df -h"
echo "   b) Find large files: du -sh /* | sort -h"
echo "   c) Clean logs: docker exec vpn_postgres_1 truncate -s 0 /var/log/postgresql/postgresql.log"
echo "   d) Check backups: ls -lh backups/"
echo ""

echo "6. Rate Limiting False Positives"
echo "   a) Check rate limit metrics: curl 'http://localhost:9090/api/v1/query?query=rate_limiter_blocked_requests_total'"
echo "   b) View active rate limit entries: redis-cli KEYS 'rate_limit:*' | wc -l"
echo "   c) Clear specific user limit: redis-cli DEL 'rate_limit:user:${USER_ID}'"
echo "   d) Clear all limits (use with caution): redis-cli FLUSHALL"
echo ""

echo "7. Payment Processing Failures"
echo "   a) Check Stripe webhook logs: grep 'stripe' /tmp/vpn-api.log | tail -20"
echo "   b) View failed payments: SELECT * FROM payments WHERE status = 'failed' ORDER BY created_at DESC LIMIT 10"
echo "   c) Retry failed payment: UPDATE payments SET status = 'retry' WHERE id = '${PAYMENT_ID}'"
echo ""

echo "ESCALATION CONTACTS"
echo "==================="
echo ""
echo "Level 1 (On-Call Engineer)"
echo "  Slack: @on-call"
echo "  Phone: ${ONCALL_PHONE}"
echo ""
echo "Level 2 (Team Lead)"
echo "  Slack: @lead"
echo "  Phone: ${LEAD_PHONE}"
echo ""
echo "Level 3 (CTO)"
echo "  Slack: @cto"
echo "  Email: cto@vpn-service.com"
echo ""

echo "EMERGENCY CONTACTS"
echo "=================="
echo ""
echo "Critical Infrastructure:"
echo "  AWS Support: https://console.aws.amazon.com/support"
echo "  Database Vendor: support@postgresql.org"
echo "  Payment Provider: support@stripe.com"
echo ""

echo "KEY METRICS TO MONITOR"
echo "====================="
echo ""
echo "Dashboard URLs:"
echo "  1. API Performance: http://localhost:3000/d/api-performance"
echo "  2. Infrastructure:  http://localhost:3000/d/infrastructure"
echo "  3. Database:        http://localhost:3000/d/database"
echo "  4. Security:        http://localhost:3000/d/security"
echo ""
echo "Critical Thresholds:"
echo "  • Error Rate: > 1%"
echo "  • P95 Latency: > 500ms"
echo "  • CPU Usage: > 80%"
echo "  • Memory Usage: > 85%"
echo "  • DB Connections: > 90% of pool"
echo "  • Disk Usage: > 90%"
echo ""

echo "DOCUMENTATION"
echo "=============="
echo ""
echo "Full Guides:"
echo "  • Incident Response: ./INCIDENT_RESPONSE.md"
echo "  • Production Runbook: ./PRODUCTION_DEPLOYMENT_RUNBOOK.md"
echo "  • Canary Deployment: ./CANARY_DEPLOYMENT.md"
echo "  • Architecture Docs: ./docs/architecture/"
echo ""

echo "==============================================="
echo "For detailed troubleshooting, see INCIDENT_RESPONSE.md"
echo "For deployment issues, see PRODUCTION_DEPLOYMENT_RUNBOOK.md"
echo "==============================================="
echo ""
