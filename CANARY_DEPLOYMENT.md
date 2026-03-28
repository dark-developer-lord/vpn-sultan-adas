# Canary Deployment Strategy

## Overview

Canary deployments allow us to roll out new versions gradually while monitoring the health of the new version before shifting 100% of traffic to it.

## Deployment Phases

### Phase 1: Canary Release (5% Traffic)
- Deploy new version to canary pods (2-3 replicas)
- Route 5% of traffic to new version
- Duration: 5-10 minutes
- Monitoring: All metrics, error rates, latency
- Rollback if: Error rate > 1%, P95 latency > 2x baseline

### Phase 2: Early Adoption (25% Traffic)
- Route 25% of traffic to new version
- Duration: 15-30 minutes
- Monitoring: Business metrics, user experience
- Rollback if: Same thresholds as Phase 1

### Phase 3: Progressive Rollout (50% Traffic)
- Route 50% of traffic to new version
- Duration: 30-60 minutes
- Monitoring: Extended validation on production workloads
- Rollback if: Same thresholds as Phase 1

### Phase 4: Full Deployment (100% Traffic)
- Route 100% of traffic to new version
- Keep old version running for 30 minutes
- Monitoring: Full production traffic validation
- Rollback if: Critical issues detected

### Post-Deployment (Maintenance)
- Keep old version for 24 hours
- Monitor for delayed issues
- Retain all deployment artifacts for 1 week

## Health Check Thresholds

```yaml
canary_thresholds:
  error_rate:
    absolute: 0.01  # 1%
    relative: 1.5   # 1.5x baseline
  latency:
    p50: 1.2x baseline
    p95: 1.5x baseline
    p99: 2.0x baseline
  custom_metrics:
    business_errors: 0.05  # 5%
    payment_failures: 0.02  # 2%
```

## Rollback Triggers

| Metric | Threshold | Action |
|--------|-----------|--------|
| Error Rate | > 1% | Immediate rollback |
| P95 Latency | > 2x baseline | Immediate rollback |
| OOM Events | Any | Immediate rollback |
| Database Connections | > 90% | Immediate rollback |
| CPU | > 85% sustained | Progressive rollback |
| Memory | > 80% sustained | Progressive rollback |

## Deployment Process

### Pre-Deployment

1. **Build & Test**
   ```bash
   # Run all tests
   cargo test
   # Build Docker image
   docker build -t vpn-api:v1.2.0 .
   # Push to ECR
   aws ecr push vpn-api:v1.2.0
   ```

2. **Pre-deployment Checks**
   ```bash
   # Security scan
   trivy image vpn-api:v1.2.0
   
   # Load test (staging)
   k6 run tests/load-test.js --vus 100 --stage 60s
   
   # Smoke test (staging)
   k6 run tests/smoke-test.js
   ```

3. **Update Monitoring**
   ```bash
   # Update dashboards
   # Update alert thresholds
   # Notify on-call team
   ```

### Deployment Execution

1. **Start Canary (5%)**
   ```bash
   kubectl patch deployment vpn-api -p \
     '{"spec":{"template":{"metadata":{"annotations":{"version":"v1.2.0"}}}}}'
   
   # Wait 30 seconds for pods to start
   sleep 30
   
   # Update traffic split to 5%
   kubectl patch vs vpn-gateway -p \
     '{"spec":{"hosts":[{"name":"vpn-api.example.com","http":[{"match":[{"uri":{"prefix":"/"}}],"route":[{"destination":{"host":"vpn-api-stable","port":{"number":8080}},"weight":95},{"destination":{"host":"vpn-api-canary","port":{"number":8080}},"weight":5}]}]}}'
   ```

2. **Monitor Canary (5-10 min)**
   ```bash
   # Watch metrics
   watch -n 5 'kubectl top pods -l version=v1.2.0'
   
   # Check logs
   kubectl logs -f deployment/vpn-api -l version=v1.2.0
   
   # Dashboard: http://grafana.example.com/d/canary
   ```

3. **Decision: Continue to 25%**
   ```bash
   # If metrics are good:
   kubectl patch vs vpn-gateway -p '{"spec":{"hosts":[{"name":"vpn-api.example.com","http":[{"match":[{"uri":{"prefix":"/"}}],"route":[{"destination":{"host":"vpn-api-stable","port":{"number":8080}},"weight":75},{"destination":{"host":"vpn-api-canary","port":{"number":8080}},"weight":25}]}]}}'
   ```

4. **Monitor at 25% (15-30 min)**

5. **Decision: Continue to 50%**

6. **Monitor at 50% (30-60 min)**

7. **Decision: Continue to 100%**

8. **Final Verification (30 min)**
   - Keep stable version running
   - Monitor for anomalies
   - After 30 min, scale down stable version

### Immediate Rollback

```bash
#!/bin/bash
# Rollback script
kubectl patch vs vpn-gateway -p \
  '{"spec":{"hosts":[{"name":"vpn-api.example.com","http":[{"match":[{"uri":{"prefix":"/"}}],"route":[{"destination":{"host":"vpn-api-stable","port":{"number":8080}},"weight":100}]}]}}'

# Delete canary pods
kubectl delete pods -l version=v1.2.0

# Verify traffic is back to stable
sleep 10
curl -s http://vpn-api.example.com/health | jq .

# Alert team
curl -X POST $SLACK_WEBHOOK -d '{"text":"Deployment v1.2.0 rolled back due to error rate spike"}'
```

## Artifacts & History

### Deployment Record

```yaml
deployment:
  version: v1.2.0
  timestamp: "2024-01-15T14:30:00Z"
  deployed_by: devops-ci
  image_sha: "sha256:abc123..."
  phases:
    - phase: canary
      traffic: 5%
      duration: 8m
      status: success
    - phase: early_adoption
      traffic: 25%
      duration: 20m
      status: success
    - phase: progressive
      traffic: 50%
      duration: 45m
      status: success
    - phase: full
      traffic: 100%
      duration: 30m
      status: success
  metrics:
    error_rate: 0.002
    p95_latency: 45ms
    peak_throughput: 2500 req/s
```

## Monitoring Dashboard

Canary deployment monitoring dashboard includes:

- **Real-time Traffic Split**: Shows % routed to canary vs stable
- **Error Rates**: Comparison between canary and stable
- **Latency**: P50, P95, P99 for both versions
- **Resource Usage**: CPU, memory, network for canary
- **Custom Metrics**: Business-specific KPIs
- **Logs**: Filtered logs for canary version only
- **Predefined Alerts**: Triggers for automatic rollback

## Post-Deployment Actions

1. **30 Minutes After Full Deployment**
   - Scale down old version
   - Clean up canary resources
   - Document deployment in change log

2. **24 Hours After Deployment**
   - Archive logs and metrics
   - Delete old image from ECR (keep for 1 week)
   - Close deployment ticket

3. **1 Week After Deployment**
   - Final verification of stability
   - Performance comparison with previous version
   - Delete old resources

## Example: k6 Canary Monitoring

```javascript
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  stages: [
    { duration: '5m', target: 100 },   // Canary: 100 users
    { duration: '5m', target: 100 },   // Early adoption: stay at 100
  ],
  thresholds: {
    'http_req_duration': ['p(95)<500', 'p(99)<1000'],
    'http_req_failed': ['rate<0.01'],
  },
};

export default function () {
  let res = http.get('http://vpn-api.example.com/api/users');
  
  check(res, {
    'status is 200': (r) => r.status === 200,
    'latency < 500ms': (r) => r.timings.duration < 500,
  });
  
  // Check for deployment version
  console.log(`Response time: ${res.timings.duration}ms`);
}
```

## Failure Scenarios

### Scenario 1: High Error Rate

```
Time: 5m into Phase 1 (Canary, 5%)
Error Rate: 2.5% (threshold: 1%)

Action: IMMEDIATE ROLLBACK
1. Traffic shifted to 0% canary
2. Canary pods terminated
3. Alert sent to team
4. Incident created
5. Team investigates logs
6. Fix deployed as v1.2.1
```

### Scenario 2: Memory Leak

```
Time: 15m into Phase 2 (Early Adoption, 25%)
Memory: Increasing 5MB/min
OOM Prediction: 45 minutes

Action: GRADUAL ROLLBACK
1. Traffic reduced from 25% to 5%
2. Monitor memory trend
3. If trend continues: Full rollback
4. Team investigates heap dumps
5. Fix deployed after analysis
```

### Scenario 3: Database Connection Exhaustion

```
Time: 30m into Phase 3 (Progressive, 50%)
DB Connections: 85% of max, rising

Action: PHASE HOLD
1. Keep at 50%, don't progress
2. Open DB connection investigation
3. If stable for 15m: Continue
4. If rising: Begin gradual rollback
```

## Success Criteria

✅ Deployment is successful if:
- All 4 phases complete without rollback
- Error rate stays < 1% in each phase
- Latency remains within baseline
- No OOM or resource exhaustion
- User-reported issues = 0
- Business metrics maintained

## Team Responsibilities

| Role | Responsibility |
|------|-----------------|
| DevOps | Manage deployment process, monitor phases |
| SRE | Monitor infrastructure, respond to alerts |
| Backend Team | On-call for code issues, ready to fix |
| Product | Monitor business metrics, user impact |
| Incident Commander | Coordinates escalation if needed |

## Related Documentation

- [Incident Response Playbook](incident-response-playbook.md)
- [Monitoring Guide](monitoring-guide.md)
- [Alerting Rules](alert-rules.yml)
- [Rollback Procedures](rollback-procedures.md)
