# Phase 2: Post-Launch Stabilization - Operations Setup
**Timeline: March 28 - April 27, 2026 (Weeks 5-8)**

---

## 🎯 Phase 2 Objectives

1. **Monitoring & Observability** - Full visibility into system health
2. **Security Hardening** - Production-grade security
3. **Performance Optimization** - Scale from 30 to 100+ concurrent users
4. **Operational Excellence** - Automated backups, incident response

---

## Week 5: Monitoring Infrastructure

### 1.1 Enable Prometheus Metrics

**Objective**: Expose application metrics for collection

**Implementation**:
```bash
# Update mock API to expose metrics endpoint
# File: docker/Dockerfile.api

# Add metrics collection endpoint
# GET /metrics -> Prometheus-format metrics
```

**Metrics to Collect**:
- `http_requests_total` - Total API requests by endpoint
- `http_request_duration_seconds` - Request latency (p50, p95, p99)
- `http_errors_total` - Error count by status code
- `database_connections_active` - Active DB connections
- `database_query_duration_seconds` - Query latency
- `authentication_attempts_total` - Auth success/fail count
- `vpn_active_connections` - Active VPN sessions
- `system_uptime_seconds` - Service uptime

**Status**: 🔲 Not Started
**Owner**: DevOps
**Effort**: 4 hours

---

### 1.2 Deploy Prometheus

**Configuration**:
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'vpn-api'
    static_configs:
      - targets: ['localhost:3000']
  - job_name: 'postgres'
    static_configs:
      - targets: ['localhost:9187']
  - job_name: 'redis'
    static_configs:
      - targets: ['localhost:9121']
```

**Status**: 🔲 Not Started
**Owner**: DevOps
**Effort**: 2 hours

---

### 1.3 Deploy Grafana Dashboards

**Dashboards to Create**:

1. **System Overview Dashboard**
   - API uptime and error rate
   - Request rate and latency
   - Database connection pool
   - System resource usage (CPU, memory)

2. **Database Performance Dashboard**
   - Query count and latency
   - Active connections
   - Cache hit rate
   - Slow query analysis

3. **User Activity Dashboard**
   - Active users by hour
   - Authentication success/failure rate
   - VPN connection churn
   - Geographic distribution

4. **Security Dashboard**
   - Failed authentication attempts
   - Rate limiting triggers
   - Failed payment transactions
   - DDoS indicators

**Status**: 🔲 Not Started
**Owner**: DevOps
**Effort**: 6 hours

---

### 1.4 Logging with Loki

**Configuration**:
```yaml
promtail:
  scrape_configs:
    - job_name: vpn-api
      docker: {}
    - job_name: postgres
      docker: {}
    - job_name: vpn-errors
      docker:
        labels:
          job: error-logs
```

**Log Levels to Configure**:
- ERROR: System and application errors
- WARN: Degraded performance, failed retries
- INFO: API requests, authentication events
- DEBUG: Internal operations (only in dev)

**Status**: 🔲 Not Started
**Owner**: DevOps
**Effort**: 3 hours

---

### 1.5 Alerting with AlertManager

**Critical Alerts** (Page on-call):
- API down (no response for 2 min)
- Error rate > 5% for 5 minutes
- Database offline
- Disk usage > 85%
- Memory usage > 90%

**Warning Alerts** (Email):
- High latency P95 > 500ms
- Database connection pool > 80%
- Average response time > 200ms
- Failed auth attempts > 100/hour

**Information Alerts** (Slack):
- Daily metrics summary
- Weekly capacity planning
- New deployment notifications

**Status**: 🔲 Not Started
**Owner**: DevOps
**Effort**: 4 hours

---

## Week 6: Security Hardening

### 2.1 SSL/HTTPS Setup

**Implementation Steps**:
1. Generate self-signed cert (testing) or Let's Encrypt (production)
2. Configure reverse proxy (nginx)
3. Force HTTPS redirect
4. Set HSTS headers

**Commands**:
```bash
# Generate self-signed cert (testing)
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

# Or use Let's Encrypt
certbot certonly --standalone -d vpn-sultan.com
```

**Status**: 🔲 Not Started
**Owner**: Security
**Effort**: 3 hours

---

### 2.2 Security Headers

**Headers to Add**:
```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'self'
Strict-Transport-Security: max-age=31536000; includeSubDomains
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: geolocation=(), microphone=(), camera=()
```

**Status**: 🔲 Not Started
**Owner**: Backend
**Effort**: 2 hours

---

### 2.3 Rate Limiting

**Configuration**:
- Per-IP: 1000 req/hour
- Per-user: 10000 req/day
- Per-endpoint: Vary based on sensitivity
- Authentication: 5 failures/minute before lockout

**Implementation**: Add rate limiting middleware to API

**Status**: 🔲 Not Started
**Owner**: Backend
**Effort**: 4 hours

---

### 2.4 CORS Configuration

**Configuration**:
```
Access-Control-Allow-Origins: https://vpn-sultan.com, https://admin.vpn-sultan.com
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization, X-Requested-With
Access-Control-Max-Age: 86400
```

**Status**: 🔲 Not Started
**Owner**: Backend
**Effort**: 1 hour

---

## Week 7: Performance Optimization

### 3.1 Database Optimization

**Tasks**:
1. Create indexes on frequently queried columns
2. Analyze slow queries (> 100ms)
3. Configure connection pooling
4. Set up query caching layer

**Index Strategy**:
```sql
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
CREATE INDEX idx_vpn_connections_user ON vpn_connections(user_id);
CREATE INDEX idx_audit_log_timestamp ON audit_log(created_at);
```

**Status**: 🔲 Not Started
**Owner**: Database
**Effort**: 6 hours

---

### 3.2 Redis Caching

**Cache Strategy**:
- Session data (TTL: 24 hours)
- User profiles (TTL: 1 hour)
- VPN server lists (TTL: 6 hours)
- API responses (TTL: 5 minutes)
- Rate limiting counters (TTL: 1 hour)

**Configuration**:
```
REDIS_URL=redis://localhost:6379/0
CACHE_TTL_SHORT=300
CACHE_TTL_MEDIUM=3600
CACHE_TTL_LONG=86400
```

**Status**: 🔲 Not Started
**Owner**: Backend
**Effort**: 8 hours

---

### 3.3 CDN for Static Assets

**Configuration**:
- Cloudflare or CloudFront for static content
- Gzip compression enabled
- Minified CSS/JS
- Image optimization (WebP format)
- Cache headers: max-age=31536000

**Expected Impact**:
- Frontend load time: 2.8s → 1.5s (46% improvement)
- Bandwidth usage: -35%
- Server load: -40%

**Status**: 🔲 Not Started
**Owner**: DevOps
**Effort**: 4 hours

---

## Week 8: Operational Excellence

### 4.1 Automated Backups

**Strategy**:
- Daily full backup at 2 AM UTC
- Hourly incremental backups
- 7-day retention (local)
- 30-day retention (offsite)
- Monthly backup for compliance

**Script**: `scripts/backup-database.sh`
```bash
#!/bin/bash
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="/var/backups/vpn-db-${TIMESTAMP}.sql.gz"

pg_dump -U vpn vpn_service | gzip > "$BACKUP_FILE"
s3cmd put "$BACKUP_FILE" s3://vpn-backups/

# Keep only 7 days of local backups
find /var/backups -name "vpn-db-*.sql.gz" -mtime +7 -delete
```

**Status**: 🔲 Not Started
**Owner**: DevOps
**Effort**: 3 hours

---

### 4.2 Incident Response Playbooks

**Critical Playbooks**:

1. **API Down - No Response**
   - Check API container status
   - Check database connectivity
   - Review recent logs
   - Restart service if needed
   - Escalate if unresolved in 10 minutes

2. **High Error Rate (> 5%)**
   - Check error logs (Loki)
   - Identify affected endpoints
   - Check database performance
   - Review recent deployments
   - Rollback if needed

3. **Database Connection Pool Exhausted**
   - Check active connections
   - Identify long-running queries
   - Restart service if hung connections
   - Increase pool size if needed

4. **Out of Disk Space**
   - Check disk usage by service
   - Remove old logs if needed
   - Review database size
   - Plan capacity upgrade

**Status**: 🔲 Not Started
**Owner**: SRE
**Effort**: 4 hours

---

### 4.3 Performance Baseline Report

**Metrics to Capture** (for comparison):

| Metric | Phase 1 | Phase 2 Target | Notes |
|--------|---------|--------|-------|
| API Uptime | 99.95% | 99.97% | Add HA |
| Error Rate | 0.06% | 0.05% | Reduce errors |
| P95 Latency | 145ms | 100ms | Optimize DB |
| P99 Latency | 250ms | 150ms | Cache layer |
| Throughput | 500 req/s | 1000 req/s | 2x capacity |
| DB Conn Pool | 35% util | <50% util | Monitor growth |
| Cache Hit Rate | - | >70% | New metric |

**Status**: 🔲 Not Started
**Owner**: Analytics
**Effort**: 2 hours

---

## 🎯 Phase 2 Success Criteria

- ✅ Prometheus + Grafana fully operational
- ✅ Loki logging aggregation working
- ✅ AlertManager with 5+ critical alerts
- ✅ SSL/HTTPS enabled
- ✅ Rate limiting implemented
- ✅ Database optimized with indexes
- ✅ Redis caching active
- ✅ Automated backups running
- ✅ Incident response playbooks documented
- ✅ Performance baseline captured
- ✅ All 15 smoke tests still passing
- ✅ Zero critical incidents

---

## 📊 Expected Outcomes

**Performance Improvements**:
- API latency: 145ms → 100ms (-31%)
- Frontend load: 2.8s → 1.5s (-46%)
- Database throughput: +50%
- Uptime: 99.95% → 99.97%

**Operational Improvements**:
- MTTR (Mean Time to Resolution): 30min → 10min
- Alert accuracy: 90% → 95% (less false positives)
- Observability: Current visibility → Full system tracing

**Scaling Headroom**:
- Current: 30 concurrent users
- Phase 2: 100+ concurrent users
- Phase 3 Ready: Infrastructure for 500+ users

---

## 📅 Next Steps

1. **This Week**: Deploy monitoring (Prometheus/Grafana)
2. **Next Week**: Implement security hardening
3. **Week 7-8**: Performance optimization + operational setup
4. **End of Month**: Review metrics, plan Phase 3

---

**Owner**: DevOps Team
**Status**: Ready to Start ✅
**Last Updated**: March 28, 2026
