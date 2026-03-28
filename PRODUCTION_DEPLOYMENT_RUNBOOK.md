# Production Deployment Runbook

## Quick Reference

| Component | Command | Status Check |
|-----------|---------|--------------|
| API | `systemctl start vpn-api` | `curl localhost:8080/health` |
| Database | `systemctl start postgresql` | `pg_isready` |
| Redis | `systemctl start redis` | `redis-cli ping` |
| Nginx | `systemctl start nginx` | `curl localhost/health` |
| Monitoring | `docker-compose -f docker-compose.monitoring.yml up -d` | `curl localhost:9090` |

## Pre-Deployment Checklist

- [ ] All tests pass locally: `cargo test`
- [ ] Docker image builds successfully: `docker build .`
- [ ] No security warnings: `cargo audit`
- [ ] Code reviewed and approved
- [ ] Change request created
- [ ] Team notified on Slack
- [ ] Backup taken: `/usr/local/bin/backup.sh`

## Deployment Steps

### 1. Backup Current State

```bash
# Backup database
/usr/local/bin/backup.sh

# Verify backup
ls -lh /backups/databases/vpn_db_*.sql.gz | head -1

# Backup configuration
cp -r /etc/vpn-api /backups/config/$(date +%Y%m%d_%H%M%S)
```

### 2. Pull Latest Code

```bash
cd /opt/vpn-service
git fetch origin
git checkout v1.2.0  # or main for development
```

### 3. Build Binaries

```bash
cargo build --release
./target/release/vpn-api --version

# Size check
du -h ./target/release/vpn-api
# Expected: ~22MB
```

### 4. Update Configuration

```bash
# Review changes
git diff production.env

# Update environment
sudo systemctl stop vpn-api
cp production.env ~/.env.prod
source ~/.env.prod

# Verify variables
echo $DATABASE_URL
echo $REDIS_URL
```

### 5. Run Database Migrations

```bash
# Before deployment, test migrations
sqlx migrate info --database-url $DATABASE_URL

# Verify migration compatibility
cargo sqlx prepare --database-url $DATABASE_URL

# Apply migrations
sqlx migrate run --database-url $DATABASE_URL

# Verify migration success
sqlite3 $DATABASE_URL "SELECT version FROM _sqlx_migrations;"
```

### 6. Deploy Application

```bash
# Stop old version
sudo systemctl stop vpn-api

# Backup old binary
sudo cp /opt/vpn-api/bin/api /opt/vpn-api/bin/api.backup.v1.1.0

# Copy new binary
sudo cp ./target/release/vpn-api /opt/vpn-api/bin/api

# Set permissions
sudo chown vpn-api:vpn-api /opt/vpn-api/bin/api
sudo chmod 755 /opt/vpn-api/bin/api

# Start new version
sudo systemctl start vpn-api

# Verify startup (wait 5 seconds)
sleep 5
systemctl status vpn-api
```

### 7. Health Checks

```bash
# API health
curl -s http://localhost:8080/health | jq .
# Expected: {"status":"ok","version":"v1.2.0","uptime":"00:00:05"}

# Database connectivity
curl -s http://localhost:8080/api/admin/health | jq .

# Endpoint test
curl -s -H "Authorization: Bearer $TEST_TOKEN" http://localhost:8080/api/users | jq . | head -20

# Response time check
time curl -s http://localhost:8080/health > /dev/null
# Expected: < 100ms
```

### 8. Frontend Deployment (if applicable)

```bash
cd crates/frontend
npm install
npm run build
npm run test

# Copy to web root
sudo cp -r dist/vpn-ui/* /var/www/vpn-app/

# Clear cache
sudo systemctl restart nginx
```

### 9. Monitoring Validation

```bash
# Check Prometheus is scraping
curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets[].labels.job'

# Check Grafana dashboards load
curl -s http://localhost:3000/api/search | jq '.[] | .title' | head -5

# Check alert rules loaded
curl -s http://localhost:9090/api/v1/rules | jq '.data.groups[].name'

# Expected: At least 30 alert rules
curl -s http://localhost:9090/api/v1/rules | jq '.data.groups[].rules | length' | paste -sd+ | bc
```

### 10. Smoke Tests

```bash
# Run automated smoke tests
k6 run tests/smoke-test.js \
  --vus 1 \
  --duration 30s \
  --api-url http://localhost:8080

# Expected: 100% pass rate
```

## Post-Deployment

### Immediate (0-5 minutes)

```bash
# Monitor logs
tail -f /var/log/vpn-api/app.log

# Check for errors
grep ERROR /var/log/vpn-api/app.log

# Monitor metrics
watch -n 2 'curl -s http://localhost:9090/api/v1/query?query=up | jq ".data.result[].value[1]"'

# Check resource usage
watch -n 2 'top -u vpn-api -b | head -10'
```

### Within 30 Minutes

1. **Monitor Error Rates**
   ```bash
   # Should be < 0.5%
   curl -s 'http://localhost:9090/api/v1/query?query=rate(http_requests_total%7Bstatus=~"5.."%7D%5B5m%5D)' | jq .
   ```

2. **Monitor Latency**
   ```bash
   # P95 should be < 500ms
   curl -s 'http://localhost:9090/api/v1/query?query=histogram_quantile(0.95,http_request_duration_seconds_bucket)' | jq .
   ```

3. **Check Database Connections**
   ```bash
   # Should not exceed 20
   psql -c "SELECT count(*) FROM pg_stat_activity;"
   ```

4. **Review Logs**
   ```bash
   # No FATAL or ERROR
   grep -E "FATAL|ERROR" /var/log/vpn-api/app.log | wc -l
   # Expected: 0
   ```

### Within Hours

1. **Run Load Test**
   ```bash
   k6 run tests/load-test.js \
     --vus 100 \
     --duration 5m \
     --api-url http://localhost:8080
   ```

2. **Monitor Business Metrics**
   ```bash
   # Check active sessions
   curl -s http://localhost:8080/api/admin/stats | jq '.activeUsers'
   
   # Check successful transactions
   curl -s 'http://localhost:9090/api/v1/query?query=rate(payments_total%5B1h%5D)' | jq .
   ```

3. **Verify Backups**
   ```bash
   # Next backup should be scheduled
   crontab -l | grep backup
   
   # Test restore (use copy of production DB)
   psql -c "CREATE DATABASE vpn_db_restore;"
   gunzip -c /backups/databases/vpn_db_latest.sql.gz | psql vpn_db_restore
   ```

## Rollback Procedure

### If Critical Issues Detected

```bash
# Stop new version
sudo systemctl stop vpn-api

# Revert binary
sudo cp /opt/vpn-api/bin/api.backup.v1.1.0 /opt/vpn-api/bin/api
sudo systemctl start vpn-api

# Verify rollback
sleep 5
curl http://localhost:8080/health

# Revert database (if migrations caused issues)
# WARNING: This will lose data since last backup
# Only execute with approval from DBA
# psql -f /backups/migrations/vpn_db_v1.1.0_schema.sql
```

### Database Rollback

```bash
# If migrations caused issue
# 1. Stop application
sudo systemctl stop vpn-api

# 2. Restore from backup
sudo -u postgres pg_restore -d vpn_db /backups/databases/vpn_db_2024_01_15_14_30.sql.gz

# 3. Verify restoration
psql -c "SELECT version FROM _sqlx_migrations ORDER BY installed_on DESC LIMIT 1;"

# 4. Restart with previous version
sudo systemctl start vpn-api
```

## Monitoring Dashboard Links

- **API Performance**: http://grafana.internal/d/api-performance
- **Infrastructure**: http://grafana.internal/d/infrastructure
- **Database**: http://grafana.internal/d/database
- **Security**: http://grafana.internal/d/security
- **Alerts**: http://alertmanager.internal/

## Logs Location

```bash
# Application logs
/var/log/vpn-api/app.log
/var/log/vpn-api/error.log

# System logs
/var/log/syslog
journalctl -u vpn-api -f

# Database logs
/var/log/postgresql/postgresql.log

# Nginx logs
/var/log/nginx/access.log
/var/log/nginx/error.log
```

## Emergency Contacts

| Role | Name | Phone | Slack |
|------|------|-------|-------|
| On-Call Engineer | TBD | +1-555-0100 | @oncall |
| DevOps Lead | TBD | +1-555-0101 | @devops-lead |
| Database Admin | TBD | +1-555-0102 | @dba |
| Infrastructure | TBD | +1-555-0103 | @infrastructure |

## Incident Response

### If Deployment Fails

1. Check logs:
   ```bash
   journalctl -u vpn-api -n 50 -e
   ```

2. Verify configuration:
   ```bash
   cat /etc/vpn-api/production.env
   ```

3. Check dependencies:
   ```bash
   systemctl status postgresql redis-server nginx
   ```

4. If unrecoverable, roll back:
   ```bash
   ./scripts/rollback.sh v1.1.0
   ```

### If Performance Degrades Post-Deployment

1. Identify the issue:
   ```bash
   # High CPU?
   top -u vpn-api
   
   # High memory?
   ps aux | grep vpn-api
   
   # Database slow?
   psql -c "SELECT query, mean_exec_time FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 5;"
   ```

2. Temporary mitigation:
   ```bash
   # Scale up if in Kubernetes
   kubectl scale deployment vpn-api --replicas=5
   
   # Restart if memory leak suspected
   sudo systemctl restart vpn-api
   
   # Clear cache if performance issue
   redis-cli FLUSHALL
   redis-cli -n 0 FLUSHALL  # Only production cache
   ```

3. Escalate if not resolved in 10 minutes

## Success Criteria

✅ Deployment successful if after 1 hour:
- Error rate < 0.5%
- P95 latency within 10% of baseline
- Zero database issues
- All health checks pass
- No customer complaints
- Monitoring shows normal traffic patterns

## Post-Deployment Report

```markdown
# Deployment Report - v1.2.0

**Date/Time**: 2024-01-15 14:30:00 UTC
**Duration**: 15 minutes
**Status**: ✅ SUCCESS

## Metrics Before/After

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Error Rate | 0.02% | 0.01% | ✅ -50% |
| P95 Latency | 45ms | 42ms | ✅ -7% |
| Throughput | 2500 req/s | 2650 req/s | ✅ +6% |
| Memory | 512MB | 520MB | ⚠️ +1.6% |

## Issues Encountered

None

## Rollback Status

Not required

## Next Steps

- Monitor for 24 hours
- Schedule retrospective if any issues
- Archive deployment artifacts
```

## Related Documents

- [Canary Deployment Strategy](CANARY_DEPLOYMENT.md)
- [Incident Response Guide](INCIDENT_RESPONSE.md)
- [Database Migration Guide](DATABASE_MIGRATIONS.md)
- [Monitoring Setup](MONITORING_SETUP.md)
