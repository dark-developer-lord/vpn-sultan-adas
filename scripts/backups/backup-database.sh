#!/bin/bash

# VPN Service Automated Backup Script
# Backs up PostgreSQL database with daily + archived storage
# Scheduled: Daily at 2:00 AM UTC via cron

set -e

# Configuration
BACKUP_DIR="/var/backups/vpn-sultan-adas"
ARCHIVE_DIR="/archive/vpn-backups"
RETENTION_DAYS_LOCAL=7
RETENTION_DAYS_ARCHIVE=30
DB_USER="vpn"
DB_NAME="vpn_service"
DB_HOST="postgres"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
DATE_SHORT=$(date +%Y-%m-%d)
LOG_FILE="/var/log/vpn-backups/backup-${DATE_SHORT}.log"

# Create directories if they don't exist
mkdir -p "$BACKUP_DIR" "$ARCHIVE_DIR" "$(dirname "$LOG_FILE")"

# Logging function
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Error handling
error_exit() {
    log "ERROR: $1"
    exit 1
}

log "=== VPN Service Database Backup Started ==="

# 1. Create backup
BACKUP_FILE="${BACKUP_DIR}/vpn-db-${TIMESTAMP}.sql.gz"
log "Creating backup: $BACKUP_FILE"

if pg_dump -h "$DB_HOST" -U "$DB_USER" "$DB_NAME" | gzip > "$BACKUP_FILE"; then
    SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
    log "✓ Backup created successfully (Size: $SIZE)"
else
    error_exit "Failed to create database backup"
fi

# 2. Create checksum for integrity verification
SHA256=$(sha256sum "$BACKUP_FILE" | awk '{print $1}')
echo "$SHA256" > "${BACKUP_FILE}.sha256"
log "✓ Checksum created: $SHA256"

# 3. Archive old backups to long-term storage
ARCHIVE_FILE="${ARCHIVE_DIR}/vpn-db-${DATE_SHORT}.sql.gz"
if [ ! -f "$ARCHIVE_FILE" ]; then
    cp "$BACKUP_FILE" "$ARCHIVE_FILE"
    cp "${BACKUP_FILE}.sha256" "${ARCHIVE_FILE}.sha256"
    log "✓ Backup archived to long-term storage"
fi

# 4. Clean up old local backups (keep 7 days)
log "Cleaning up backups older than ${RETENTION_DAYS_LOCAL} days..."
find "$BACKUP_DIR" -name "vpn-db-*.sql.gz" -mtime +${RETENTION_DAYS_LOCAL} -delete
find "$BACKUP_DIR" -name "vpn-db-*.sha256" -mtime +${RETENTION_DAYS_LOCAL} -delete
log "✓ Old backups cleaned"

# 5. Clean up archived backups (keep 30 days)
log "Cleaning up archived backups older than ${RETENTION_DAYS_ARCHIVE} days..."
find "$ARCHIVE_DIR" -name "vpn-db-*.sql.gz" -mtime +${RETENTION_DAYS_ARCHIVE} -delete
find "$ARCHIVE_DIR" -name "vpn-db-*.sha256" -mtime +${RETENTION_DAYS_ARCHIVE} -delete
log "✓ Old archived backups cleaned"

# 6. Verify backup integrity
log "Verifying backup integrity..."
if sha256sum -c "${BACKUP_FILE}.sha256" > /dev/null 2>&1; then
    log "✓ Backup integrity verified"
else
    error_exit "Backup integrity check failed"
fi

# 7. Generate backup summary
log "=== Backup Summary ==="
log "Total local backups: $(ls -1 ${BACKUP_DIR}/vpn-db-*.sql.gz 2>/dev/null | wc -l)"
log "Total archived backups: $(ls -1 ${ARCHIVE_DIR}/vpn-db-*.sql.gz 2>/dev/null | wc -l)"
log "Latest backup: $BACKUP_FILE ($(du -h "$BACKUP_FILE" | cut -f1))"

# 8. Send notification
if command -v mail &> /dev/null; then
    echo "VPN Service backup completed successfully on $(date)" | \
        mail -s "✓ VPN Service Backup Report - ${DATE_SHORT}" backup-admin@vpn-service.local
fi

log "=== VPN Service Database Backup Completed ==="
