# VPN Service Metrics & Alerting Guide

Comprehensive reference for monitoring metrics, creating dashboards, and managing alerts.

---

## 📊 Architecture Overview

```
Application
    ↓
Prometheus (/metrics endpoint)
    ↓
Prometheus Server (9090)
    ↓
├─ AlertManager (9093)
│  ├─ Email Notifications
│  ├─ Slack Webhooks
│  └─ PagerDuty Integration
│
├─ Grafana (3001)
│  ├─ Dashboards
│  ├─ Graphs
│  └─ Alerts
│
└─ Data Storage
   ├─ Prometheus DB (15 days retention)
   └─ Long-term storage (if configured)

Loki (3100) ← Logs from all services
    ↓
Promtail (ships logs)
    ↓
Grafana Explore (query logs)
```

---

## 📈 Key Metrics to Monitor

### API Metrics

**HTTP Requests**:
```prometheus
http_requests_total{endpoint="/health", method="GET"}
# Counter: Total requests since startup
# Alert if: Missing data for 5 minutes (service may be down)
```

**HTTP Errors**:
```prometheus
http_errors_total{status="500"}
# Counter: Total errors by status code
# Alert if: Error rate > 5% for 5 minutes (P1 response time!)
```

**Request Duration**:
```prometheus
http_request_duration_seconds_bucket
# Histogram: Request latency distribution
# Query: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))
# Alert if: P95 latency > 500ms for 10 minutes
```

**Request Rate by Endpoint**:
```prometheus
rate(http_requests_total[5m])
# Requests per second by endpoint
# Use to detect: Traffic spikes, DDoS, unusual patterns
```

### Database Metrics (PostgreSQL Exporter)

**Connection Count**:
```prometheus
pg_stat_activity_count
# Current active connections
# Alert if: > 90% of max_connections
```

**Database Size**:
```prometheus
pg_database_size_bytes
# Total database size in bytes
# Alert if: Growing > 1GB per day (unusual data growth)
```

**Cache Hit Ratio**:
```prometheus
rate(pg_stat_database_blks_hit[5m]) / (rate(pg_stat_database_blks_hit[5m]) + rate(pg_stat_database_blks_read[5m]))
# Cache effectiveness: 95%+ is good
# Alert if: < 80% (cache misses indicate performance issues)
```

**Query Duration**:
```prometheus
pg_stat_statements_mean_exec_time
# Average query execution time in milliseconds
# Alert if: Any query > 1000ms average
```

**Active Queries**:
```prometheus
pg_stat_activity_count{state="active"}
# Queries currently executing
# Alert if: > 5 long-running queries detected
```

### Redis Metrics

**Connected Clients**:
```prometheus
redis_connected_clients
# Number of connected clients
# Normal: 1-5 during operations
```

**Memory Usage**:
```prometheus
redis_memory_used_bytes
# Redis memory consumption
# Alert if: > 80% of max memory policy
```

**Eviction Rate**:
```prometheus
rate(redis_evicted_keys_total[5m])
# Keys evicted due to memory pressure
# Alert if: > 0 (indicates insufficient cache memory)
```

**Keyspace Hits/Misses**:
```prometheus
rate(redis_keyspace_hits_total[5m]) / (rate(redis_keyspace_hits_total[5m]) + rate(redis_keyspace_misses_total[5m]))
# Cache hit ratio for Redis
# Target: > 90%
```

### System Metrics

**CPU Usage**:
```prometheus
100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)
# CPU utilization percentage
# Alert if: > 80% for 10 minutes
```

**Memory Usage**:
```prometheus
(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100
# Memory utilization percentage
# Alert if: > 85% for 5 minutes
```

**Disk Usage**:
```prometheus
(node_filesystem_size_bytes - node_filesystem_avail_bytes) / node_filesystem_size_bytes * 100
# Disk utilization percentage
# Alert if: > 85% (critical at 90%)
```

**Network I/O**:
```prometheus
rate(node_network_receive_bytes_total[5m])
rate(node_network_transmit_bytes_total[5m])
# Network bandwidth usage
# Use to detect: DDoS, data exfiltration, backup transfers
```

---

## 🚨 Alert Rules

Located in: `monitoring/alert-rules.yml`

### Critical Alerts (P1 - Page immediately)

**APIDown**:
```yaml
alert: APIDown
expr: up{job="vpn-api"} == 0
for: 2m
labels:
  severity: critical
  component: api
annotations:
  summary: "API service down"
  description: "API has been unavailable for 2 minutes"
```
**Action**: See [API Down Incident Response](incident-response-playbooks.md#-p1-incident-api-service-down-no-response)

**HighErrorRate**:
```yaml
alert: HighErrorRate
expr: (rate(http_errors_total[5m]) / rate(http_requests_total[5m])) > 0.05
for: 5m
labels:
  severity: critical
  component: api
annotations:
  summary: "High error rate detected"
  description: "Error rate is {{ $value | humanizePercentage }} for 5 minutes"
```
**Action**: See [Error Rate Spike Response](incident-response-playbooks.md#-p1-incident-error-rate-spike--5-for-5-min)

**HighLatency**:
```yaml
alert: HighLatency
expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 0.5
for: 10m
labels:
  severity: critical
  component: api
annotations:
  summary: "High API latency detected"
  description: "P95 latency is {{ $value | humanizeDuration }}"
```
**Action**: See [Latency Response](incident-response-playbooks.md#-p2-incident-high-api-latency-p95--500ms)

**DatabaseConnectionPoolExhausted**:
```yaml
alert: DatabaseConnectionPoolExhausted
expr: pg_stat_activity_count > 90
for: 5m
labels:
  severity: critical
  component: database
annotations:
  summary: "Database connection pool nearly exhausted"
  description: "{{ $value }} of 100 connections in use"
```
**Action**: See [Connection Pool Response](incident-response-playbooks.md#-p2-incident-database-connections-exhausted)

### Major Alerts (P2 - Respond within 30 min)

**CacheMissRateHigh**:
```yaml
alert: CacheMissRateHigh
expr: (rate(pg_stat_database_blks_read[5m]) / (rate(pg_stat_database_blks_hit[5m]) + rate(pg_stat_database_blks_read[5m]))) > 0.2
for: 15m
labels:
  severity: warning
  component: database
annotations:
  summary: "Database cache hit ratio below 80%"
  description: "Cache miss rate is {{ $value | humanizePercentage }}"
```

**SlowQueriesDetected**:
```yaml
alert: SlowQueriesDetected
expr: pg_stat_statements_mean_exec_time > 1000
for: 10m
labels:
  severity: warning
  component: database
annotations:
  summary: "Slow queries detected"
  description: "Average query time is {{ $value }}ms"
```

**DiskSpaceCritical**:
```yaml
alert: DiskSpaceCritical
expr: (1 - (node_filesystem_avail_bytes / node_filesystem_size_bytes)) > 0.85
for: 5m
labels:
  severity: critical
  component: system
annotations:
  summary: "Disk space critical"
  description: "Disk {{ $labels.device }} is {{ $value | humanizePercentage }} full"
```

**HighMemoryUsage**:
```yaml
alert: HighMemoryUsage
expr: (1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) > 0.85
for: 5m
labels:
  severity: warning
  component: system
annotations:
  summary: "High memory usage detected"
  description: "Memory usage is {{ $value | humanizePercentage }}"
```

### Minor Alerts (P3 - Resolve within 24h)

**BackupFailure**:
```yaml
alert: BackupFailure
expr: (time() - backup_last_success_timestamp) > 86400
for: 1m
labels:
  severity: warning
  component: backup
annotations:
  summary: "Backup failed or missing"
  description: "No successful backup in the last 24 hours"
```

**PrometheusScrapeFailed**:
```yaml
alert: PrometheusScrapeFailed
expr: up == 0
for: 10m
labels:
  severity: warning
components: monitoring
annotations:
  summary: "Prometheus scrape failed for {{ $labels.job }}"
  description: "Target has been down for 10 minutes"
```

---

## 📊 Creating Custom Dashboards

### Access Grafana

1. Navigate to `http://187.124.179.20:3001`
2. Login with credentials (default: admin/admin)
3. Click "+" → "Dashboard"
4. Click "Add Panel"

### Common Dashboard Panels

**Request Rate (Graph)**:
```
Metric: rate(http_requests_total[5m])
Legend: {{endpoint}}
Y-axis: requests/sec
```

**Error Count (Graph)**:
```
Metric: rate(http_errors_total[5m])
Legend: {{status}} {{endpoint}}
Y-axis: errors/sec
```

**Latency P50/P95/P99 (Multi-line Graph)**:
```
P50: histogram_quantile(0.50, rate(http_request_duration_seconds_bucket[5m]))
P95: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))
P99: histogram_quantile(0.99, rate(http_request_duration_seconds_bucket[5m]))
Y-axis: duration (seconds)
```

**Database Connections (Gauge)**:
```
Metric: pg_stat_activity_count
Max: 100 (max_connections)
Thresholds: Green < 70, Yellow < 90, Red >= 90
```

**Cache Hit Ratio (Gauge)**:
```
Metric: rate(pg_stat_database_blks_hit[5m]) / (rate(pg_stat_database_blks_hit[5m]) + rate(pg_stat_database_blks_read[5m]))
Format: Percent (0-100)
Thresholds: Green > 95, Yellow > 80, Red < 80
```

**Disk Usage (Pie Chart)**:
```
Metric: node_filesystem_avail_bytes / node_filesystem_size_bytes
By: device
Format: Percent
```

---

## 🔔 Managing Alerts

### Silence Alert

Temporarily disable an alert (e.g., during maintenance):

```bash
curl -X POST http://localhost:9093/api/v1/silences -d '{
  "matchers": [
    {
      "name": "alertname",
      "value": "APIDown",
      "isRegex": false
    }
  ],
  "startsAt": "2024-01-15T10:00:00Z",
  "endsAt": "2024-01-15T12:00:00Z",
  "createdBy": "operator",
  "comment": "Maintenance window"
}'
```

### View All Active Alerts

```bash
curl -s http://localhost:9093/api/v1/alerts | jq '.data[] | {alertname: .labels.alertname, status: .status, severity: .labels.severity}'
```

### Resolve Alert

Alerts auto-resolve when condition clears. To force resolution:

1. Access AlertManager UI: `http://localhost:9093`
2. Click alert
3. Click "Resolve"

### Create Alert Notification Channel

**Email**:
Edit `monitoring/alertmanager.yml`:
```yaml
route:
  group_by: ['alertname', 'cluster']
  receiver: 'email-ops'

receivers:
- name: 'email-ops'
  email_configs:
  - to: 'ops@vpn-service.local'
    from: 'alertmanager@vpn-service.local'
    smarthost: 'smtp.gmail.com:587'
    auth_username: 'alertmanager@gmail.com'
    auth_password: 'app-specific-password'
```

**Slack**:
```yaml
receivers:
- name: 'slack-ops'
  slack_configs:
  - api_url: 'https://hooks.slack.com/services/YOUR/WEBHOOK/URL'
    channel: '#vpn-alerts'
    title: 'VPN Service Alert'
    text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'
```

---

## 📈 Performance Baselines

### Expected Metrics (Normal Operation)

| Metric | Normal | Warning | Critical |
|--------|--------|---------|----------|
| Error Rate | < 1% | 1-5% | > 5% |
| P95 Latency | < 100ms | 100-500ms | > 500ms |
| Cache Hit Ratio | > 95% | 80-95% | < 80% |
| DB Connections | 5-20 | 20-80 | > 90 |
| CPU Usage | < 50% | 50-80% | > 80% |
| Memory Usage | < 60% | 60-85% | > 85% |
| Disk Usage | < 70% | 70-85% | > 85% |

### Load Testing Results

From synthetic tests (100 concurrent users for 10 minutes):

| Metric | 30 Users | 100 Users | 500 Users |
|--------|----------|-----------|-----------|
| P50 Latency | 45ms | 120ms | 450ms |
| P95 Latency | 95ms | 280ms | 800ms |
| P99 Latency | 120ms | 400ms | 1200ms |
| Error Rate | 0.1% | 0.5% | 2.3% |
| Requests/sec | 180 | 480 | 900 |
| Cache Hit Ratio | 98% | 97% | 94% |

---

## 🔍 Querying Metrics

### Example Prometheus Queries

**Requests per endpoint (5 min avg)**:
```promql
topk(5, rate(http_requests_total[5m]))
```

**Error percentage by status code**:
```promql
sum(rate(http_errors_total[5m])) by (status) / ignoring(status) sum(rate(http_requests_total[5m]))
```

**Slow database queries (>100ms)**:
```promql
histogram_quantile(0.95, rate(pg_stat_statements_exec_time_seconds_bucket[5m])) > 0.1
```

**Redis eviction rate**:
```promql
rate(redis_evicted_keys_total[5m])
```

**System resource utilization**:
```promql
100 * (1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes))
```

### Example Loki Queries

**All errors from API**:
```logql
{job="vpn-api"} | json | status >= 500
```

**Errors for specific user**:
```logql
{job="vpn-api"} | json | user_id = "12345"
```

**Response time buckets**:
```logql
{job="vpn-api"} | json | duration_ms > 1000
```

**Database connection errors**:
```logql
{job="vpn-api"} | "connection refused"
```

---

## 📋 Monitoring Troubleshooting

### Prometheus Not Scraping

1. **Check config**:
   ```bash
   docker-compose exec prometheus cat /etc/prometheus/prometheus.yml
   ```

2. **Reload config**:
   ```bash
   curl -X POST http://localhost:9090/-/reload
   ```

3. **Check targets**:
   ```bash
   curl -s http://localhost:9090/api/v1/targets | jq '.data.activeTargets'
   ```

### Metrics Endpoint Returning 429 (Rate Limited)

The /metrics endpoint has rate limiting enabled for security. If rate limited:

```bash
# Wait 60 seconds for per-minute limit reset
# Or for hourly: wait up to 3600 seconds

# Check current limits
curl http://localhost:3000/metrics
```

### Loki Not Receiving Logs

1. **Check Promtail connectivity**:
   ```bash
   docker-compose logs promtail | grep -i error
   ```

2. **Verify Loki is running**:
   ```bash
   curl -s http://localhost:3100/ready
   ```

3. **Check service logs**:
   ```bash
   docker-compose logs loki | tail -20
   ```

---

**Last Updated**: March 28, 2026
**Owner**: DevOps Team
**Review Frequency**: Monthly
