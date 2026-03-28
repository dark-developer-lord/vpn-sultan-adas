# VPN Service Disaster Recovery & Scaling Guide

Procedures for recovering from critical failures and scaling the system for growth.

---

## 🚨 Disaster Recovery Plan

### Recovery Time Objectives (RTO & RPO)

| Scenario | RTO | RPO | Procedure |
|----------|-----|-----|-----------|
| API Container Crash | 5 min | 0 min | Restart container |
| Database Corruption | 30 min | 24h | Restore from last backup |
| VPS Disk Full | 15 min | 1h | Archive old backups |
| Complete VPS Failure | 2h | 24h | Restore to new VPS |
| Data Center Outage | 4h | 24h | Failover to secondary DC |

### Backup Strategy

**Current Setup**:
- **Daily Full Backups**: 2:00 AM UTC, stored locally
- **Local Retention**: 7 days
- **Archive Retention**: 30 days (moved to archive storage)
- **Verification**: SHA256 checksums validated daily

**Backup Locations**:
```
/var/backups/vpn-db-*.sql.gz        # Local (7 day retention)
/archive/vpn-backups/vpn-db-*.sql.gz  # Archive (30 day retention)
```

**Backup Contents**:
- Complete PostgreSQL database (all tables, indexes, sequences)
- Application configuration (stored in database)
- User data, subscriptions, payments, audit logs
- Does NOT include: Docker images (can be rebuilt), logs (in Loki)

### Disaster Recovery Scenarios

---

## Scenario 1: Single Container Crash

**Time to Recover**: 5 minutes | **Data Loss**: None

### Step 1: Detect

AlertManager fires "ContainerDown" alert:
```bash
curl http://localhost:9093/api/v1/alerts | jq '.data[] | select(.labels.severity=="critical")'
```

### Step 2: Diagnose

```bash
# Check which container is down
docker-compose ps | grep -v "Up"

# Review recent logs
docker-compose logs --tail 50 <container_name>
```

### Step 3: Recover

**Option A: Simple Restart** (try first):
```bash
docker-compose restart <container_name>
sleep 5
curl http://localhost:3000/health
```

**Option B: Force Recreate**:
```bash
docker-compose up -d --force-recreate <container_name>
sleep 5
curl http://localhost:3000/health
```

### Step 4: Verify

```bash
bash scripts/smoke-tests.sh
```

---

## Scenario 2: Database Corruption

**Time to Recover**: 30 minutes | **Data Loss**: Up to 24 hours

### Step 1: Detect

Database queries fail with corruption errors:
```
ERROR: invalid page header in block...
ERROR: relation is corrupted...
```

### Step 2: Assess Scope

```bash
# Try to connect
docker-compose exec postgres psql -U vpn -d vpn_service -c "SELECT COUNT(*) FROM users;"

# If fails, database is corrupted
```

### Step 3: Backup Current State (for investigation)

```bash
# Dump what we can recover
docker-compose exec postgres pg_dumpall > /backups/corrupted-dump-$(date +%s).sql

# Stop API to prevent further writes
docker-compose stop vpn-api
```

### Step 4: Restore from Latest Backup

See [Database Backup & Recovery](operational-runbooks.md#-database-backup--recovery) in Runbooks.

```bash
# Find latest good backup
ls -lhrS /var/backups/vpn-db-*.sql.gz | head -1

# Stop all services writing to DB
docker-compose stop vpn-api pgadmin

# Drop corrupted database
docker-compose exec postgres dropdb -U vpn vpn_service

# Recreate empty database
docker-compose exec postgres createdb -U vpn vpn_service

# Restore from backup
zcat /var/backups/vpn-db-2024-01-15.sql.gz | docker-compose exec -T postgres psql -U vpn vpn_service

# Restart services
docker-compose up -d vpn-api pgadmin
sleep 5
```

### Step 5: Verify Integrity

```bash
# Test all tables exist
docker-compose exec postgres psql -U vpn -d vpn_service -c "\dt"

# Run smoke tests
bash scripts/smoke-tests.sh

# Check table row counts
docker-compose exec postgres psql -U vpn -d vpn_service -c "
SELECT tablename, n_live_tup FROM pg_stat_user_tables 
ORDER BY n_live_tup DESC;"
```

### Step 6: Post-Incident

1. Document what caused corruption
2. Add sanity check to monitoring
3. Update backup frequency if needed

---

## Scenario 3: Disk Full

**Time to Recover**: 15 minutes | **Data Loss**: None

### Step 1: Detect

AlertManager fires "DiskSpaceCritical":
```bash
df -h / | awk 'NR==2 {print $5}'  # > 85%
```

### Step 2: Identify What's Using Space

```bash
du -sh /var/lib/docker/volumes/*/
du -sh /var/backups/
du -sh /archive/
du -sh /var/log/
```

### Step 3: Free Space (Priority Order)

**Option A: Archive Old Backups** (usually 50%+ space):
```bash
# Move backups older than 3 days to archive
find /var/backups/vpn-db-*.sql.gz -mtime +3 -exec mv {} /archive/ \;

# Verify space freed
df -h /
```

**Option B: Clean Docker Images**:
```bash
docker image prune -a --force  # Remove unused images
docker volume prune --force    # Remove unused volumes
```

**Option C: Clean Log Files**:
```bash
# Truncate old log files (Loki handles retention)
find /var/log -name "vpn-*.log" -mtime +7 -delete
```

**Option D: Emergency - Delete Oldest Archives**:
```bash
# Only if still critical
find /archive -name "vpn-db-*.sql.gz" -mtime +30 -delete
```

### Step 4: Verify Recovery

```bash
df -h /  # Should show > 20% free
curl http://localhost:3000/health
```

### Step 5: Plan Expansion

If frequently hitting 85%:
1. Add additional volume: `docker volume create vpn-backups-ext`
2. Mount to `/archive` with more space
3. Implement log rotation with earlier cleanup

---

## Scenario 4: Complete VPS Loss

**Time to Recover**: 2 hours | **Data Loss**: Up to 24 hours

### Prerequisites

Before disaster strikes:
- [ ] Backups stored in S3 or external location
- [ ] Docker images pushed to Docker Hub
- [ ] Configuration documented and versioned
- [ ] SSH keys backed up

### Step 1: Provision New VPS

1. Order new VPS with same/better specs:
   - 1+ vCPU (same as current)
   - 4+ GB RAM (same as current)
   - 50+ GB SSD (same as current)
   - Ubuntu 22.04 LTS (same OS)

2. Get new IP address: `187.124.XXX.YYY` (will differ)

### Step 2: Set Up New VPS

```bash
# SSH into new VPS
ssh -i ~/.ssh/id_ed25519 root@<NEW_IP>

# Basic setup
apt update && apt upgrade -y
apt install -y docker.io docker-compose curl git

# Clone repository
cd /root
git clone https://github.com/dark-developer-lord/vpn-sultan-adas.git vpn-sultan-adas
cd vpn-sultan-adas

# Create data directories
mkdir -p /var/backups /archive /var/log/vpn-backups
```

### Step 3: Restore Database

```bash
# Download backup from S3 or external storage
# (Assuming you have backups in S3)
aws s3 cp s3://vpn-backups/vpn-db-latest.sql.gz /tmp/backup.sql.gz

# Or if on local storage, copy from laptop:
scp -i ~/.ssh/id_ed25519 /local/path/to/backup.sql.gz root@<NEW_IP>:/tmp/backup.sql.gz
```

### Step 4: Start Fresh Services

```bash
# On new VPS
cd ~/vpn-sultan-adas

# Build and start database only
docker-compose up -d postgres

# Wait for postgres to be ready (30 seconds)
sleep 30

# Check it's running
docker-compose ps

# Restore database
zcat /tmp/backup.sql.gz | docker-compose exec -T postgres psql -U vpn postgres -c "CREATE DATABASE vpn_service;"
zcat /tmp/backup.sql.gz | docker-compose exec -T postgres psql -U vpn vpn_service
```

### Step 5: Start Other Services

```bash
# Start monitoring stack (but not API yet)
docker-compose up -d prometheus grafana loki promtail alertmanager
docker-compose up -d postgres-exporter redis-exporter redis

# Start API with fresh credentials
docker-compose up -d vpn-api

# Verify health
sleep 5
curl http://localhost:3000/health
```

### Step 6: Verify Data Integrity

```bash
bash scripts/smoke-tests.sh
```

### Step 7: Update DNS

**If using domain name**:
```bash
# Update DNS A record to new IP
# Old: 187.124.179.20 → vpn.service
# New: 187.124.XXX.YYY → vpn.service

# Propagation: 15-24 hours (critical for users!)
# Alternative: Keep old IP, but use endpoint routing
```

### Step 8: Verify External Access

```bash
# From local machine
curl http://<NEW_IP>:3000/health
curl http://<NEW_IP>:3001  # Grafana

# If using domain (after DNS propagation)
curl http://vpn.service/health
```

---

## 🚀 Scaling the System

### Current Capacity (Single VPS)

```
Current Setup: 1 vCPU, 4GB RAM, 50GB SSD
Estimated Capacity: 30-100 concurrent users
Performance: P95 latency ~100ms, error rate < 1%
```

### Monitoring Capacity Usage

```bash
# Real-time metrics
watch docker stats

# Database size
docker-compose exec postgres psql -U vpn -d vpn_service -c "
  SELECT pg_size_pretty(pg_database_size('vpn_service'));"

# API request rate
curl -s 'http://localhost:9090/api/v1/query?query=rate(http_requests_total[5m])' | jq '.data.result[0].value'
```

### When to Scale

**Trigger Scaling If**:
- CPU usage consistently > 70%
- RAM usage > 80%
- P95 latency > 200ms
- Error rate > 1%
- Database size > 10GB
- Expected growth > 50% users in next month

### Scaling Strategy Options

#### Option 1: Vertical Scaling (Bigger VPS)

**Simplest approach - upgrade on same server**:

1. Order larger VPS (2+ vCPU, 8+ GB RAM, 100+ GB SSD)
2. Backup database: `bash scripts/backups/backup-database.sh`
3. Stop services: `docker-compose down`
4. Migrate volumes to new VPS
5. Start services: `docker-compose up -d`
6. Run tests: `bash scripts/smoke-tests.sh`

**Pros**: Simple, no code changes
**Cons**: Single point of failure, limited growth

#### Option 2: Horizontal Scaling (API Load Balancing)

**Production-grade approach**:

```yaml
# Updated docker-compose.yml structure
version: '3.8'

services:
  # Load Balancer
  nginx:
    image: nginx:latest
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - vpn-api-1
      - vpn-api-2
  
  # API Replicas
  vpn-api-1:
    build: ./docker
    environment:
      - PORT=3001
      - INSTANCE_ID=1
    depends_on:
      - postgres
  
  vpn-api-2:
    build: ./docker
    environment:
      - PORT=3002
      - INSTANCE_ID=2
    depends_on:
      - postgres
  
  # Shared databases stay the same
  postgres:
    image: postgres:15-alpine
  
  redis:
    image: redis:7-alpine
```

**nginx.conf** (simple round-robin):
```nginx
upstream vpn_api {
    server vpn-api-1:3001;
    server vpn-api-2:3002;
    server vpn-api-3:3003;
}

server {
    listen 80;
    location / {
        proxy_pass http://vpn_api;
    }
}
```

**Pros**: Distribute load, higher availability
**Cons**: Complexity increases, need session management

#### Option 3: Database Read Replicas

**For read-heavy workloads**:

```bash
# Create read replica of PostgreSQL
docker run \
  --name postgres-replica \
  -e POSTGRES_REPLICATION_MODE=slave \
  -e POSTGRES_MASTER_SERVICE=postgres \
  postgres:15-alpine
```

**Benefits**:
- Offload read queries to replica
- Better query distribution
- Failover option

#### Option 4: Caching Layer (Redis Cluster)

**For high-concurrency scenarios**:

```yaml
redis-cluster:
  image: redis:7-alpine
  command: redis-server --cluster-enabled yes
  volumes:
    - redis-cluster-data:/data
```

**Caching strategies**:
- Cache user profile lookups (TTL: 1 hour)
- Cache VPN server list (TTL: 24 hours)
- Cache subscription status (TTL: 5 minutes)

### Scaling Roadmap (0-500 Users)

| Phase | Users | vCPU | RAM | Approach |
|-------|-------|------|-----|----------|
| Current | 30-100 | 1 | 4GB | Single VPS |
| Phase 3A | 100-200 | 1 | 8GB | Vertical scale |
| Phase 3B | 200-400 | 2 | 16GB | + API replicas (2x) |
| Phase 3C | 400-500 | 4 | 32GB | + DB replicas + Redis cluster |

### Load Testing Before Scaling

**Prepare load test**:

```bash
# Create load test script using Apache Bench or k6
ab -n 10000 -c 100 http://localhost:3000/health

# Or with k6 (more sophisticated)
cat > load-test.js <<EOF
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  stages: [
    { duration: '2m', target: 100 },
    { duration: '5m', target: 100 },
    { duration: '2m', target: 0 },
  ],
};

export default function() {
  let res = http.get('http://localhost:3000/health');
  check(res, { 'status was 200': (r) => r.status == 200 });
}
EOF

k6 run load-test.js
```

**Analyze results**:
```bash
# Check Grafana metrics during load test
# Key graphs:
# - Request rate (should handle target without errors)
# - P95/P99 latency (should stay < 500ms)
# - Error rate (should be < 1%)
# - CPU/Memory (should have headroom)
```

---

## 🔄 Failover & High Availability

### Active-Passive Setup (Future)

**When to implement**: After reaching 200+ concurrent users

```
        DNS Round-Robin
             |
      +------+------+
      |             |
    Primary      Secondary
    (Active)      (Standby)
   187.124.1.1   187.124.1.2
   
   Shared Database (Primary-Replica)
   Shared Storage (Replicated)
```

**Setup**:
1. Provision second VPS
2. Set up PostgreSQL replication
3. Configure DNS failover
4. Test failover procedures monthly

### Auto-Failover Script

```bash
#!/bin/bash
# Check primary health every 10 seconds

while true; do
  if ! curl -f http://primary-ip:3000/health >/dev/null 2>&1; then
    echo "Primary down! Failing over to secondary"
    
    # Update DNS to point to secondary
    aws route53 change-resource-record-sets \
      --hosted-zone-id Z123456 \
      --change-batch '[{"Action":"UPSERT","ResourceRecordSet":{"Name":"vpn.service","Type":"A","TTL":60,"ResourceRecords":[{"Value":"187.124.1.2"}]}}]'
    
    echo "Failover complete - secondary is now primary"
    break
  fi
  
  sleep 10
done
```

---

## 📋 Runbook Checklist

### Disaster Recovery Readiness

- [ ] Backups test-restored monthly
- [ ] Backup stored in multiple locations (local + cloud)
- [ ] VPS new provisioning documented
- [ ] DNS failover tested
- [ ] Database replication validated
- [ ] Team trained on recovery procedures
- [ ] Recovery scripts executable and tested

### Scaling Readiness

- [ ] Load testing scripts created
- [ ] Monitoring dashboards show all key metrics
- [ ] Scaling triggers documented
- [ ] Horizontal scaling docker-compose ready
- [ ] Load balancer configuration ready
- [ ] Database replication tested
- [ ] Redis cluster configuration ready

---

**Last Updated**: March 28, 2026
**Owner**: DevOps & SRE Team
**Review Frequency**: Quarterly
**Last Drill**: None (First implementation)
