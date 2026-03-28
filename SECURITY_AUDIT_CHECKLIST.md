# Security Audit Checklist - VPN Service

## Pre-Launch Security Assessment

All items must be completed and verified before production deployment.

---

## 1. AUTHENTICATION & AUTHORIZATION ✅

### 1.1 JWT Token Security
- [ ] JWT tokens use HS256 or RS256 algorithm
- [ ] Token expiration set (recommended: 24 hours for access, 7 days for refresh)
- [ ] Refresh token rotation implemented
- [ ] Token revocation mechanism in place
- [ ] Token stored in HTTP-only cookies or secure storage
- [ ] JWT algorithm not set to 'none'
- [ ] Invalid signatures rejected
- [ ] Token tampering prevention tested

### 1.2 Two-Factor Authentication (2FA)
- [ ] TOTP (Time-based OTP) implementation verified
- [ ] Backup codes generated and stored securely
- [ ] 2FA enforcement for admin accounts
- [ ] 2FA optional for regular users
- [ ] Recovery procedures documented
- [ ] Session timeout after 3 consecutive failed attempts

### 1.3 Password Policy
- [ ] Minimum 12 characters required
- [ ] Complexity requirements enforced (uppercase, lowercase, numbers, symbols)
- [ ] Password not in common password dictionary
- [ ] Previous 5 passwords cannot be reused
- [ ] Passwords hashed with Argon2 or bcrypt
- [ ] Password reset token expires in 30 minutes
- [ ] Password reset requires email verification

### 1.4 Access Control
- [ ] Role-based access control (RBAC) implemented
- [ ] Admin role restricted to necessary users only
- [ ] Service accounts have minimal required permissions
- [ ] API key management implemented
- [ ] API keys rotated monthly
- [ ] Least privilege principle applied

---

## 2. DATA ENCRYPTION 🔐

### 2.1 Transport Layer
- [ ] TLS 1.2+ enforced (TLS 1.3 preferred)
- [ ] Certificate obtained from trusted CA
- [ ] Certificate validity verified (not self-signed in production)
- [ ] HSTS header set (min 31536000 seconds)
- [ ] Certificate renewal automated
- [ ] SSL/TLS certificate pinning considered for mobile apps
- [ ] Perfect forward secrecy (PFS) enabled

### 2.2 Data at Rest
- [ ] Sensitive data encrypted in database
- [ ] Encryption keys stored separately from encrypted data
- [ ] Key rotation procedure documented
- [ ] Encryption algorithm: AES-256 or better
- [ ] Backup data encrypted with same standards
- [ ] Personally identifiable information (PII) encrypted

### 2.3 Password & Secrets Management
- [ ] Database containing hashed passwords only
- [ ] API keys, secrets NOT in source code
- [ ] Environment variables used for secrets
- [ ] Secrets manager (e.g., Vault, AWS Secrets Manager) considered
- [ ] No secrets in logs or error messages
- [ ] Database credentials rotated on schedule

---

## 3. INPUT VALIDATION & OUTPUT ENCODING 📝

### 3.1 Input Validation
- [ ] All inputs validated on server-side (not just client-side)
- [ ] SQL injection prevention: parameterized queries used
- [ ] NoSQL injection prevention if applicable
- [ ] Command injection prevention
- [ ] Path traversal prevention
- [ ] File upload validation: type and size checks
- [ ] Email validation with RFC compliance
- [ ] Rate limiting on input endpoints

### 3.2 Output Encoding
- [ ] HTML output properly escaped
- [ ] JSON responses properly formatted
- [ ] XML output validated before sending
- [ ] CSV output sanitized
- [ ] XSS protection headers set (X-XSS-Protection)
- [ ] Content-Type headers set correctly

### 3.3 Data Validation
- [ ] Schema validation for all JSON requests
- [ ] File upload virus scanning implemented
- [ ] File upload destination outside web root
- [ ] File permissions restricted appropriately
- [ ] Executable files not allowed to upload

---

## 4. API SECURITY 🔑

### 4.1 API Authentication
- [ ] All API endpoints require authentication
- [ ] API key authentication implemented
- [ ] Bearer token authentication verified
- [ ] OAuth2 flow properly implemented if used
- [ ] API key rotation in place
- [ ] Deprecated APIs removed from production

### 4.2 API Rate Limiting
- [ ] Rate limiting per user implemented (100 req/min)
- [ ] Rate limiting per IP for public endpoints (10 req/min)
- [ ] Exponential backoff for retry attempts
- [ ] Rate limit headers returned in responses
- [ ] DDoS protection service (e.g., CloudFlare) considered
- [ ] Bot detection implemented

### 4.3 API Versioning
- [ ] API versioning strategy documented
- [ ] Old API versions supported for transition period
- [ ] Deprecation timelines communicated
- [ ] Version validation implemented

---

## 5. DATABASE SECURITY 🗄️

### 5.1 Database Access
- [ ] Database runs on private network (not public internet)
- [ ] Database connection encrypted
- [ ] Database user created with minimal permissions
- [ ] Default credentials changed
- [ ] Connection pooling used
- [ ] Idle connections cleaned up
- [ ] Admin access logged and audited

### 5.2 Database Backup
- [ ] Automated daily backups implemented
- [ ] Backups encrypted
- [ ] Backups tested for restore
- [ ] Backup retention policy: 30 days minimum
- [ ] Off-site backup storage (AWS S3, Azure Blob, etc.)
- [ ] Backup access restricted

### 5.3 SQL Injection Prevention
- [ ] Parameterized queries used everywhere
- [ ] No string concatenation in SQL
- [ ] Prepared statements utilized
- [ ] ORM (sqlx, diesel, etc.) configured securely

---

## 6. SESSION MANAGEMENT 🔐

### 6.1 Session Handling
- [ ] Session timeout: 30 minutes of inactivity
- [ ] Absolute timeout: 24 hours maximum
- [ ] Session invalidation on logout
- [ ] Simultaneous session limits enforced
- [ ] Session data encrypted
- [ ] Session cookie: HttpOnly, Secure, SameSite flags set
- [ ] Session fixation prevention

### 6.2 Single Sign-On (SSO)
- [ ] If using SSO: SAML or OpenID Connect properly configured
- [ ] SSO metadata validated
- [ ] Logout propagated across all systems
- [ ] Session state properly maintained

---

## 7. AUDIT LOGGING & MONITORING 📊

### 7.1 Logging
- [ ] All authentication attempts logged
- [ ] All admin actions logged
- [ ] Failed access attempts logged
- [ ] Data modifications tracked
- [ ] Logs include: timestamp, user, action, result, IP
- [ ] Logs centralized and immutable
- [ ] Sensitive data NOT logged (passwords, tokens, PII)
- [ ] Log retention: minimum 90 days

### 7.2 Monitoring & Alerting
- [ ] Failed login attempts monitored
- [ ] Brute force attacks detected and blocked
- [ ] Unusual access patterns detected
- [ ] Database errors monitored
- [ ] Performance metrics tracked
- [ ] Alerts configured for security events
- [ ] Incident response team assigned

### 7.3 Security Events
- [ ] Alert: 5+ failed logins in 5 minutes per user
- [ ] Alert: 10+ failed logins in 5 minutes per IP
- [ ] Alert: Unusual geographic login
- [ ] Alert: Privilege escalation attempt
- [ ] Alert: Database access outside normal hours
- [ ] Alert: Large data export detected

---

## 8. VULNERABILITY MANAGEMENT 🛡️

### 8.1 Dependency Management
- [ ] Dependency scanner (Snyk, OWASP Dependency Check) running
- [ ] Outdated dependencies identified and updated
- [ ] Known vulnerabilities remediated
- [ ] Security advisories monitored (GitHub, NVD)
- [ ] Development dependencies not in production

### 8.2 Code Security
- [ ] Static code analysis (SAST) tool configured
- [ ] Code review process mandatory
- [ ] Security linting enabled
- [ ] Secrets scanner prevents credential commits
- [ ] No hardcoded secrets in codebase

### 8.3 Penetration Testing
- [ ] Annual penetration test performed
- [ ] Findings documented and remediated
- [ ] Test scope covers authentication, authorization, data
- [ ] Third-party tester used (external review)

---

## 9. INFRASTRUCTURE SECURITY 🏗️

### 9.1 Network Security
- [ ] Firewall implemented with strict rules
- [ ] Only necessary ports open (80, 443, 5432 internal only)
- [ ] VPC/subnet isolation used
- [ ] DDoS protection enabled
- [ ] WAF (Web Application Firewall) configured
- [ ] Load balancer uses HTTPS
- [ ] CDN caching configured

### 9.2 Container Security
- [ ] Container images scanned for vulnerabilities
- [ ] Base images regularly updated
- [ ] Containers run as non-root user
- [ ] Container filesystem read-only where possible
- [ ] Resource limits set (CPU, memory)
- [ ] Container registry access controlled
- [ ] Image signing implemented

### 9.3 Server Hardening
- [ ] Server OS patched and updated
- [ ] Unnecessary services disabled
- [ ] SSH key-based authentication (no passwords)
- [ ] SSH on non-standard port (if applicable)
- [ ] Firewall rules minimal and explicit
- [ ] Security updates applied timely
- [ ] Physical security considered (if applicable)

---

## 10. COMPLIANCE & LEGAL 📋

### 10.1 Data Privacy
- [ ] GDPR compliance verified (if EU users)
- [ ] CCPA compliance verified (if California users)
- [ ] Privacy policy drafted and deployed
- [ ] Right to be forgotten implemented
- [ ] Data export functionality implemented
- [ ] Consent management system in place
- [ ] Third-party data sharing documented

### 10.2 Incident Response
- [ ] Incident response plan documented
- [ ] Contact information for key personnel maintained
- [ ] Breach notification procedure in place
- [ ] 72-hour breach reporting timeline understood
- [ ] Incident response team trained
- [ ] Incident response drills conducted

### 10.3 Documentation
- [ ] Security architecture documented
- [ ] Security procedures documented
- [ ] Data flow diagrams created
- [ ] Threat model completed
- [ ] Risk assessment performed
- [ ] Security policies documented

---

## 11. THIRD-PARTY INTEGRATIONS 🔗

### 11.1 Payment Processing (Stripe)
- [ ] PCI DSS Level 1 compliance maintained
- [ ] Credit card data never touches your servers
- [ ] Webhook signatures verified
- [ ] Webhook endpoints use HTTPS
- [ ] Stripe API key stored securely
- [ ] Test keys used in development only

### 11.2 Email Service
- [ ] Email provider uses TLS/STARTTLS
- [ ] Email verification implemented
- [ ] Password reset emails include time-limited links
- [ ] Unsubscribe functionality present

### 11.3 External APIs
- [ ] All external API calls over HTTPS
- [ ] API keys stored in environment variables
- [ ] Rate limiting on outbound calls
- [ ] Timeouts set on external API calls
- [ ] Error handling for failed API calls

---

## 12. SECURITY TESTING ✅

### 12.1 Testing Before Launch
- [ ] All unit tests passing
- [ ] Integration tests passing
- [ ] Security-focused test cases included
- [ ] OWASP Top 10 vulnerabilities tested
- [ ] SQL injection tested
- [ ] XSS vulnerabilities tested
- [ ] CSRF protection verified
- [ ] Load testing completed (spike and stress tests)

### 12.2 Automated Testing
- [ ] SAST (Static Application Security Testing) enabled
- [ ] DAST (Dynamic Application Security Testing) configured
- [ ] Dependency scanning automated
- [ ] Container image scanning automated
- [ ] Secret scanning automated

---

## Verification & Sign-Off

- [ ] Security team reviewed and approved
- [ ] CISO (or equivalent) sign-off obtained
- [ ] Compliance team confirms regulatory requirements met
- [ ] Risk assessment approved
- [ ] Insurance/liability confirmed

**Sign-Off Date:** _______________

**Security Lead:** _______________

**CTO/CISO:** _______________

---

## Post-Launch Monitoring

- [ ] Security tools monitoring 24/7
- [ ] Monthly security reviews scheduled
- [ ] Quarterly penetration testing scheduled
- [ ] Annual security audit scheduled
- [ ] Incident response procedures tested quarterly
