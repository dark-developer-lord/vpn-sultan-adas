# VPN Service Incident Response Playbooks

## Overview

This document outlines step-by-step procedures for responding to common production incidents.

**Response Time Targets**:
- P1 (Critical Outage): 5 minutes
- P2 (Major Degradation): 30 minutes
- P3 (Minor Issues): 4 hours

---

## 🔴 P1 Incident: API Service Down (No Response)

**Symptoms**: 
- `curl http://localhost:3000/health` times out
- Grafana shows 0 requests for 2+ minutes
- AlertManager fires "APIDown" alert

**Investigation (5 minutes)**:

1. **Check container status**:
   ```bash
   docker-compose ps | grep vpn-api
   ```
   Expected: `vpn-api ... Up`

2. **Check recent logs**:
   ```bash
   docker-compose logs vpn-api --tail 50
   ```
   Look for: Crashes, memory errors, port conflicts

3. **Check resource usage**:
   ```bash
   docker stats vpn-api
   ```
   Look for: CPU > 100%, Memory > available

4. **Test database connectivity**:
   ```bash
   docker-compose logs postgres
   pg_isready -h postgres -U vpn
   ```

5. **Check port binding**:
   ```bash
   netstat -tln | grep 3000
   ```

**Remediation**:

- **If container crashed**:
  ```bash
  docker-compose restart vpn-api
  sleep 5
  curl -v http://localhost:3000/health
  ```

- **If memory exhausted**:
  ```bash
  # Increase memory limit in docker-compose.yml
  # deploy:
  #   resources:
  #     limits:
  #       memory: 512M
  docker-compose up -d vpn-api
  ```

- **If port conflict**:
  ```bash
  lsof -i :3000  # Find process using port
  kill -9 <PID>
  docker-compose restart vpn-api
  ```

- **If unresolvable**:
  ```bash
  docker-compose down
  docker-compose build --no-cache vpn-api
  docker-compose up -d vpn-api
  ```

**Verification**:
```bash
# Should return {"status": "ok"}
curl http://localhost:3000/health

# Monitor for 5 minutes
watch 'curl -s http://localhost:3000/health'
```

**Escalation**: If still down after 10 minutes → Page CTO

---

## 🔴 P1 Incident: Error Rate Spike (> 5% for 5 min)

**Symptoms**:
- AlertManager fires "HighErrorRate"
- Grafana "Errors" dashboard showing red
- Users reporting failures

**Investigation (5 minutes)**:

1. **Identify error scope**:
   ```bash
   # From Loki logs (in Grafana)
   {job="vpn-api"} | json | status >= 400
   ```
   Look for: Which endpoints? Which error codes?

2. **Check database health**:
   ```bash
   docker-compose logs postgres
   # Should show no errors
   ```

3. **Check recent deployments**:
   ```bash
   git log --oneline -5
   # Did something change in the last 10 minutes?
   ```

4. **Review error distribution**:
   ```
   - 500 errors? → Application bug
   - 429 errors? → Rate limiting triggered
   - 503 errors? → Dependency down
   - 401 errors? → Auth system issue
   ```

**Remediation**:

- **If database is slow**:
  ```bash
  docker-compose exec postgres psql -U vpn -d vpn_service -c "SELECT pid, query FROM pg_stat_activity WHERE state = 'active';"
  # Kill long-running queries if needed
  ```

- **If rate limiting engaged**:
  ```bash
  # Check rate limit metrics in Prometheus
  # Rate limits only activate if legitimate surge
  # Options:
  # A) Wait 5 minutes for cooldown
  # B) Temporarily increase limits (if attack suspected)
  ```

- **If code bug**:
  ```bash
  # Check what changed
  git diff HEAD~1 HEAD
  
  # If clear fix available → deploy fix
  # Otherwise → revert to previous version
  docker-compose up vpn-api  # Redeploy from last good commit
  ```

- **If auth system issue**:
  ```bash
  # Restart auth service
  docker-compose restart vpn-api
  ```

**Verification**:
```bash
# Error rate should drop below 1%
# Check in Grafana dashboard
# Prometheus query: rate(http_errors_total[1m]) / rate(http_requests_total[1m])
```

---

## 🟡 P2 Incident: Database Connections Exhausted

**Symptoms**:
- "Connection refused" errors
- Grafana shows connection pool at 100%
- Queries timing out

**Investigation (10 minutes)**:

1. **Check connection pool status**:
   ```bash
   docker-compose exec postgres psql -U vpn -d vpn_service -c "SELECT count(*) FROM pg_stat_activity;"
   # And max_connections setting
   SHOW max_connections;
   ```

2. **Identify long-running queries**:
   ```bash
   docker-compose exec postgres psql -U vpn -d vpn_service <<EOF
   SELECT 
       pid, 
       usename, 
       application_name, 
       state, 
       query_start,
       query
   FROM pg_stat_activity 
   WHERE state != 'idle' 
   ORDER BY query_start;
   EOF
   ```

3. **Check for connection leaks**:
   ```bash
   # Connections by application
   SELECT application_name, count(*) FROM pg_stat_activity GROUP BY application_name;
   ```

**Remediation**:

- **Kill idle connections**:
  ```bash
  docker-compose exec postgres psql -U vpn -d vpn_service -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE state = 'idle' AND query_start < now() - interval '10 minutes';"
  ```

- **Kill long-running query**:
  ```bash
  docker-compose exec postgres psql -U vpn -d vpn_service -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE query_start < now() - interval '30 minutes';"
  ```

- **Increase connection pool**:
  ```bash
  # Edit docker-compose.yml
  # Add environment: POSTGRES_INIT_ARGS="-c max_connections=200"
  docker-compose restart postgres
  ```

- **Restart API connections**:
  ```bash
  docker-compose restart vpn-api
  ```

---

## 🟡 P2 Incident: Disk Space Critical (> 85%)

**Symptoms**:
- AlertManager fires "DiskSpaceCritical"
- Writes may start failing
- PostgreSQL might hang

**Investigation (5 minutes)**:

1. **Check disk usage**:
   ```bash
   df -h /
   du -sh /var/lib/docker/volumes/*
   ```

2. **Identify large items**:
   ```bash
   du -sh /var/lib/docker/volumes/*/
   du -sh /var/backups/*
   ```

**Remediation** (in priority order):

1. **Clean old backups** (usually frees 50%+):
   ```bash
   find /var/backups -name "vpn-db-*.sql.gz" -mtime +3 -delete
   ```

2. **Clean Docker images**:
   ```bash
   docker image prune -a --force
   ```

3. **Clean Docker volumes**:
   ```bash
   docker volume prune --force
   ```

4. **Check database size**:
   ```bash
   docker-compose exec postgres psql -U vpn -d vpn_service -c "SELECT pg_size_pretty(pg_database_size('vpn_service'));"
   ```

5. **If still critical**: Plan storage upgrade

---

## 🟡 P2 Incident: High API Latency (P95 > 500ms)

**Symptoms**:
- AlertManager fires "HighLatency"
- Slow API responses
- Users experiencing timeouts

**Investigation (10 minutes)**:

1. **Check database query times**:
   ```bash
   # From Prometheus: avg(rate(database_query_duration_seconds_sum/database_query_duration_seconds_count[5m]))
   ```

2. **Identify slow queries from Loki**:
   ```
   {job="vpn-api"} | json | duration_ms > 500
   ```

3. **Check Redis cache hit rate**:
   ```
   Cache hit rate should be > 70%
   If < 50%: Cache is not warming up properly
   ```

4. **Check system resources**:
   ```bash
   docker stats
   # CPU, memory, network
   ```

**Remediation**:

- **If database slow**:
  ```bash
  # Run optimization script
  bash scripts/backups/optimize-database.sh
  # Creates indexes, vacuums tables
  ```

- **If cache missing**:
  ```bash
  # Restart Redis
  docker-compose restart redis
  # Force cache refresh
  ```

- **If CPU maxed**:
  ```bash
  # Check for runaway process
  docker stats
  
  # May need to scale horizontally
  # Update docker-compose to add replicas
  ```

**Verification**:
```bash
# Latency should drop to < 200ms
# Check Grafana P95 metric
# Prometheus: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))
```

---

## 🟢 P3 Incident: Non-Critical Service Degradation

**Symptoms**:
- Minor features not working
- Some endpoint returning 4xx errors
- But API itself is up and responding

**Response Time Target**: 4 hours

**Process**:

1. **Document the issue**:
   - What is affected?
   - Which users impacted?
   - When did it start?

2. **Add to backlog** (don't fix immediately)
   - Create GitHub issue with tags
   - Assign to team for next sprint

3. **Workaround for users** (if possible):
   - Document in status page
   - Provide alternative approach
   - Set expectation for fix timeline

4. **Plan fix** for next release

---

## 📋 Universal Troubleshooting Checklist

For ANY incident, follow this order:

1. ☐ Assess severity (P1/P2/P3)
2. ☐ Check container status: `docker-compose ps`
3. ☐ Check recent logs: `docker-compose logs --tail 100`
4. ☐ Check Grafana dashboards
5. ☐ Check Loki error logs
6. ☐ Check Prometheus alerts
7. ☐ Identify root cause (code/infra/external)
8. ☐ Implement fix or workaround
9. ☐ Verify resolution
10. ☐ Document incident
11. ☐ Schedule post-mortem (P1/P2 only)

---

## 📞 Escalation Matrix

| Severity | Initial Response | Unresolved After | Action |
|----------|------------------|-------------------|--------|
| P1 | 5 min | 10 min | Page CTO |
| P1 | 5 min | 30 min | CEO notification |
| P2 | 30 min | 2 hours | CTO consultation |
| P3 | 4 hours | 1 day | Add to backlog |

---

## 🔧 Quick Commands Reference

```bash
# Restart all services
docker-compose restart

# Restart specific service
docker-compose restart vpn-api

# View logs
docker-compose logs -f vpn-api

# Execute command in container
docker-compose exec postgres psql -U vpn -d vpn_service -c "SELECT version();"

# Check health
curl http://localhost:3000/health

# View metrics
curl http://localhost:3000/metrics

# Prometheus query
curl 'http://localhost:9090/api/v1/query?query=up'

# AlertManager alerts
curl http://localhost:9093/api/v1/alerts
```

---

**Last Updated**: March 28, 2026
**Owner**: DevOps Team
**Review Frequency**: Quarterly
