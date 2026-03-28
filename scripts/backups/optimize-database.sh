#!/bin/bash

# Database Optimization Script
# Runs database maintenance tasks for performance
# - Create indexes on frequently used columns
# - Analyze query performance
# - Vacuum and optimize tables
# - Generate statistics

set -e

DB_USER="vpn"
DB_NAME="vpn_service"
DB_HOST="postgres"
LOG_FILE="/var/log/vpn-db-optimization-$(date +%Y-%m-%d).log"

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

log "=== PostgreSQL Database Optimization Started ==="

# Connect to database
psql_cmd="psql -h $DB_HOST -U $DB_USER -d $DB_NAME"

# 1. Create missing indexes
log "Creating performance indexes..."

$psql_cmd << EOF
-- User-related indexes
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);

-- Subscription indexes
CREATE INDEX IF NOT EXISTS idx_subscriptions_user_id ON subscriptions(user_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_status ON subscriptions(status);
CREATE INDEX IF NOT EXISTS idx_subscriptions_created_at ON subscriptions(created_at DESC);

-- VPN connection indexes
CREATE INDEX IF NOT EXISTS idx_vpn_connections_user_id ON vpn_connections(user_id);
CREATE INDEX IF NOT EXISTS idx_vpn_connections_server_id ON vpn_connections(server_id);
CREATE INDEX IF NOT EXISTS idx_vpn_connections_connected_at ON vpn_connections(connected_at DESC);

-- Authentication indexes
CREATE INDEX IF NOT EXISTS idx_auth_logs_user_id ON auth_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_auth_logs_created_at ON auth_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_auth_logs_status ON auth_logs(status);

-- Payment indexes
CREATE INDEX IF NOT EXISTS idx_payments_subscription_id ON payments(subscription_id);
CREATE INDEX IF NOT EXISTS idx_payments_status ON payments(status);
CREATE INDEX IF NOT EXISTS idx_payments_created_at ON payments(created_at DESC);

-- Audit log indexes
CREATE INDEX IF NOT EXISTS idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_action ON audit_log(action);
CREATE INDEX IF NOT EXISTS idx_audit_log_created_at ON audit_log(created_at DESC);
EOF

log "✓ Indexes created"

# 2. Analyze query performance
log "Analyzing query performance..."
$psql_cmd << EOF
ANALYZE;
EOF
log "✓ Query analysis complete"

# 3. Vacuum tables
log "Vacuuming tables..."
$psql_cmd << EOF
VACUUM ANALYZE;
EOF
log "✓ Vacuum complete"

# 4. Show table sizes
log "Current table sizes:"
$psql_cmd << EOF
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
EOF

# 5. Show indexes
log "Database indexes created:"
$psql_cmd << EOF
SELECT
    schemaname,
    tablename,
    indexname,
    pg_size_pretty(pg_relation_size(indexrelid)) AS size
FROM pg_indexes
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
ORDER BY pg_relation_size(indexrelid) DESC;
EOF

# 6. Check cache hit ratio
log "Checking cache hit ratio..."
$psql_cmd << EOF
SELECT
    sum(heap_blks_read) as heap_read,
    sum(heap_blks_hit) as heap_hit,
    sum(heap_blks_hit) / (sum(heap_blks_hit) + sum(heap_blks_read)) as cache_ratio
FROM pg_statio_user_tables;
EOF

# 7. Show slow queries (if any)
log "Checking for slow queries..."
$psql_cmd << EOF
SELECT
    query,
    calls,
    mean_time,
    total_time
FROM pg_stat_statements
WHERE db = (SELECT datname FROM pg_database WHERE datname = current_database())
ORDER BY mean_time DESC
LIMIT 10;
EOF

log "=== PostgreSQL Database Optimization Completed ==="
