# Disaster Recovery Procedure - VPN Service

## Overview

This document outlines the procedures for recovering from various disaster scenarios affecting the VPN Service platform. All procedures have defined Recovery Time Objective (RTO) and Recovery Point Objective (RPO) targets.

---

## 1. CRITICAL SERVICE FAILURES

### 1.1 Database Failure (RTO: 1 hour, RPO: 15 minutes)

#### Symptoms
- Database connection errors in logs
- {"error": "database connection failed"}
- Unable to authenticate users
- Slow/unresponsive API requests

#### Recovery Steps

**Step 1: Assess the Situation**
```bash
# Check database connectivity
docker exec vpn-db pg_isready -h localhost

# Check database logs
docker logs vpn-db | tail -100

# Verify disk space
docker exec vpn-db df -h
```

**Step 2: Attempt Local Recovery**
```bash
# Restart database container
docker-compose restart db

# Wait 30 seconds
sleep 30

# Test connection
psql -h localhost -U vpn_user -d vpn_db -c "SELECT 1"
```

**Step 3: If Local Recovery Fails - Restore from Backup**
```bash
# Connect to backup storage (S3/Azure)
aws s3 ls s3://vpn-backups/

# List available database backups
aws s3 ls s3://vpn-backups/database/

# Download latest backup
aws s3 cp s3://vpn-backups/database/vpn_db_backup_2024-01-15_02-00.sql.gz .

# Decompress backup
gunzip vpn_db_backup_2024-01-15_02-00.sql.gz

# Restore from backup
cat vpn_db_backup_2024-01-15_02-00.sql | \
  docker exec -i vpn-db psql -U vpn_user -d vpn_db

# Verify restore
docker exec vpn-db psql -U vpn_user -d vpn_db \
  -c "SELECT COUNT(*) FROM users"
```

**Step 4: Verify System Health**
```bash
# Check user count
curl -H "Authorization: Bearer $ADMIN_TOKEN" \
  http://localhost:8080/admin/dashboard

# Run health checks
curl http://localhost:8080/health
```

**Step 5: Update Backup References** (if needed)
```bash
# If using earlier backup, determine data loss
# Notify users of potential data loss from last backup
# Document in incident report
```

---

### 1.2 API Server Crash (RTO: 5 minutes, RPO: real-time)

#### Symptoms
- All API requests return 502/503
- API container exited
- Application logs show panic/critical error

#### Recovery Steps

**Step 1: Check Container Status**
```bash
# Check if backend running
docker ps | grep vpn-api

# View recent logs
docker logs vpn-api --tail 200
```

**Step 2: Restart Service**
```bash
# Restart the API container
docker-compose restart api

# Wait for startup (30-60 seconds)
sleep 60

# Verify health endpoint
curl http://localhost:8080/health
```

**Step 3: If Container Won't Start**
```bash
# Check error logs
docker logs vpn-api

# View resource constraints
docker inspect vpn-api | grep -i memory

# If memory issue: increase Docker allocation
# If compilation error: rebuild image
docker-compose build api
docker-compose up -d api
```

**Step 4: Verify All Services**
```bash
# Check all containers running
docker-compose ps

# Test API endpoints
curl http://localhost:8080/auth/login -X POST \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@test.com","password":"test"}'
```

**Step 5: If Multiple Instances**
```bash
# If using load balancer, verify failover
# Check that traffic shifting occurred
# Monitor remaining instance health

# Once recovered, rebalance traffic
docker-compose up -d --scale api=2
```

---

### 1.3 Redis Cache Failure (RTO: 30 minutes, RPO: 0)

#### Symptoms
- Cache misses on every request
- Redis connection errors
- Slow API responses even with database up

#### Recovery Steps

**Step 1: Check Redis Service**
```bash
# Test Redis connectivity
redis-cli -h localhost -p 6379 ping

# Check Redis info
redis-cli info stats
```

**Step 2: Restart Redis**
```bash
docker-compose restart redis

sleep 10

# Verify connectivity
redis-cli ping
```

**Step 3: Warm Up Cache**
```bash
# Preload frequently accessed data
curl -X POST -H "Authorization: Bearer $ADMIN_TOKEN" \
  http://localhost:8080/admin/cache/warm-up
```

**Step 4: Monitor Performance**
```bash
# Check cache hit rate
curl http://localhost:8080/metrics | grep cache_hit_rate

# Should target 85%+ hit rate
```

**Note:** Redis cache loss results in degraded performance but NOT data loss. All data exists in the database.

---

## 2. DATA LOSS SCENARIOS

### 2.1 Accidental Data Deletion (RTO: 2 hours, RPO: 1 day)

#### Scenario
User or admin accidentally deletes important data (users, VPN configs, subscriptions).

#### Recovery Steps

**Step 1: Identify Scope**
```bash
# Query audit logs for deletion
SELECT * FROM audit_logs 
  WHERE event_type = 'DELETE' 
  AND created_at > NOW() - INTERVAL '1 hour'
  ORDER BY created_at DESC;

# Identify which users/resources deleted
# Example: User john@example.com deleted on 2024-01-15 14:32:15
```

**Step 2: Prepare Restore Point**
```bash
# Use point-in-time recovery (PITr)
# PostgreSQL allows recovery to any second

# Check backup availability
aws s3 ls s3://vpn-backups/pitr/

# Download backup from before deletion
aws s3 cp s3://vpn-backups/pitr/vpn_db_2024-01-15_14-00.sql.gz .
gunzip vpn_db_2024-01-15_14-00.sql.gz
```

**Step 3: Restore from Backup**
```bash
# Create temporary database
docker exec vpn-db createdb -U vpn_user vpn_db_restore

# Restore backup to temp database
cat vpn_db_2024-01-15_14-00.sql | \
  docker exec -i vpn-db psql -U vpn_user -d vpn_db_restore

# Compare data between current and restored
# Query for missing records
```

**Step 4: Selective Data Recovery**
```bash
-- If only some users/data deleted:
-- Insert recovered data back into production

INSERT INTO users (id, email, name, created_at, subscription_tier)
SELECT id, email, name, created_at, subscription_tier 
FROM vpn_db_restore.users 
WHERE email NOT IN (SELECT email FROM vpn_db.users)
  AND deleted_at > NOW() - INTERVAL '1 day';

-- Verify recovery
SELECT COUNT(*) FROM users WHERE deleted_at IS NOT NULL;
```

**Step 5: Notify Users**
- Send emails to affected users
- Explain data restoration
- Request password reset if needed
- Provide support contact

---

### 2.2 Database Corruption (RTO: 2 hours, RPO: 15 minutes)

#### Symptoms
- Integrity constraint violations
- Duplicate key errors
- UUID format errors
- Unable to run queries

#### Recovery Steps

**Step 1: Verify Corruption**
```bash
# Run database integrity check
docker exec vpn-db pg_dump -U vpn_user vpn_db | \
  psql -U vpn_user -d test_db < /dev/null

# Check specific tables
docker exec vpn-db psql -U vpn_user -d vpn_db -c "\d+ users"
```

**Step 2: Backup Current State**
```bash
# Create backup of corrupted database (for analysis)
docker exec vpn-db pg_dump -U vpn_user -Fc vpn_db > \
  /backups/vpn_db_corrupted_$(date +%s).sql.gz
```

**Step 3: Restore from Last Known Good Backup**
```bash
# Stop application
docker-compose stop api

# Restore from backup
docker exec vpn-db dropdb -U vpn_user vpn_db
docker exec vpn-db createdb -U vpn_user vpn_db

# Restore backup
aws s3 cp s3://vpn-backups/vpn_db_latest.sql.gz .
gunzip vpn_db_latest.sql.gz
cat vpn_db_latest.sql | \
  docker exec -i vpn-db psql -U vpn_user -d vpn_db

# Verify
docker exec vpn-db psql -U vpn_user -d vpn_db -c "SELECT COUNT(*) FROM users"

# Restart application
docker-compose up -d api
```

**Step 4: Data Validation**
```bash
# Run consistency checks
SELECT table_name FROM information_schema.tables 
  WHERE table_schema = 'public';

-- Verify foreign key relationships
SELECT COUNT(*) as orphaned_records FROM peers 
  WHERE user_id NOT IN (SELECT id FROM users);
```

---

## 3. SECURITY INCIDENTS

### 3.1 Suspected Data Breach (RTO: immediate, RPO: N/A)

#### Response Plan

**Step 1: Isolate System (Immediately)**
```bash
# Stop accepting external traffic
# Keep internal monitoring active
iptables -I INPUT 1 -i eth0 -j DROP
iptables -I INPUT 2 -i eth0 -s 10.0.0.0/8 -j ACCEPT
```

**Step 2: Preserve Evidence**
```bash
# Create full backup for forensics
docker exec vpn-db pg_dump -U vpn_user vpn_db | \
  gzip > /forensics/vpn_db_breach_$(date +%s).sql.gz

# Copy logs for analysis
docker logs vpn-api > /forensics/api_logs_$(date +%s).txt
docker logs vpn-db > /forensics/db_logs_$(date +%s).txt

# Network packet capture (if available)
tcpdump -i eth0 -w /forensics/packets_$(date +%s).pcap
```

**Step 3: Analyze Breach**
```bash
# Check audit logs for unauthorized access
SELECT * FROM audit_logs 
  WHERE created_at > NOW() - INTERVAL '24 hours'
  AND event_type IN ('LOGIN_FAILED', 'PRIVILEGE_ESCALATION', 'DATA_EXPORT')
  ORDER BY created_at DESC;

# Check for suspicious data access
SELECT * FROM audit_logs 
  WHERE event_type = 'DATA_EXPORT' 
  OR event_type = 'USER_DATA_ACCESSED'
  ORDER BY created_at DESC LIMIT 100;
```

**Step 4: Reset Credentials**
```bash
# Force password reset for all users
UPDATE users SET force_password_reset = true;

# Revoke all API keys
DELETE FROM api_keys WHERE created_at < NOW();

# Invalidate all sessions
DELETE FROM sessions WHERE created_at < NOW() - INTERVAL '1 hour';

# Rotate service credentials
# Change database passwords
# Regenerate API keys
# Rotate encryption keys
```

**Step 5: Incident Response**
- Notify security team immediately
- Document timeline
- Contact legal/compliance
- Prepare breach notification
- Alert users within 72 hours
- Report to regulatory authorities if required

---

### 3.2 Account Compromise (RTO: 15 minutes, RPO: N/A)

#### Response Steps

**Step 1: Identify Compromised Account**
```bash
# Query for suspicious activity
SELECT user_id, event_type, COUNT(*) 
FROM audit_logs 
WHERE created_at > NOW() - INTERVAL '1 hour' 
GROUP BY user_id, event_type 
ORDER BY count(*) DESC;
```

**Step 2: Suspend Account**
```bash
-- Suspend the account immediately
UPDATE users SET suspended = true 
WHERE id = 'compromised_user_id';

-- Invalidate all sessions
DELETE FROM sessions WHERE user_id = 'compromised_user_id';

-- Revoke API keys
UPDATE api_keys SET revoked = true 
WHERE user_id = 'compromised_user_id';
```

**Step 3: Notify User**
- Send urgent email alerting of account compromise
- Provide link to reset password
- Suggest enabling 2FA
- Request confirmation of actions

**Step 4: Monitor**
```bash
# Monitor for further suspicious activity
SELECT * FROM audit_logs 
WHERE user_id = 'compromised_user_id' 
AND created_at > NOW() - INTERVAL '24 hours'
ORDER BY created_at DESC;
```

---

## 4. NETWORK & INFRASTRUCTURE

### 4.1 Network Outage (RTO: varies, RPO: 0)

#### Recovery Steps

**Step 1: Assess Connectivity**
```bash
# Check network interfaces
ip addr show

# Check routing
route -n

# Test external connectivity
ping 8.8.8.8

# Test DNS
nslookup vpn-service.com
```

**Step 2: Failover**
```bash
# If primary network down, failover to secondary
# Configure backup network interface

# Update DNS/load balancer to redirect traffic
# To secondary data center / failover region
```

**Step 3: Restore Service**
- Once network restored, verify all services
- Run health checks on all components
- Gradually shift traffic back

---

### 4.2 DDoS Attack (RTO: ongoing, RPO: 0)

#### Mitigation

**Step 1: Activate DDoS Protection**
```bash
# Enable CloudFlare On (if using)
# Increase rate limiting
# Activate WAF rules
```

**Step 2: Scale Infrastructure**
```bash
# Auto-scale backend instances
docker-compose up -d --scale api=5

# Increase cache capacity
# Increase database connections
```

**Step 3: Rate Limiting**
```bash
# Enable aggressive rate limiting
# 100 requests per minute per IP
redis-cli CONFIG SET maxmemory 2gb
```

**Step 4: Monitor**
- Watch traffic patterns
- Identify attacker IPs
- Block at firewall level if possible

---

## 5. BACKUP & RESTORE PROCEDURES

### 5.1 Automated Daily Backups

```bash
#!/bin/bash
# Location: scripts/backup.sh

BACKUP_DIR="/backups/vpn"
S3_BUCKET="s3://vpn-backups"
RETENTION_DAYS=30

# Create backup
docker exec vpn-db pg_dump -U vpn_user -Fc vpn_db | \
  gzip > $BACKUP_DIR/vpn_db_backup_$(date +%Y-%m-%d_%H-%M-%S).sql.gz

# Upload to S3
aws s3 sync $BACKUP_DIR $S3_BUCKET/database/

# Cleanup old backups (keep 30 days)
find $BACKUP_DIR -name "*.sql.gz" -mtime +$RETENTION_DAYS -delete

# Verify backup integrity
gunzip -t $BACKUP_DIR/vpn_db_backup_latest.sql.gz

# Log backup completion
echo "Backup completed at $(date)" >> /var/log/vpn-backups.log
```

### 5.2 Test Restore Procedures

**Monthly Restore Test (1st of every month):**
```bash
#!/bin/bash
# Download latest backup
aws s3 cp $(aws s3 ls s3://vpn-backups/database/ | tail -1 | awk '{print $4}') \
  /tmp/test_backup.sql.gz

# Restore to test database
gunzip -c /tmp/test_backup.sql.gz | \
  docker exec -i vpn-db psql -U vpn_user -d vpn_db_test

# Run validation queries
docker exec vpn-db psql -U vpn_user -d vpn_db_test -c \
  "SELECT COUNT(*) FROM users; SELECT COUNT(*) FROM vpn_connections;"

# Report results
echo "Restore test: PASSED" >> /var/log/restore-tests.log
```

---

## 6. COMMUNICATION PLAN

### Notification Hierarchy

**Level 1 (Site Down):** Notify within 5 minutes
- [ ] On-call engineer
- [ ] Engineering manager
- [ ] CTO

**Level 2 (Significant Impact):** Notify within 15 minutes
- [ ] All engineers
- [ ] Customer success team
- [ ] Marketing

**Level 3 (User Impact):** Notify within 30 minutes
- [ ] All affected users (email)
- [ ] Status page update
- [ ] Twitter/social media

### Status Page
- Update https://status.vpn-service.com every 15 minutes
- Post-incident review within 24 hours

---

## 7. RTO/RPO SUMMARY

| Scenario | RTO | RPO | Notes |
|----------|-----|-----|-------|
| Database Failure | 1 hour | 15 min | Automated backup restore |
| API Crash | 5 min | 0 | Container restart |
| Redis Failure | 30 min | 0 | Cache rebuild from DB |
| Data Deletion | 2 hours | 1 day | Point-in-time recovery |
| DB Corruption | 2 hours | 15 min | Restore from backup |
| Breach | Immediate | N/A | Evidence preservation |
| Network Outage | Varies | 0 | Failover to secondary |

---

## 8. Recovery Contacts

**Primary On-Call Engineer:** [Name/Phone]
**Secondary On-Call:** [Name/Phone]
**Engineering Manager:** [Name/Phone]
**CTO:** [Name/Phone]
**Incident Commander:** [Name/Email]

---

**Last Updated:** 2024-01-15
**Next Review:** 2024-04-15
**Document Version:** 1.0
