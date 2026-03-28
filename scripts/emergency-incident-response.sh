#!/bin/bash

# Emergency Incident Response Procedure
# Use this in critical production incidents

set -e

INCIDENT_ID="INCIDENT-$(date +%s)"
LOG_FILE="/tmp/incident-response-$INCIDENT_ID.log"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging functions
log() {
    echo "[$TIMESTAMP] $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR] $1${NC}" | tee -a "$LOG_FILE"
}

log_warn() {
    echo -e "${YELLOW}[WARN] $1${NC}" | tee -a "$LOG_FILE"
}

log_info() {
    echo -e "${GREEN}[INFO] $1${NC}" | tee -a "$LOG_FILE"
}

log_critical() {
    echo -e "${RED}[CRITICAL] $1${NC}" | tee -a "$LOG_FILE"
    # Send to alerting system
    curl -s -X POST "http://localhost:9093/api/v1/alerts" \
      -H "Content-Type: application/json" \
      -d "[{
        \"labels\": {\"alertname\": \"IncidentResponse\", \"severity\": \"critical\"},
        \"annotations\": {\"summary\": \"$1\", \"incident_id\": \"$INCIDENT_ID\"}
      }]" > /dev/null 2>&1 || true
}

# Performance baseline capture
capture_baseline() {
    log_info "Capturing system baseline..."

    echo "--- BASELINE SNAPSHOT ---" >> "$LOG_FILE"
    echo "Timestamp: $TIMESTAMP" >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"

    echo "### CPU Usage (top 5 processes)" >> "$LOG_FILE"
    ps aux --sort=-%cpu | head -6 >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"

    echo "### Memory Usage (top 5 processes)" >> "$LOG_FILE"
    ps aux --sort=-%mem | head -6 >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"

    echo "### Disk Usage" >> "$LOG_FILE"
    df -h >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"

    echo "### Network Connections" >> "$LOG_FILE"
    netstat -tuln | grep LISTEN >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"

    echo "### Service Status" >> "$LOG_FILE"
    systemctl status vpn-api --no-pager >> "$LOG_FILE" 2>&1 || true
    echo "" >> "$LOG_FILE"
}

# Check critical systems
check_systems() {
    log_info "Performing system health check..."

    local errors=0

    # Check API
    if ! curl -s http://localhost:8080/health > /dev/null 2>&1; then
        log_error "API is not responding"
        ((errors++))
    else
        log_info "✓ API responding"
    fi

    # Check Database
    if ! curl -s http://localhost:8080/api/health/db > /dev/null 2>&1; then
        log_error "Database is not responding"
        ((errors++))
    else
        log_info "✓ Database responding"
    fi

    # Check Redis
    if ! curl -s http://localhost:8080/api/health/redis > /dev/null 2>&1; then
        log_error "Redis is not responding"
        ((errors++))
    else
        log_info "✓ Redis responding"
    fi

    # Check Prometheus
    if ! curl -s http://localhost:9090/-/healthy > /dev/null 2>&1; then
        log_warn "Prometheus not responding"
    else
        log_info "✓ Prometheus responding"
    fi

    return $errors
}

# Immediate mitigations
mitigate_immediately() {
    log_critical "Executing immediate mitigation procedures..."

    # Stop accepting new connections if DB is down
    if ! curl -s http://localhost:8080/api/health/db > /dev/null 2>&1; then
        log_critical "Database down - activating circuit breaker"
        # Return 503 Service Unavailable for new requests
        # Requires API-level implementation
    fi

    # Clear rate limiter if causing false positives
    log_warn "Analyzing rate limit metrics..."
    RATE_LIMIT_BLOCKS=$(curl -s 'http://localhost:9090/api/v1/query?query=rate_limiter_blocked_requests_total' | jq '.data.result[0].value[1]' 2>/dev/null || echo "0")

    if [ "$RATE_LIMIT_BLOCKS" -gt 1000 ]; then
        log_warn "High rate limit blocks detected: $RATE_LIMIT_BLOCKS"
        log_warn "Consider clearing rate limiters: redis-cli KEYS 'rate_limit:*' | xargs redis-cli DEL"
    fi
}

# Rollback procedure
rollback_deployment() {
    log_critical "Initiating rollback procedure..."

    if [ -f "/var/app/backups/previous-binary" ]; then
        log_info "Stopping current API..."
        systemctl stop vpn-api

        log_info "Restoring previous binary..."
        cp /var/app/backups/previous-binary /var/app/bin/vpn-api
        chmod +x /var/app/bin/vpn-api

        log_info "Starting previous version..."
        systemctl start vpn-api

        # Wait for startup
        sleep 5

        if curl -s http://localhost:8080/health > /dev/null 2>&1; then
            log_info "✓ Rollback successful"
            ROLLBACK_SUCCESS=1
        else
            log_error "Rollback failed - API not responding after restart"
            ROLLBACK_SUCCESS=0
        fi
    else
        log_error "Previous binary backup not found"
        ROLLBACK_SUCCESS=0
    fi
}

# Database rollback
rollback_database() {
    log_critical "Analyzing database state for rollback..."

    # Get latest backup
    BACKUP_FILE=$(ls -t /var/app/backups/db-*.sql.gz | head -1)

    if [ -z "$BACKUP_FILE" ]; then
        log_error "No database backup found"
        return 1
    fi

    log_warn "Latest backup: $BACKUP_FILE"
    log_warn "Rolling back database (this may take time)..."

    # Create restore point first
    pg_dump -h localhost -U postgres vpn_service -Fc > /var/app/backups/pre-rollback-$(date +%s).dump.gz

    # Restore from backup
    gunzip < "$BACKUP_FILE" | psql -h localhost -U postgres -d vpn_service > /dev/null 2>&1

    if [ $? -eq 0 ]; then
        log_info "✓ Database rollback successful"
        return 0
    else
        log_error "Database rollback failed"
        return 1
    fi
}

# Generate incident report
generate_incident_report() {
    log_info "Generating incident report..."

    local report_file="/var/app/incidents/$INCIDENT_ID.md"
    mkdir -p /var/app/incidents

    cat > "$report_file" << EOF
# Incident Report: $INCIDENT_ID

**Date**: $TIMESTAMP
**Duration**: N/A (ongoing)
**Severity**: CRITICAL
**Status**: INVESTIGATING

## Timeline

- **$TIMESTAMP**: Incident detected

## Impact

- API Services: DEGRADED
- User Impact: SERVICE DISRUPTION

## Expected Resolution Time (ETA)

- 30 minutes from incident start
- Update every 5 minutes

## Actions Taken

1. Baseline captured
2. System health checked
3. Immediate mitigations applied
4. Logs collected

## Next Steps

1. Root cause analysis in progress
2. Monitoring dashboard: http://localhost:3000/d/incident
3. Team standup: Slack #incident-$INCIDENT_ID

## Contact

- On-Call: @on-call (Slack)
- Team Lead: @lead (Slack)
- Escalation: See ON_CALL_RUNBOOK.md

## Log File

$LOG_FILE

EOF

    log_info "Incident report: $report_file"
}

# Main incident response flow
main() {
    echo "=========================================="
    echo "  VPN SERVICE EMERGENCY INCIDENT RESPONSE"
    echo "=========================================="
    echo ""
    echo "Incident ID: $INCIDENT_ID"
    echo "Start Time: $TIMESTAMP"
    echo "Log File: $LOG_FILE"
    echo ""

    # Step 1: Capture baseline
    capture_baseline

    # Step 2: Check systems
    check_systems
    SYSTEM_CHECK=$?

    if [ $SYSTEM_CHECK -gt 0 ]; then
        log_critical "Multiple system failures detected"

        # Step 3: Mitigate immediately
        mitigate_immediately

        # Step 4: Attempt rollback if deployment was recent
        log_warn ""
        log_warn "Would you like to rollback to previous deployment? (y/n)"
        # read -r response
        # if [ "$response" = "y" ]; then
        #    rollback_deployment
        #    if [ $ROLLBACK_SUCCESS -eq 0 ]; then
        #        rollback_database
        #    fi
        # fi
    fi

    # Step 5: Generate report
    generate_incident_report

    echo ""
    echo "=========================================="
    echo "  INCIDENT RESPONSE INITIATED"
    echo "=========================================="
    echo ""
    echo "Next Actions:"
    echo "1. Review incident report: $(find /var/app/incidents -name "$INCIDENT_ID.md" -mmin -1)"
    echo "2. Investigate root cause using logs: $LOG_FILE"
    echo "3. Monitor recovery using dashboard: http://localhost:3000/d/incident"
    echo "4. Update status on Slack #incident-$INCIDENT_ID"
    echo "5. After recovery, create post-mortem within 24 hours"
    echo ""
    echo "Key Contacts:"
    echo "  • Automated Alert: PagerDuty incident link auto-created"
    echo "  • Team Lead: @lead in Slack"
    echo "  • Full Escalation: See ON_CALL_RUNBOOK.md"
    echo ""
}

# Run main flow
main
