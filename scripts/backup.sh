#!/bin/bash
# VPN Service Database Backup Script
# Run via cron: 0 2 * * * /opt/vpn-service/scripts/backup.sh

set -e

# Configuration
BACKUP_DIR="/data/backups"
RETENTION_DAYS=30
DB_USER="vpn"
DB_PASSWORD="${DB_PASSWORD:?Error: DB_PASSWORD not set}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="vpn_service"
S3_BUCKET="${S3_BUCKET:?Error: S3_BUCKET not set}"
AWS_REGION="${AWS_REGION:-us-east-1}"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/vpn_service_$TIMESTAMP.sql.gz"
LOG_FILE="/var/log/vpn-backups.log"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Logging function
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

log_message "Starting database backup..."

# Create backup
export PGPASSWORD="$DB_PASSWORD"
if pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" "$DB_NAME" | gzip > "$BACKUP_FILE"; then
    BACKUP_SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
    log_message "✅ Database backup created: $BACKUP_FILE ($BACKUP_SIZE)"
else
    log_message "❌ Database backup failed"
    exit 1
fi

# Upload to S3
log_message "Uploading backup to S3..."
if aws s3 cp "$BACKUP_FILE" "s3://$S3_BUCKET/$(date +%Y/%m)/$TIMESTAMP.sql.gz" \
    --region "$AWS_REGION" \
    --sse AES256 \
    --storage-class STANDARD_IA; then
    log_message "✅ Backup uploaded to S3"
else
    log_message "❌ S3 upload failed"
    exit 1
fi

# Cleanup local backups older than retention days
log_message "Cleaning up old backups (retention: $RETENTION_DAYS days)..."
find "$BACKUP_DIR" -name "vpn_service_*.sql.gz" -mtime +$RETENTION_DAYS -delete
log_message "✅ Cleanup completed"

# Verify backup integrity
log_message "Verifying backup integrity..."
if gunzip -t "$BACKUP_FILE" 2>/dev/null; then
    log_message "✅ Backup integrity verified"
else
    log_message "❌ Backup integrity check failed"
    exit 1
fi

log_message "Backup completed successfully"
