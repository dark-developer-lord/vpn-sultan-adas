# VPN Service - Deployment Checklist

## Pre-Deployment Verification

### ✅ Code Quality Checks
- [ ] Backend compiles with `cargo check --bin vpn-api` (0 errors)
- [ ] All 22 integration tests pass: `cargo test --test integration_tests`
- [ ] Unit tests pass: `cargo test --lib`
- [ ] Frontend builds successfully: `npm run build`
- [ ] No TypeScript errors in frontend
- [ ] No security warnings in dependencies: `npm audit`
- [ ] Code follows project conventions
- [ ] All TODOs and FIXMEs are resolved

### ✅ Security Validation
- [ ] JWT secret is strong and not in version control
- [ ] Database password is strong and not in version control
- [ ] No sensitive data in logs
- [ ] CORS is properly configured for production domain
- [ ] HTTPS/TLS is enforced in production
- [ ] SQL injection prevention verified (sqlx queries)
- [ ] XSS protection enabled in Angular
- [ ] CSRF tokens implemented
- [ ] Rate limiting configured
- [ ] Password hashing algorithm is Argon2 (not plaintext)

### ✅ Database Validation
- [ ] All 9 tables created successfully
- [ ] Foreign key constraints verified
- [ ] Indices created on frequently queried columns
- [ ] Database backups are automated
- [ ] Database replication tested
- [ ] Connection pooling configured
- [ ] Query performance baseline established

### ✅ API Validation
- [ ] All 13 endpoints documented
- [ ] Request validation tested
- [ ] Response format consistent (JSON with status/data structure)
- [ ] Error responses follow standard format
- [ ] Pagination implemented and tested
- [ ] Rate limiting per user/IP
- [ ] API versioning strategy defined

### ✅ Frontend Validation
- [ ] All pages load without errors
- [ ] Responsive design tested on mobile/tablet/desktop
- [ ] Navigation works correctly
- [ ] Authentication flows (login, register, logout)
- [ ] Data binding to API working
- [ ] Error messages display correctly
- [ ] Loading states visible during async operations
- [ ] No console errors or warnings

### ✅ Infrastructure Validation
- [ ] Docker image builds successfully
- [ ] Docker Compose starts all services without errors
- [ ] Services communicate correctly
- [ ] Environment variables properly configured
- [ ] Health checks responding
- [ ] Logs are being collected
- [ ] Monitoring/alerting configured

---

## Production Deployment Steps

### Phase 1: Pre-Deployment (2-4 hours)

#### 1.1 Environment Setup
```bash
# Create production environment
export ENVIRONMENT=production
export API_PORT=3000
export DATABASE_URL=postgres://prod_user:strong_password@RDS_ENDPOINT:5432/vpn_service
export JWT_SECRET=$(openssl rand -base64 32)
export DB_USER=vpn_prod_user
export DB_PASSWORD=$(openssl rand -base64 32)
```

#### 1.2 Database Preparation
```bash
# Create production database
createdb -h <RDS_ENDPOINT> -U postgres vpn_service_prod

# Create application user
psql -h <RDS_ENDPOINT> -U postgres -c "CREATE USER vpn_prod_user WITH PASSWORD 'PASSWORD';"

# Grant privileges
psql -h <RDS_ENDPOINT> -U postgres vpn_service_prod -c "GRANT ALL ON SCHEMA public TO vpn_prod_user;"

# Run migrations
sqlx migrate run --database-url $DATABASE_URL
```

#### 1.3 SSL/TLS Certificate
```bash
# Generate or import certificate
# Option 1: Let's Encrypt
certbot certonly --dns-route53 -d api.example.com

# Option 2: AWS Certificate Manager
aws acm request-certificate --domain-name api.example.com
```

#### 1.4 Secrets Management
```bash
# Store secrets in secure location
# AWS Options:
# - AWS Secrets Manager
# - AWS Parameter Store
# - Vault

# Example: Store in Secrets Manager
aws secretsmanager create-secret \
  --name vpn-service/production \
  --secret-string '{
    "jwt_secret": "...",
    "db_password": "...",
    "api_key": "..."
  }'
```

### Phase 2: Build & Registry (1-2 hours)

#### 2.1 Build Docker Image
```bash
# Build for production
docker build -t vpn-service:1.0.0 \
  --build-arg API_PORT=3000 \
  --build-arg ENVIRONMENT=production \
  .

# Build Angular frontend separately
docker build -t vpn-dashboard:1.0.0 -f Dockerfile.frontend .
```

#### 2.2 Push to Registry
```bash
# Tag images
docker tag vpn-service:1.0.0 registry.example.com/vpn-service:1.0.0
docker tag vpn-dashboard:1.0.0 registry.example.com/vpn-dashboard:1.0.0

# Push to registry
docker push registry.example.com/vpn-service:1.0.0
docker push registry.example.com/vpn-dashboard:1.0.0
```

#### 2.3 Verify Images
```bash
# Test image
docker run --rm registry.example.com/vpn-service:1.0.0 /app/vpn-api --version

# Scan for vulnerabilities
trivy image registry.example.com/vpn-service:1.0.0
```

### Phase 3: Infrastructure Deployment (2-3 hours)

#### 3.1 Kubernetes Deployment
```bash
# Apply configuration
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/secrets.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/postgres.yaml
kubectl apply -f k8s/redis.yaml
kubectl apply -f k8s/api.yaml
kubectl apply -f k8s/frontend.yaml

# Verify deployment
kubectl get pods -n vpn-service
kubectl get svc -n vpn-service
```

#### 3.2 Load Balancer Setup
```bash
# AWS ALB
aws elbv2 create-load-balancer \
  --name vpn-service-lb \
  --subnets subnet-1 subnet-2 \
  --security-groups sg-1

# Configure target groups
aws elbv2 create-target-group \
  --name vpn-api \
  --protocol HTTP \
  --port 3000 \
  --vpc-id vpc-123
```

#### 3.3 DNS Configuration
```bash
# Update DNS to point to load balancer
aws route53 change-resource-record-sets \
  --hosted-zone-id Z123 \
  --change-batch '{
    "Changes": [{
      "Action": "UPSERT",
      "ResourceRecordSet": {
        "Name": "api.example.com",
        "Type": "CNAME",
        "TTL": 300,
        "ResourceRecords": [{"Value": "vpn-service-lb.elb.amazonaws.com"}]
      }
    }]
  }'
```

### Phase 4: Application Setup (1-2 hours)

#### 4.1 Database Initialization
```bash
# Verify database is accessible
psql -h api.example.com -U vpn_prod_user vpn_service -c "SELECT version();"

# Check tables
psql -h api.example.com -U vpn_prod_user vpn_service -c "\dt"

# Create indices
psql -h api.example.com -U vpn_prod_user vpn_service < indices.sql
```

#### 4.2 Application Health Check
```bash
# Test API endpoint
curl https://api.example.com/health

# Expected: {"status":"ok"}

# Test readiness
curl https://api.example.com/health/ready

# Expected: {"status":"ready"}
```

#### 4.3 Create Initial Users
```bash
# Create admin user
curl -X POST https://api.example.com/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "strong_password_here"
  }'

# Verify admin can login
curl -X POST https://api.example.com/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "strong_password_here"
  }'
```

### Phase 5: Monitoring & Logging (1-2 hours)

#### 5.1 Application Metrics
```yaml
# Deploy Prometheus
kubectl apply -f monitoring/prometheus.yaml

# Deploy Grafana
kubectl apply -f monitoring/grafana.yaml

# Configure dashboards
# - Request latency
# - Error rate
# - Database query time
# - Active connections
```

#### 5.2 Logging Setup
```bash
# ELK Stack or CloudWatch
# Configure log aggregation
# - Application logs
# - Access logs
# - Error logs
# - Audit logs

# Example: CloudWatch
aws logs create-log-group --log-group-name /vpn-service/api
aws logs create-log-stream --log-group-name /vpn-service/api --log-stream-name production
```

#### 5.3 Alerting
```yaml
# Configure alerts
alerts:
  - name: HighErrorRate
    condition: error_rate > 5%
    action: notify_slack

  - name: DatabaseDown
    condition: db_connection_failed
    action: page_oncall

  - name: HighLatency
    condition: p99_latency > 1000ms
    action: notify_slack
```

### Phase 6: Testing (1-2 hours)

#### 6.1 Smoke Tests
```bash
# Run critical path tests
./test-e2e.sh

# Verify:
# - User registration
# - User login
# - Peer creation
# - Config download
# - Peer deletion
```

#### 6.2 Load Testing
```bash
# Run load test
ab -n 10000 -c 100 https://api.example.com/health

# Check performance
# - Response time
# - Throughput
# - Error rate
```

#### 6.3 Security Testing
```bash
# OWASP dependency check
dependency-check --project "VPN Service" --scan .

# SQL injection test
# - Verify sqlx prevents injection

# XSS test
# - Verify Angular sanitization

# CORS test
curl -H "Origin: http://attacker.com" https://api.example.com/peers
```

### Phase 7: Go-Live (30 minutes)

#### 7.1 Final Verification
```bash
# [ ] All services healthy
# [ ] Database responding
# [ ] API endpoints working
# [ ] Frontend loads
# [ ] Logs being recorded
# [ ] Monitoring active
# [ ] Backups running
```

#### 7.2 Traffic Switch
```bash
# Update DNS TTL to low value
aws route53 change-resource-record-sets --update...

# Monitor traffic
# - Gradually shift traffic if using canary deployment
# - Monitor error rates
# - Check application logs

# Full traffic switch
# - Update load balancer rules
# - Monitor metrics
```

#### 7.3 Notification
```bash
# Notify stakeholders
# - Deployment complete
# - System is live
# - Monitoring is active
# - Support contacts
```

---

## Post-Deployment Validation

### ✅ System Health
- [ ] All services running
- [ ] Database responding
- [ ] API responding to requests
- [ ] Frontend loading correctly
- [ ] Monitoring active
- [ ] Logs being recorded
- [ ] Backups running

### ✅ Functional Testing
- [ ] User registration works
- [ ] User login works
- [ ] Can create peers
- [ ] Can download configs
- [ ] Can delete peers
- [ ] Dashboard shows data
- [ ] Nodes list displays
- [ ] Audit logs recorded

### ✅ Performance Baseline
- [ ] API response time < 500ms
- [ ] Database query time < 100ms
- [ ] Frontend load time < 3s
- [ ] Throughput: > 1000 req/s
- [ ] Error rate: < 0.1%

### ✅ Security Verify
- [ ] HTTPS working
- [ ] Certificates valid
- [ ] JWT tokens valid
- [ ] Password hashing working
- [ ] Audit logs secure
- [ ] API rate limiting active
- [ ] CORS properly configured

---

## Rollback Procedure

If deployment fails:

### Immediate Actions
```bash
# 1. Stop new deployment
kubectl rollout undo deployment/vpn-api -n vpn-service

# 2. Verify rollback
kubectl get pods -n vpn-service

# 3. Check services are responding
curl https://api.example.com/health

# 4. Notify team
# - Rollback initiated
# - Time when service will be available
# - Root cause analysis underway
```

### Investigation
```bash
# Check logs for errors
kubectl logs -f deployment/vpn-api -n vpn-service

# Check events
kubectl describe pod -n vpn-service

# Check metrics
# - Error rate
# - Response time
# - Database connectivity
```

### Fix and Retry
```bash
# 1. Fix issue identified
# 2. Test in staging
# 3. Rebuild container
# 4. Push to registry
# 5. Re-deploy with new image
```

---

## Success Criteria

✅ **Deployment is successful when:**

1. All services are running and healthy
2. API responds to requests with < 500ms latency
3. Frontend loads and displays correctly
4. User can complete full workflow (register → login → create peer → download config)
5. No errors in application logs
6. Monitoring dashboard shows healthy metrics
7. At least 99.5% uptime in first 24 hours
8. Zero data loss or corruption

❌ **Deploy should be rolled back if:**

1. Any service fails to start
2. API response time > 2s
3. Error rate > 1%
4. Database connectivity issues
5. Frontend not loading
6. Unable to authenticate users
7. Audit logs not being recorded
8. Data corruption detected

---

## Maintenance Schedule

### Daily
- [ ] Check error rates
- [ ] Verify backups completed
- [ ] Monitor resource usage

### Weekly
- [ ] Review audit logs
- [ ] Update dependencies
- [ ] Performance report

### Monthly
- [ ] Full backup recovery test
- [ ] Security audit
- [ ] Capacity planning
- [ ] Release planning

---

## Contacts

- **On-Call Engineer**: [Name/Phone]
- **Database Admin**: [Name/Phone]
- **DevOps Lead**: [Name/Phone]
- **Product Manager**: [Name/Phone]

---

**Last Updated**: 2024  
**Version**: 1.0-MVP
