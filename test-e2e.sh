#!/bin/bash

# VPN Service - End-to-End Test Script
# This script validates the complete system including:
# - Backend API endpoints
# - Frontend build
# - Database connectivity
# - Full authentication flow
# - Peer creation and config generation

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
API_URL="http://localhost:3000"
FRONTEND_URL="http://localhost:4200"
DB_URL="postgres://vpn_user:vpn_password@localhost:5432/vpn_service"

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
    ((TESTS_PASSED++))
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
    ((TESTS_FAILED++))
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# Test 1: Check if services are running
test_services_running() {
    print_header "TEST 1: Service Availability"
    
    # Check backend
    if curl -s "$API_URL/health" > /dev/null; then
        print_success "Backend API is running on $API_URL"
    else
        print_error "Backend API is NOT running on $API_URL"
        return 1
    fi
    
    # Check frontend
    if curl -s "$FRONTEND_URL" > /dev/null; then
        print_success "Frontend is running on $FRONTEND_URL"
    else
        print_error "Frontend is NOT running on $FRONTEND_URL"
        return 1
    fi
}

# Test 2: Database connectivity
test_database() {
    print_header "TEST 2: Database Connectivity"
    
    # Check if connection works
    if PGPASSWORD=vpn_password psql -h localhost -U vpn_user -d vpn_service -c "SELECT version();" > /dev/null 2>&1; then
        print_success "PostgreSQL database is accessible"
    else
        print_error "PostgreSQL database is NOT accessible"
        return 1
    fi
    
    # Check if tables exist
    TABLE_COUNT=$(PGPASSWORD=vpn_password psql -h localhost -U vpn_user -d vpn_service -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public';" | xargs)
    if [ "$TABLE_COUNT" -ge 5 ]; then
        print_success "Database schema exists ($TABLE_COUNT tables)"
    else
        print_error "Database schema not properly initialized (found $TABLE_COUNT tables)"
        return 1
    fi
}

# Test 3: Backend health checks
test_backend_health() {
    print_header "TEST 3: Backend Health Checks"
    
    HEALTH=$(curl -s "$API_URL/health")
    if echo "$HEALTH" | grep -q '"status":"ok"'; then
        print_success "Health check endpoint responding"
    else
        print_error "Health check endpoint failed"
        return 1
    fi
    
    READY=$(curl -s "$API_URL/health/ready")
    if echo "$READY" | grep -q '"status":"ready"'; then
        print_success "Readiness check endpoint responding"
    else
        print_error "Readiness check endpoint failed"
        return 1
    fi
}

# Test 4: User registration
test_user_registration() {
    print_header "TEST 4: User Registration"
    
    TEST_EMAIL="test_$(date +%s)@example.com"
    TEST_PASSWORD="TestPassword123"
    
    REGISTER_RESPONSE=$(curl -s -X POST "$API_URL/auth/register" \
        -H "Content-Type: application/json" \
        -d "{
            \"email\": \"$TEST_EMAIL\",
            \"password\": \"$TEST_PASSWORD\"
        }")
    
    if echo "$REGISTER_RESPONSE" | grep -q '"status":"success"'; then
        print_success "User registration successful for $TEST_EMAIL"
        
        # Extract token
        TOKEN=$(echo "$REGISTER_RESPONSE" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
        if [ ! -z "$TOKEN" ]; then
            print_success "JWT token received (${#TOKEN} characters)"
            export TEST_TOKEN=$TOKEN
            export TEST_EMAIL=$TEST_EMAIL
        else
            print_error "Failed to extract JWT token from response"
            return 1
        fi
    else
        print_error "User registration failed"
        print_info "Response: $REGISTER_RESPONSE"
        return 1
    fi
}

# Test 5: User login
test_user_login() {
    print_header "TEST 5: User Login"
    
    if [ -z "$TEST_EMAIL" ] || [ -z "$TEST_PASSWORD" ]; then
        print_error "Test email or password not set"
        return 1
    fi
    
    LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/auth/login" \
        -H "Content-Type: application/json" \
        -d "{
            \"email\": \"$TEST_EMAIL\",
            \"password\": \"$TEST_PASSWORD\"
        }")
    
    if echo "$LOGIN_RESPONSE" | grep -q '"status":"success"'; then
        print_success "User login successful"
        
        LOGIN_TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
        if [ ! -z "$LOGIN_TOKEN" ]; then
            print_success "Login JWT token received"
            export TEST_TOKEN=$LOGIN_TOKEN
        fi
    else
        print_error "User login failed"
        print_info "Response: $LOGIN_RESPONSE"
        return 1
    fi
}

# Test 6: List nodes
test_list_nodes() {
    print_header "TEST 6: List Nodes"
    
    if [ -z "$TEST_TOKEN" ]; then
        print_error "Test token not available"
        return 1
    fi
    
    NODES_RESPONSE=$(curl -s -X GET "$API_URL/nodes" \
        -H "Authorization: Bearer $TEST_TOKEN")
    
    if echo "$NODES_RESPONSE" | grep -q '"status":"success"'; then
        print_success "Nodes endpoint responding"
        
        NODE_COUNT=$(echo "$NODES_RESPONSE" | grep -o '"id":"[^"]*"' | wc -l)
        print_info "Found $NODE_COUNT nodes in system"
        
        if [ "$NODE_COUNT" -gt 0 ]; then
            # Extract first node ID
            NODE_ID=$(echo "$NODES_RESPONSE" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
            export TEST_NODE_ID=$NODE_ID
            print_success "Using node ID: $NODE_ID"
        else
            print_error "No nodes available in system (might need to create one)"
            return 1
        fi
    else
        print_error "Nodes endpoint failed"
        return 1
    fi
}

# Test 7: Create peer
test_create_peer() {
    print_header "TEST 7: Create Peer (VPN Client)"
    
    if [ -z "$TEST_TOKEN" ] || [ -z "$TEST_NODE_ID" ]; then
        print_error "Test token or node ID not available"
        return 1
    fi
    
    PEER_NAME="test-peer-$(date +%s)"
    
    CREATE_PEER=$(curl -s -X POST "$API_URL/peers" \
        -H "Authorization: Bearer $TEST_TOKEN" \
        -H "Content-Type: application/json" \
        -d "{
            \"name\": \"$PEER_NAME\",
            \"node_id\": \"$TEST_NODE_ID\"
        }")
    
    if echo "$CREATE_PEER" | grep -q '"status":"success"'; then
        print_success "Peer created successfully: $PEER_NAME"
        
        PEER_ID=$(echo "$CREATE_PEER" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
        if [ ! -z "$PEER_ID" ]; then
            export TEST_PEER_ID=$PEER_ID
            print_success "Peer ID: $PEER_ID"
        fi
    else
        print_error "Failed to create peer"
        print_info "Response: $CREATE_PEER"
        return 1
    fi
}

# Test 8: Get peer config
test_get_peer_config() {
    print_header "TEST 8: Get WireGuard Configuration"
    
    if [ -z "$TEST_TOKEN" ] || [ -z "$TEST_PEER_ID" ]; then
        print_error "Test token or peer ID not available"
        return 1
    fi
    
    CONFIG=$(curl -s -X GET "$API_URL/peers/$TEST_PEER_ID/config" \
        -H "Authorization: Bearer $TEST_TOKEN")
    
    if echo "$CONFIG" | grep -q '"status":"success"'; then
        print_success "WireGuard config requested successfully"
        
        WG_CONFIG=$(echo "$CONFIG" | grep -o '"config":"[^"]*"' | cut -d'"' -f4 | head -1)
        if [ ! -z "$WG_CONFIG" ] && echo "$WG_CONFIG" | grep -q "\\[Interface\\]"; then
            print_success "Valid WireGuard configuration received"
        else
            print_error "Invalid WireGuard configuration format"
            return 1
        fi
    else
        print_error "Failed to get peer config"
        return 1
    fi
}

# Test 9: List peers
test_list_peers() {
    print_header "TEST 9: List User Peers"
    
    if [ -z "$TEST_TOKEN" ]; then
        print_error "Test token not available"
        return 1
    fi
    
    LIST_PEERS=$(curl -s -X GET "$API_URL/peers" \
        -H "Authorization: Bearer $TEST_TOKEN")
    
    if echo "$LIST_PEERS" | grep -q '"status":"success"'; then
        print_success "Peers list endpoint responding"
        
        PEER_COUNT=$(echo "$LIST_PEERS" | grep -o '"id":"[^"]*"' | wc -l)
        print_info "User has $PEER_COUNT peer(s)"
    else
        print_error "Failed to list peers"
        return 1
    fi
}

# Test 10: Delete peer
test_delete_peer() {
    print_header "TEST 10: Delete Peer"
    
    if [ -z "$TEST_TOKEN" ] || [ -z "$TEST_PEER_ID" ]; then
        print_error "Test token or peer ID not available"
        return 1
    fi
    
    DELETE_PEER=$(curl -s -X DELETE "$API_URL/peers/$TEST_PEER_ID" \
        -H "Authorization: Bearer $TEST_TOKEN")
    
    if echo "$DELETE_PEER" | grep -q '"status":"success"'; then
        print_success "Peer deleted successfully"
    else
        print_error "Failed to delete peer"
        print_info "Response: $DELETE_PEER"
        return 1
    fi
}

# Test 11: Error handling - unauthorized
test_unauthorized_error() {
    print_header "TEST 11: Error Handling - Unauthorized"
    
    UNAUTH=$(curl -s -X GET "$API_URL/peers")
    
    if echo "$UNAUTH" | grep -q '"status":"error"'; then
        print_success "Unauthorized access properly rejected"
    else
        print_error "Should have received unauthorized error"
        return 1
    fi
}

# Test 12: Error handling - invalid token
test_invalid_token_error() {
    print_header "TEST 12: Error Handling - Invalid Token"
    
    INVALID=$(curl -s -X GET "$API_URL/peers" \
        -H "Authorization: Bearer invalid_token_here")
    
    if echo "$INVALID" | grep -q '"status":"error"'; then
        print_success "Invalid token properly rejected"
    else
        print_error "Should have received invalid token error"
        return 1
    fi
}

# Test 13: Frontend build verification
test_frontend_build() {
    print_header "TEST 13: Frontend Build Verification"
    
    FRONTEND_DIR="/Users/sultonshonazarshoev/Documents/vpn-service/frontend"
    
    if [ -d "$FRONTEND_DIR/dist" ]; then
        print_success "Frontend production build exists"
        
        BUILD_SIZE=$(du -sh "$FRONTEND_DIR/dist" | cut -f1)
        print_info "Build size: $BUILD_SIZE"
    else
        print_error "Frontend build directory not found"
        return 1
    fi
    
    if [ -f "$FRONTEND_DIR/package.json" ]; then
        print_success "package.json found"
    else
        print_error "package.json not found"
        return 1
    fi
}

# Test 14: Database integrity
test_database_integrity() {
    print_header "TEST 14: Database Integrity"
    
    # Check users table
    USER_COUNT=$(PGPASSWORD=vpn_password psql -h localhost -U vpn_user -d vpn_service -t -c "SELECT COUNT(*) FROM users;" | xargs)
    print_info "Users in database: $USER_COUNT"
    
    # Check if our test user was created
    if [ "$USER_COUNT" -gt 0 ]; then
        print_success "User data stored in database"
    else
        print_error "No users in database"
        return 1
    fi
    
    # Check audit logs
    LOG_COUNT=$(PGPASSWORD=vpn_password psql -h localhost -U vpn_user -d vpn_service -t -c "SELECT COUNT(*) FROM audit_logs;" 2>/dev/null | xargs)
    if [ ! -z "$LOG_COUNT" ] && [ "$LOG_COUNT" -gt 0 ]; then
        print_success "Audit logs being recorded ($LOG_COUNT events)"
    else
        print_info "No audit logs yet (database might not support it)"
    fi
}

# Test 15: Performance - Response time
test_performance() {
    print_header "TEST 15: Performance - Response Time"
    
    if [ -z "$TEST_TOKEN" ]; then
        print_error "Test token not available"
        return 1
    fi
    
    START_TIME=$(date +%s%N)
    curl -s -X GET "$API_URL/peers" \
        -H "Authorization: Bearer $TEST_TOKEN" > /dev/null
    END_TIME=$(date +%s%N)
    
    RESPONSE_TIME=$(( (END_TIME - START_TIME) / 1000000 ))
    print_info "List peers response time: ${RESPONSE_TIME}ms"
    
    if [ "$RESPONSE_TIME" -lt 1000 ]; then
        print_success "Response time acceptable (< 1000ms)"
    else
        print_error "Response time too slow (> 1000ms)"
        return 1
    fi
}

# Main execution
main() {
    print_header "VPN SERVICE - END-TO-END TEST SUITE"
    
    echo "Testing system at: ${BLUE}$API_URL${NC}"
    echo ""
    
    # Run all tests (continue even if some fail for full report)
    test_services_running || true
    test_database || true
    test_backend_health || true
    test_user_registration || true
    test_user_login || true
    test_list_nodes || true
    test_create_peer || true
    test_get_peer_config || true
    test_list_peers || true
    test_delete_peer || true
    test_unauthorized_error || true
    test_invalid_token_error || true
    test_frontend_build || true
    test_database_integrity || true
    test_performance || true
    
    # Print summary
    print_header "TEST SUMMARY"
    
    TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED))
    PASS_RATE=$((TESTS_PASSED * 100 / TOTAL_TESTS))
    
    echo -e "Total Tests: ${BLUE}$TOTAL_TESTS${NC}"
    echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
    echo -e "Pass Rate: ${BLUE}${PASS_RATE}%${NC}"
    
    echo ""
    if [ "$TESTS_FAILED" -eq 0 ]; then
        echo -e "${GREEN}✓ ALL TESTS PASSED - System is ready!${NC}"
        return 0
    else
        echo -e "${RED}✗ Some tests failed - See details above${NC}"
        return 1
    fi
}

# Run main
main
exit $?
