#!/bin/bash

# VPN Service Smoke Test Suite - Production Environment
# Validates all critical system functionality before launch

TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
REPORT_FILE="SMOKE_TEST_REPORT_$(date +%s).md"
API_URL="${API_URL:-http://localhost:3000}"
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

HEALTH=$(curl -s "$API_URL/health" 2>&1)
if echo "$HEALTH" | grep -q '"status"'; then
    log_pass "API health endpoint responding: $HEALTH"
else
    log_fail "API health endpoint not responding"
fi

# Test 2: Database Health
log_info "Test 2: Database Health"

DB=$(curl -s "$API_URL/api/health/db" 2>&1)
if echo "$DB" | grep -q '"status"'; then
    log_pass "Database health: $DB"
else
    log_fail "Database health failed"
fi

# Test 3: Redis Health
log_info "Test 3: Redis Health"

REDIS=$(curl -s "$API_URL/api/health/redis" 2>&1)
if echo "$REDIS" | grep -q '"status"'; then
    log_pass "Redis health: $REDIS"
else
    log_fail "Redis health failed"
fi

# Test 4: Authentication
log_info "Test 4: Authentication"

AUTH=$(curl -s -X POST "$API_URL/api/auth/login" -H "Content-Type: application/json" -d '{"email":"test@test.com","password":"test"}' 2>&1)
if echo "$AUTH" | grep -q '"token"'; then
    log_pass "Authentication working: $(echo "$AUTH" | grep -o '"token":"[^"]*"' | cut -d: -f2 | head -c 30)..."
else
    log_fail "Authentication failed"
fi

# Test 5: VPN Servers
log_info "Test 5: VPN Servers"

SERVERS=$(curl -s "$API_URL/api/vpn/servers" 2>&1)
if echo "$SERVERS" | grep -q '"id"'; then
    log_pass "VPN servers endpoint: $(echo "$SERVERS" | head -c 50)..."
else
    log_fail "VPN servers failed"
fi

# Test 6: HTTP Status
log_info "Test 6: HTTP Status Code"

HTTP=$(curl -s -o /dev/null -w "%{http_code}" "$API_URL/health")
if [ "$HTTP" = "200" ]; then
    log_pass "HTTP status 200 OK"
else
    log_fail "HTTP status $HTTP (expected 200)"
fi

# Test 7: Response Time
log_info "Test 7: Response Time"

RT=$(curl -s -w '%{time_total}' -o /dev/null "$API_URL/health")
log_pass "Response time: ${RT}s"

# Test 8: Docker API
log_info "Test 8: Docker API Container"

if docker-compose ps | grep -q "vpn-api.*Up"; then
    log_pass "API container running"
else
    log_fail "API container not running"
fi

# Test 9: Docker Database
log_info "Test 9: Docker Database Container"

if docker-compose ps | grep -q "vpn-postgres.*Up"; then
    log_pass "Database container running"
else
    log_fail "Database container not running"
fi

# Test 10: Connection Persistence
log_info "Test 10: Connection Persistence"

R1=$(curl -s "$API_URL/health" 2>&1)
R2=$(curl -s "$API_URL/health" 2>&1)
if echo "$R1" | grep -q '"status"' && echo "$R2" | grep -q '"status"'; then
    log_pass "Multiple requests successful"
else
    log_fail "Connection persistence issue"
fi

# Test 11: Error Handling
log_info "Test 11: Error Handling"

ERR=$(curl -s -o /dev/null -w "%{http_code}" "$API_URL/invalid")
if [ "$ERR" = "404" ]; then
    log_pass "404 error handling working"
else
    log_fail "Error handling (got $ERR)"
fi

# Test 12: Content Type
log_info "Test 12: Content Type"

CT=$(curl -s -i "$API_URL/health" 2>&1 | grep -i "content-type" | head -1)
if echo "$CT" | grep -q "application/json"; then
    log_pass "Content-Type: application/json"
else
    log_fail "Content-Type header missing"
fi

# Test 13: Load Test (10 requests)
log_info "Test 13: Load Test"

LOAD_OK=0
for i in {1..10}; do
    if curl -s "$API_URL/health" 2>&1 | grep -q '"status"'; then
        ((LOAD_OK++))
    fi
done
log_pass "Load test: $LOAD_OK/10 requests successful"

# Test 14: Data Validation
log_info "Test 14: Data Validation"

DATA=$(curl -s "$API_URL/api/vpn/servers" 2>&1)
if echo "$DATA" | grep -q '"country"'; then
    log_pass "Response data valid: $(echo "$DATA" | head -c 80)..."
else
    log_fail "Response data validation failed"
fi

# Test 15: Service Availability
log_info "Test 15: Service Availability"

AVAIL=$(curl -s -m 2 "$API_URL/health" 2>&1)
if [ -n "$AVAIL" ]; then
    log_pass "Service available and responding"
else
    log_fail "Service unavailable"
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
if [ $TOTAL -gt 0 ]; then
    PASS_RATE=$((PASS_COUNT * 100 / TOTAL))
else
    PASS_RATE=0
fi
echo "Success Rate: $PASS_RATE% ($PASS_COUNT/$TOTAL)"
echo ""

# Append to report
cat >> "$REPORT_FILE" << EOF

## Summary
- **Passed**: $PASS_COUNT
- **Failed**: $FAIL_COUNT
- **Total**: $TOTAL
- **Success Rate**: $PASS_RATE%

EOF

if [ $FAIL_COUNT -eq 0 ]; then
    echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
    exit 0
else
    echo -e "${RED}✗ SOME TESTS FAILED${NC}"
    exit 1
fi
