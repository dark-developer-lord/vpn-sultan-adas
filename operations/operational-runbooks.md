# VPN Service Operational Runbooks

Complete procedures for routine maintenance, deployments, and operational tasks.

---

## 🚀 Deployment Runbook

### Pre-Deployment Checklist

- [ ] Git branch is clean: `git status`
- [ ] All tests passing locally: `bash scripts/smoke-tests.sh`
- [ ] Code review completed and approved
- [ ] No outstanding security vulnerabilities: `docker scan`
- [ ] Database backup exists: `ls -lh /var/backups/vpn-db-*.sql.gz`
- [ ] Maintenance window scheduled (if needed)
- [ ] Team notified of deployment time

### Deployment Procedure

1. **Create release branch**:
   ```bash
   git checkout -b release/v1.X.Y main
   ```

2. **Tag the release**:
   ```bash
   git tag -a v1.X.Y -m "Release version 1.X.Y"
   git push origin v1.X.Y
   ```

3. **Build new images** (on VPS):
   ```bash
   cd ~/vpn-sultan-adas
   git fetch origin
   git checkout main
   docker-compose build --no-cache vpn-api
   ```

4. **Backup before changes**:
   ```bash
   bash scripts/backups/backup-database.sh
   ```

5. **Deploy with health checks**:
   ```bash
   # Remove old container
   docker-compose stop vpn-api
   docker rm vpn-api
   
   # Start new version
   docker-compose up -d vpn-api
   
   # Wait for startup
   sleep 5
   
   # Health check
   curl -f http://localhost:3000/health || (
       docker-compose down
       git checkout HEAD~1
       docker-compose build vpn-api
       docker-compose up -d vpn-api
       exit 1
   )
   ```

6. **Run smoke tests**:
   ```bash
   bash scripts/smoke-tests.sh
   ```

7. **Monitor for 10 minutes**:
   ```bash
   watch -n 2 'curl -s http://localhost:3000/metrics | grep http_errors_total'
   ```

### Rollback Procedure

If deployment fails and health checks pass but metrics show issues:

```bash
# Revert to previous version
git checkout HEAD~1
docker-compose build --no-cache vpn-api
docker-compose up -d vpn-api
sleep 5
curl http://localhost:3000/health

# Run tests
bash scripts/smoke-tests.sh

# Monitor for issues
docker-compose logs -f vpn-api
```

---

## 💾 Database Backup & Recovery

### Automated Daily Backup (Already Scheduled)

Backup script runs daily at 2:00 AM UTC via cron:
```bash
0 2 * * * /root/vpn-sultan-adas/scripts/backups/backup-database.sh
```

**Backup verification**:
```bash
# Check backup exists
ls -lh /var/backups/vpn-db-*.sql.gz | tail -5

# Verify backup integrity
for file in /var/backups/vpn-db-*.sql.gz; do
  sha256sum -c "${file}.sha256" && echo "✓ $file OK" || echo "✗ $file FAILED"
done
```

### Manual Backup

```bash
cd ~/vpn-sultan-adas
docker-compose exec postgres pg_dump -U vpn vpn_service | gzip > backup-manual-$(date +%Y%m%d-%H%M%S).sql.gz
```

### Point-in-Time Recovery

**Step 1: List available backups**:
```bash
ls -lh /var/backups/vpn-db-*.sql.gz
ls -lh /archive/vpn-backups/vpn-db-*.sql.gz  # Archived backups
```

**Step 2: Stop dependent services**:
```bash
docker-compose stop vpn-api
docker-compose stop pgadmin
```

**Step 3: Drop and recreate database**:
```bash
docker-compose exec postgres dropdb -U vpn vpn_service
docker-compose exec postgres createdb -U vpn vpn_service
```

**Step 4: Restore from backup**:
```bash
zcat /var/backups/vpn-db-2024-01-15.sql.gz | docker-compose exec -T postgres psql -U vpn vpn_service
```

**Step 5: Restart services**:
```bash
docker-compose up -d vpn-api pgadmin
sleep 5
curl http://localhost:3000/health
```

**Step 6: Verify data integrity**:
```bash
bash scripts/smoke-tests.sh
```

### On-Demand Full Export

For long-term archival:

```bash
docker-compose exec postgres pg_dump -U vpn vpn_service \
  --format=custom \
  --verbose \
  --file=/backups/vpn-full-export-$(date +%Y%m%d).dump

# This file can be restored with:
# docker-compose exec postgres pg_restore -U vpn -d vpn_service /backups/vpn-full-export-20240115.dump
```

---

## ⚡ Database Optimization Runbook

### Automated Weekly Optimization (Already Scheduled)

Optimization script runs weekly on Sunday at 3:00 AM UTC:
```bash
0 3 * * 0 /root/vpn-sultan-adas/scripts/backups/optimize-database.sh
```

### Manual Optimization

Run during low-traffic periods:

```bash
bash ~/vpn-sultan-adas/scripts/backups/optimize-database.sh
```

**What it does**:
- Creates performance indexes on key columns
- Runs VACUUM ANALYZE for garbage collection
- Analyzes cache hit ratios
- Identifies slow queries
- Reports table sizes

### Index Management

**View all indexes**:
```bash
docker-compose exec postgres psql -U vpn -d vpn_service -c "\di"
```

**Check unused indexes**:
```bash
docker-compose exec postgres psql -U vpn -d vpn_service <<EOF
SELECT 
  t.tablename,
  indexname,
  idx_scan
FROM pg_indexes i
JOIN pg_stat_user_indexes s ON i.indexname = s.relname
JOIN pg_tables t ON i.tablename = t.tablename
WHERE idx_scan = 0
ORDER BY pg_relation_size(i.indexname) DESC;
EOF
```

**Drop unused index**:
```bash
docker-compose exec postgres psql -U vpn -d vpn_service -c "DROP INDEX IF EXISTS index_name;"
```

### Query Performance Analysis

**Find slow queries** (over 1 second):
```bash
docker-compose exec postgres psql -U vpn -d vpn_service -c "
SELECT query, calls, total_time, mean_time 
FROM pg_stat_statements 
WHERE mean_time > 1000 
ORDER BY mean_time DESC 
LIMIT 10;"
```

**Explain query plan**:
```bash
docker-compose exec postgres psql -U vpn -d vpn_service -c "
EXPLAIN ANALYZE
SELECT * FROM users WHERE email = 'test@example.com';"
```

---

## 🔒 Security Maintenance

### SSL Certificate Renewal (Let's Encrypt)

Certificates auto-renew 30 days before expiry with certbot:

```bash
# Manual renewal (optional)
docker-compose exec nginx certbot renew

# Check certificate expiry
docker-compose exec nginx openssl x509 -enddate -noout -in /etc/letsencrypt/live/vpn.service/fullchain.pem
```

### Password Rotation

Every 90 days, rotate database password:

```bash
# Generate new password
NEW_PASS=$(openssl rand -base64 32)

# Update PostgreSQL
docker-compose exec postgres psql -U vpn -d vpn_service -c "ALTER USER vpn WITH PASSWORD '$NEW_PASS';"

# Update docker-compose.yml with new password
# Restart services
docker-compose restart
```

### Security Headers Verification

Verify security headers are being sent:

```bash
curl -i http://localhost:3000/health | grep -i "X-"

# Expected headers:
# X-Content-Type-Options: nosniff
# X-Frame-Options: DENY
# X-XSS-Protection: 1; mode=block
# Strict-Transport-Security: max-age=31536000
# Content-Security-Policy: default-src 'self'
```

### Dependency Vulnerability Scanning

```bash
docker scan vpn-api
docker scan vpn-postgres
```

Review reports and patch vulnerabilities if any are found.

---

## 📊 Monitoring & Observability

### Check Prometheus Health

```bash
# Verify Prometheus is scraping targets
curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets | length'

# Should return 8+ targets (api, postgres, redis, alertmanager, exporters)

# Query active metrics
curl -s 'http://localhost:9090/api/v1/query?query=up' | jq '.data.result'
```

### Check Grafana Dashboards

Access via: `http://187.124.179.20:3001`

**Default credentials**:
- Username: admin
- Password: admin (change on first login!)

**Key dashboards to verify**:
- Node Exporter (System metrics)
- PostgreSQL (Database metrics)
- Redis (Cache metrics)
- API Service (Request metrics)

### Check Loki Logs

Query recent errors:
```bash
# Via curl (for scripting)
curl -s 'http://localhost:3100/loki/api/v1/query?query={job="vpn-api"}%20|%20json%20|%20status%26gt;%3D500' | jq '.data.result'
```

Or manually in Grafana:
- Click "Explore" 
- Select Loki datasource
- Query: `{job="vpn-api"} | json | status >= 500`

### Check AlertManager

```bash
# List current alerts
curl http://localhost:9093/api/v1/alerts | jq '.data'

# List all alert rules
curl http://localhost:9093/api/v1/rules | jq '.data'
```

---

## 🔄 Service Maintenance Windows

### Schedule Maintenance

```bash
# Create maintenance mode (if deployed with nginx)
docker-compose exec nginx echo "Service maintenance in progress" > /var/www/html/maint.html
docker-compose exec nginx sed -i 's/server {/server { error_page 503 \/maint.html;\n  location \/ { return 503; }/' /etc/nginx/conf.d/default.conf
docker-compose exec nginx nginx -s reload
```

### Perform Maintenance

While in maintenance mode:
- Run backups (already happening)
- Optimize database (already happening)
- Update dependencies
- Test changes

### Resume Service

```bash
# Exit maintenance mode
docker-compose exec nginx sed -i '/error_page 503/d; /return 503;/d' /etc/nginx/conf.d/default.conf
docker-compose exec nginx nginx -s reload
```

---

## 📈 Capacity Planning

### Monitor Resource Usage

```bash
# Real-time resource stats
watch docker stats

# Weekly capacity report
docker stats --no-stream --format 'table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}'
```

### Disk Space Planning

Current usage trends:
```bash
df -h / | awk 'NR==2 {print $5}'  # Percentage used

# If > 80%, plan expansion:
# - Archive old backups to S3
# - Increase volume size
# - Add second volume
```

### Database Size Monitoring

```bash
docker-compose exec postgres psql -U vpn -d vpn_service -c "
SELECT 
  schemaname,
  tablename,
  pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename))
FROM pg_tables
WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;"
```

**Action items if > 10GB**: Consider archiving old data or partitioning large tables.

---

## 🔧 Common Operational Tasks

### Add New Cron Job

1. Create script in `scripts/`
2. Add execution permissions: `chmod +x script.sh`
3. Edit crontab:
   ```bash
   crontab -e
   # Add line: 0 2 * * * /root/vpn-sultan-adas/scripts/path/to/script.sh
   ```
4. Verify: `crontab -l`

### View Service Logs

```bash
# Last 100 lines of API logs
docker-compose logs -n 100 vpn-api

# Stream logs in real-time
docker-compose logs -f vpn-api

# Logs from specific time
docker-compose logs --since 2024-01-15T10:00:00 vpn-api
```

### Restart Services Without Downtime

```bash
# Graceful restart (connections drain)
docker-compose stop vpn-api  # Wait for requests to complete
sleep 5
docker-compose up -d vpn-api
```

### Update System Packages

```bash
apt update
apt upgrade -y
apt autoremove -y

# If kernel updated, reboot required
reboot
```

---

## 📋 Daily Operational Checklist

Run this every morning:

```bash
#!/bin/bash
set -e

echo "=== VPN Service Daily Operations Check ==="
date

echo "1. Checking service health..."
curl -f http://localhost:3000/health || exit 1

echo "2. Verifying containers running..."
docker-compose ps | grep -E "vpn-|postgres|redis"

echo "3. Checking recent errors..."
docker-compose logs --since 1h vpn-api | grep -i error | tail -5 || echo "  ✓ No errors in last hour"

echo "4. Verifying recent backup exists..."
ls -lh /var/backups/vpn-db-*.sql.gz | head -1

echo "5. Checking disk space..."
df -h / | awk 'NR==2 {if ($5 ~ /^([89][0-9]|100)/) print "  ⚠️  WARNING: " $5 " used"; else print "  ✓ " $5 " used"}'

echo "6. Checking Prometheus scraping..."
TARGETS=$(curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets | length')
echo "  ✓ $TARGETS targets active"

echo "7. Checking for active alerts..."
ALERTS=$(curl -s http://localhost:9093/api/v1/alerts | jq '.data | length')
if [ "$ALERTS" -gt 0 ]; then
  echo "  ⚠️  $ALERTS active alerts"
  curl -s http://localhost:9093/api/v1/alerts | jq '.data[].labels.alertname'
else
  echo "  ✓ No active alerts"
fi

echo ""
echo "✅ Daily check complete!"
```

Save as `scripts/daily-check.sh` and add to crontab:
```bash
0 8 * * * /root/vpn-sultan-adas/scripts/daily-check.sh >> /var/log/vpn-daily-check.log 2>&1
```

---

**Last Updated**: March 28, 2026
**Owner**: Operations Team
**Review Frequency**: Quarterly
