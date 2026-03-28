#!/bin/bash

# VPN Service Smoke Test Suite - Production Environment
# Validates all critical system functionality before launch

set -e

TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
REPORT_FILE="SMOKE_TEST_REPORT_$(date +%s).md"
API_URL="${API_URL:-http://localhost:8080}"
PASS_COUNT=0
FAIL_COUNT=0

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_pass() {
    echo -e "${GREEN}✓${NC} $1" | tee -a "$REPORT_FILE"
    ((PASS_COUNT++))
}

log_fail() {
    echo -e "${RED}✗${NC} $1" | tee -a "$REPORT_FILE"
    ((FAIL_COUNT++))
}

log_info() {
    echo -e "${BLUE}→${NC} $1" | tee -a "$REPORT_FILE"
}

# Initialize report
cat > "$REPORT_FILE" << EOF
# VPN Service Smoke Test Report
**Date**: $TIMESTAMP
**Environment**: production
**API URL**: $API_URL

## Test Results

EOF

echo "================================"
echo "  VPN Service Smoke Test Suite"
echo "================================"
echo ""

# Test 1: API Health
echo "Running health check tests..."
log_info "Test 1: API Health Check"

if curl -s "$API_URL/health" | jq -e '.status == "ok"' > /dev/null 2>&1; then
    log_pass "API health endpoint responding"
else
    log_fail "API health endpoint not responding"
fi

# Test 2: Database Connectivity
log_info "Test 2: Database Connectivity"

if curl -s "$API_URL/api/health/db" | jq -e '.status == "ok"' > /dev/null 2>&1; then
    log_pass "Database connected and responsive"
else
    log_fail "Database connectivity check failed"
fi

# Test 3: Redis Cache
log_info "Test 3: Redis Cache"

if curl -s "$API_URL/api/health/redis" | jq -e '.status == "ok"' > /dev/null 2>&1; then
    log_pass "Redis cache connected"
else
    log_fail "Redis cache check failed"
fi

# Test 4: Authentication Flow
log_info "Test 4: Authentication Flow"

LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"test123"}' 2>/dev/null || echo '{}')

if echo "$LOGIN_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
    AUTH_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token')
    log_pass "Authentication endpoint working (token issued)"
else
    log_fail "Authentication endpoint failed"
    AUTH_TOKEN=""
fi

# Test 5: VPN Servers Endpoint
log_info "Test 5: VPN Servers List"

if [ -n "$AUTH_TOKEN" ]; then
    if curl -s -H "Authorization: Bearer $AUTH_TOKEN" "$API_URL/api/vpn/servers" | jq -e 'length > 0' > /dev/null 2>&1; then
        log_pass "VPN servers endpoint returning data"
    else
        log_fail "VPN servers endpoint empty or failed"
    fi
else
    log_fail "Cannot test VPN servers (no auth token)"
fi

# Test 6: Rate Limiting Headers
log_info "Test 6: Rate Limiting Headers"

RATE_LIMIT_HEADER=$(curl -s -i "$API_URL/health" 2>&1 | grep -i "x-ratelimit-limit" || echo "")

if [ -n "$RATE_LIMIT_HEADER" ]; then
    log_pass "Rate limiting headers present in responses"
else
    log_fail "Rate limiting headers missing"
fi

# Test 7: CORS Headers
log_info "Test 7: CORS Configuration"

CORS_HEADER=$(curl -s -i -H "Origin: http://localhost" "$API_URL/health" 2>&1 | grep -i "access-control" || echo "")

if [ -n "$CORS_HEADER" ]; then
    log_pass "CORS headers properly configured"
else
    log_fail "CORS headers missing or misconfigured"
fi

# Test 8: Security Headers
log_info "Test 8: Security Headers"

SECURITY_HEADERS=$(curl -s -i "$API_URL/health" 2>&1 | grep -iE "x-content-type-options|x-frame-options|x-xss-protection" || echo "")

if [ -n "$SECURITY_HEADERS" ]; then
    log_pass "Security headers present"
else
    log_fail "Security headers missing"
fi

# Test 9: Response Time (P95 < 500ms)
log_info "Test 9: Response Time Performance"

RESPONSE_TIMES=()
for i in {1..10}; do
    RT=$(curl -s -w '%{time_total}' -o /dev/null "$API_URL/health")
    RESPONSE_TIMES+=("$RT")
done

SORTED_TIMES=($(for t in "${RESPONSE_TIMES[@]}"; do echo $t; done | sort -n))
P95_TIME="${SORTED_TIMES[9]}"

if (( $(echo "$P95_TIME < 0.5" | bc -l) )); then
    log_pass "Response time P95 < 500ms (actual: ${P95_TIME}s)"
else
    log_fail "Response time P95 exceeds 500ms (actual: ${P95_TIME}s)"
fi

# Test 10: Metrics Endpoint
log_info "Test 10: Prometheus Metrics"

if curl -s http://localhost:9090/metrics | grep -q "http_requests_total"; then
    log_pass "Prometheus metrics endpoint functional"
else
    log_fail "Prometheus metrics endpoint not responding"
fi

# Test 11: Monitoring Dashboards
log_info "Test 11: Grafana Dashboards"

if curl -s -u admin:admin123 http://localhost:3000/api/search?query=dashboard | jq -e 'length > 0' > /dev/null 2>&1; then
    log_pass "Grafana dashboards accessible"
else
    log_fail "Grafana dashboards not accessible"
fi

# Test 12: Alertmanager
log_info "Test 12: Alertmanager"

if curl -s http://localhost:9093/api/v1/alerts | jq -e '.status == "success"' > /dev/null 2>&1; then
    log_pass "Alertmanager responding"
else
    log_fail "Alertmanager not responding"
fi

# Test 13: Database Backup Status
log_info "Test 13: Database Backup"

LATEST_BACKUP=$(ls -t /var/app/backups/db-*.sql.gz 2>/dev/null | head -1)
BACKUP_AGE=$(($(date +%s) - $(stat -f%m "$LATEST_BACKUP" 2>/dev/null || echo 0)))

if [ "$BACKUP_AGE" -lt 86400 ]; then  # Less than 1 day old
    log_pass "Recent database backup available (age: ${BACKUP_AGE}s)"
else
    log_fail "No recent database backup found"
fi

# Test 14: SSL Certificate
log_info "Test 14: SSL Certificate"

CERT_EXPIRY=$(echo | openssl s_client -servername "$API_URL" -connect localhost:443 2>/dev/null | openssl x509 -noout -dates 2>/dev/null | grep "notAfter" || echo "")

if [ -n "$CERT_EXPIRY" ]; then
    log_pass "SSL certificate configured: $CERT_EXPIRY"
else
    log_fail "SSL certificate check failed"
fi

# Test 15: Database Connections
log_info "Test 15: Database Connection Pool"

DB_CONNECTIONS=$(curl -s "$API_URL/api/health/db" | jq '.connections.active // 0')

if [ "$DB_CONNECTIONS" -lt 100 ]; then
    log_pass "Database connection pool healthy ($DB_CONNECTIONS active)"
else
    log_fail "Database connection pool stressed ($DB_CONNECTIONS active)"
fi

# Summary
echo ""
echo "================================"
echo "  Test Summary"
echo "================================"
echo ""
echo -e "${GREEN}Passed: $PASS_COUNT${NC}"
echo -e "${RED}Failed: $FAIL_COUNT${NC}"
TOTAL=$((PASS_COUNT + FAIL_COUNT))
PASS_RATE=$((PASS_COUNT * 100 / TOTAL))
echo "Success Rate: $PASS_RATE% ($PASS_COUNT/$TOTAL)"
echo ""

# Append to report
cat >> "$REPORT_FILE" << EOF

## Summary
- **Passed**: $PASS_COUNT
- **Failed**: $FAIL_COUNT
- **Total**: $TOTAL
- **Success Rate**: $PASS_RATE%

## Recommendations

EOF

if [ $FAIL_COUNT -eq 0 ]; then
    echo -e "${GREEN}✓ ALL TESTS PASSED - PRODUCTION READY${NC}"
    echo "✓ ALL TESTS PASSED - PRODUCTION READY" >> "$REPORT_FILE"
    echo "The system is ready for production deployment." >> "$REPORT_FILE"
    exit 0
else
    echo -e "${RED}✗ SOME TESTS FAILED - REVIEW REQUIRED${NC}"
    echo "✗ SOME TESTS FAILED - REVIEW REQUIRED" >> "$REPORT_FILE"
    echo "Address the failures above before production deployment." >> "$REPORT_FILE"
    exit 1
fi
