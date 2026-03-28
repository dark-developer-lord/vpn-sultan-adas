# Incident Response Playbook

## Incident Severity Levels

### Level 1: Critical
- **Definition**: Core service completely down, all users affected
- **Examples**: API server crash, database failure, all traffic blocked
- **Response Time**: < 5 minutes
- **Pages**: On-call team, Manager, CTO
- **Meeting**: War room with full team

### Level 2: Major
- **Definition**: Service significantly degraded, most users affected
- **Examples**: 50% error rate, 10x latency spike, payment processing down
- **Response Time**: < 15 minutes
- **Pages**: On-call team, team lead
- **Meeting**: Team coordination on Slack/phone

### Level 3: Minor
- **Definition**: Service partially degraded, subset of users affected
- **Examples**: 5% error rate, single feature broken, one region down
- **Response Time**: < 1 hour
- **Pages**: Relevant team lead
- **Meeting**: None required, async updates

### Level 4: Trivial
- **Definition**: Non-critical issue, minimal user impact
- **Examples**: Documentation error, minor bug, cosmetic issue
- **Response Time**: Standard ticket process
- **Pages**: None
- **Meeting**: None

## Incident Response Flow

```
Detection/Report
    ↓
[1] Declare Incident (severity)
    ↓
[2] Alert On-Call Team
    ↓
[3] Page Manager/CTO (if Level 1-2)
    ↓
[4] Gather Information
    ↓
[5] Investigate Root Cause
    ↓
[6] Implement Fix/Workaround
    ↓
[7] Validate Resolution
    ↓
[8] Communicate Status
    ↓
[9] Document Incident
    ↓
[10] Post-Mortem Review
```

## Detection & Initial Response

### 1. Alert Detected

**Grafana Alert** → Alertmanager → Slack #alerts-critical

```
Red Alert: API Error Rate > 5% (Actual: 8.2%)
Time: 2024-01-15 14:35:22 UTC
Graph: [link]
Services: vpn-api (3/5 instances failing)
```

### 2. Declare Incident

```bash
# Create incident in incident tracking system
curl -X POST https://incidents.company.com/api/incidents \
  -d '{
    "title": "API Error Rate Spike",
    "severity": "critical",
    "service": "vpn-api",
    "started_at": "2024-01-15T14:35:22Z",
    "description": "Error rate spike to 8.2%, 3/5 API instances failing",
    "detection_method": "Prometheus alert"
  }'
```

### 3. Alert On-Call Team

**Automatic via PagerDuty**:
- SMS to on-call engineer
- Email to on-call team
- Slack message to #incidents
- Alert email to escalation contacts

**Manual Notification** (if alert system fails):
```bash
# Send Slack message
curl -X POST $SLACK_WEBHOOK \
  -d '{"text":"🚨 CRITICAL: API Error Rate > 5% - Check #incidents-critical"}'

# Call on-call engineer
# [Use phone tree if SMS not acknowledged within 2 min]
```

## Investigation Phase

### 4. Gather Information

**On-Call Engineer Joins**:
```bash
# Join war room
# Slack thread or conference call

# Gather metrics
curl -s 'http://localhost:9090/api/v1/query?query=http_requests_total{status=~"5.."}' | jq .
curl -s 'http://localhost:9090/api/v1/query?query=up{job="vpn-api"}' | jq .

# Check service status
systemctl status vpn-api
journalctl -u vpn-api -n 100 -f

# Check database
psql -c "SELECT pid, query, state FROM pg_stat_activity;"

# Check capacity
free -h
df -h
top
```

### 5. Root Cause Analysis

**Common Causes Matrix**:

| Symptom | Likely Causes | Check |
|---------|--------------|-------|
| Error Rate High | Code bug, bad deployment, DB down, memory leak | Logs, recent changes, DB status |
| Latency High | CPU maxed, DB slow, network issue | top, DB queries, network |
| Memory Spike | Memory leak, high load, data structure issue | Memory profiler, heap dump, load |
| Pod Crashes | OOM, panic, liveliness check fail | Logs, resource limits, healthchecks |

**Investigation Checklist**:
- [ ] Check recent deployments: `git log -5 --oneline`
- [ ] Review recent changes: `git diff HEAD~1`
- [ ] Check pod logs: `kubectl logs -f pod-name`
- [ ] Check system resources: `top`, `free -h`, `df -h`
- [ ] Check database: `psql -c "SELECT * FROM pg_stat_activity;"`
- [ ] Check Redis: `redis-cli info stats`
- [ ] Check network: `netstat -an | grep ESTABLISHED | wc -l`
- [ ] Check error logs: `grep -E "PANIC|FATAL|ERROR" app.log | head -20`

## Resolution Phase

### 6. Implement Fix/Workaround

#### Scenario A: Memory Leak in v1.2.0

```bash
# Immediate Workaround
# Roll back to v1.1.0
./scripts/rollback.sh v1.1.0

# Verify
curl -s http://localhost:8080/health | jq .
wait 30 seconds
# Monitor error rate dropback to normal
curl 'http://localhost:9090/api/v1/query?query=rate(http_requests_total{status=~"5.."}[5m])'
```

#### Scenario B: Database Connection Pool Exhausted

```bash
# Immediate Mitigation
# Kill idle connections
psql -c "
SELECT pid FROM pg_stat_activity
WHERE state = 'idle'
  AND state_change < now() - INTERVAL '10 minutes'
  AND usename != 'postgres'
LIMIT 20;" | while read pid; do
  psql -c "SELECT pg_terminate_backend($pid);"
done

# Verify pool recovered
psql -c "SELECT count(*) FROM pg_stat_activity WHERE state = 'active';"
```

#### Scenario C: DDoS Attack

```bash
# Working with infrastructure team
# Add rate limiting rules
mysql cloudflare-api \
  -e "INSERT INTO ddos_protection (pattern, limit_rps, action) 
       VALUES ('*', 1000, 'BLOCK')"

# Enable geographic limiting
# Block traffic from suspicious regions
curl -X POST https://api.cloudflare.com/client/v4/zones/ddos-rules \
  -d '{"countries": ["XX", "YY"], "action": "block"}'
```

#### Scenario D: Bug in New Code

```bash
# Based on error logs, identified cause: NULL pointer in payment processing

# Option 1: Fix in branch and deploy (if quick)
git checkout -b hotfix/payment-null-check
# Make fix
git commit -am "Fix: null check in payment processor"
cargo build --release
# Deploy via canary

# Option 2: Feature flag rollback (if available)
FEATURE_FLAGS={"new_payment_flow": false}
systemctl restart vpn-api

# Option 3: Temporary workaround (if both above not viable)
# Add error handling middleware to catch and gracefully fail
sys config set "error_handling.strict_mode=false"
```

### 7. Validate Resolution

```bash
# After implementing fix:

# 1. Check error rate (should drop immediately)
curl -s 'http://localhost:9090/api/v1/query?query=rate(http_requests_total{status=~"5.."}[5m])' | jq '.data.result[0].value[1]'
# Expected: < 0.01 (< 1%)

# 2. Check latency (should normalize)
curl -s 'http://localhost:9090/api/v1/query?query=histogram_quantile(0.95,http_request_duration_seconds_bucket)' | jq .
# Expected: < 500ms

# 3. Run quick smoke tests
k6 run tests/smoke-test.js --vus 5 --duration 30s --api-url http://localhost:8080

# 4. Check user-facing endpoints
for endpoint in /api/users /api/payments /api/status; do
  curl -s http://localhost:8080$endpoint | jq .
done

# 5. Monitor for 5 minutes
sleep 5m
curl -s 'http://localhost:9090/api/v1/query?query=rate(http_requests_total{status=~"5.."}[5m])' | jq '.data.result[0].value[1]'
# Verify rate remains low
```

## Communication Phase

### 8. Status Updates

**Timeline**:
- **T+0min**: Alert sent to channels
- **T+2min**: Initial status post: "Investigating API error rate spike"
- **T+5min**: Root cause identified: "Memory leak in v1.2.0"
- **T+10min**: Fix in progress: "Rolling back to v1.1.0"
- **T+15min**: Resolution verified: "Error rate normalized, incident ongoing monitoring"
- **T+30min**: All clear: "System stability confirmed"

**Status Template**:
```
🚨 INCIDENT: API Error Rate Spike

Status: 🔴 CRITICAL → 🟡 INVESTIGATING → 🟢 RESOLVED

Affected: All API endpoints
Started: 2024-01-15 14:35 UTC
Duration: 20 minutes

Update (14:50):
Root cause identified: Memory leak introduced in v1.2.0
Action taken: Rolled back to v1.1.0
Status: Error rates normalizing ✓
ETA: Stability confirmation within 10 minutes

Updates: Every 5 minutes in #incidents-critical
Incident Page: https://incidents.company.com/inc-2024-0115-001
```

## Post-Incident Phase

### 9. Document Incident

**Incident Report Template**:

```yaml
incident:
  id: INC-2024-0115-001
  title: Memory Leak in v1.2.0 - API Error Rate Spike
  severity: CRITICAL
  service: vpn-api
  status: RESOLVED
  
timeline:
  detected: "2024-01-15T14:35:22Z"
  declared: "2024-01-15T14:36:00Z"
  root_cause_found: "2024-01-15T14:40:15Z"
  fix_implemented: "2024-01-15T14:50:00Z"
  resolved: "2024-01-15T14:55:30Z"
  total_duration: "20 minutes 8 seconds"
  
impact:
  users_affected: "100%"
  systems_down: ["vpn-api (3/5 instances)"]
  peak_error_rate: "8.2%"
  peak_latency: "2500ms"
  transactions_failed: "~4,500"
  revenue_impact: "$150 (estimated)"
  
root_cause:
  component: "crates/api/src/handlers/payments.rs"
  issue: "Memory not released in payment processing loop"
  line: "234-241"
  introduced_in: "v1.2.0 (commit abc123f)"
  detection_lag: "2 hours (deployed at 12:30, detected at 14:35)"
  
resolution:
  type: "ROLLBACK"
  action: "Rolled back from v1.2.0 to v1.1.0"
  duration: "8 seconds"
  validation: "All metrics normalized within 2 minutes"
  
prevention:
  action_items:
    - "Add memory profiling to pre-deployment checks"
    - "Increase canary duration for payment processing features"
    - "Add automated OOM detection and alerts"
    - "Implement feature flags for payment updates"
```

### 10. Post-Mortem Review

**Schedule**: 24-48 hours after incident

**Participants**: On-call engineer, team lead, SRE, product

**Agenda**:
1. Timeline review (5 min)
2. Root cause deep dive (15 min)
3. Detection improvements (10 min)
4. Response process evaluation (10 min)
5. Action items assignment (10 min)

**Typical Action Items**:
- Add memory profiler to CI/CD pipeline
- Increase default canary duration
- Add OOM alerts
- Code review memory management
- Create runbook for memory leaks

**Never Blame**: Focus on systemic improvements, not individual mistakes

## Prevention Checklist

- [ ] Code review checklist includes memory management
- [ ] Load tests include memory profiling
- [ ] Pre-deployment checklist includes memory profiler run
- [ ] Canary deployments last minimum 30 minutes for major changes
- [ ] All feature flags documented and tracked
- [ ] Automated rollback tested monthly
- [ ] On-call playbooks reviewed quarterly
- [ ] Alert thresholds calibrated quarterly

## Quick Reference: Common Issues

### API Down
```bash
# Check process
systemctl status vpn-api

# Check logs
journalctl -u vpn-api -n 50

# Check port
netstat -tlnp | grep 8080

# Restart
sudo systemctl restart vpn-api
```

### Database Connection Issues
```bash
# Check connections
psql -c "SELECT count(*) FROM pg_stat_activity;"

# Kill idle
psql -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE state = 'idle';"

# Restart postgres
sudo systemctl restart postgresql
```

### High Memory Usage
```bash
# Top memory consumers
top -o %MEM -b -n 1 | head -10

# Memory per process
ps aux --sort -%mem | head -10

# Check for swapping
vmstat 1 5 | tail -4

# Restart service
sudo systemctl restart vpn-api
```

### High CPU Usage
```bash
# Top CPU consumers
top -o %CPU -b -n 1 | head -10

# Check system metrics
mpstat
sar -u 1 5

# Find hot spots
perf top
```

## Escalation Tree

**Level 1 Alert** → On-Call Engineer (5 min)
**Level 2 Alert** → Team Lead (10 min)  
**Level 3 Alert** → Engineering Manager (15 min)
**Level 4 Alert** → VP Engineering (30 min)

If unresolved after 15 minutes → Auto-escalate to next level
