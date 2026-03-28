#!/bin/bash

# VPN Service Quick Setup Script
# Usage: ./scripts/quick-setup.sh [environment]
# Environments: local, staging, production

set -e

ENVIRONMENT="${1:-local}"
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🚀 VPN Service Quick Setup - $ENVIRONMENT"
echo "========================================"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_error "$1 is not installed"
        return 1
    fi
    return 0
}

# Step 1: Validate Prerequisites
log_info "Checking prerequisites..."

required_commands=("docker" "docker-compose" "cargo" "node" "npm" "postgresql" "redis-cli")

for cmd in "${required_commands[@]}"; do
    if check_command "$cmd"; then
        log_info "✓ $cmd found"
    else
        log_warn "✗ $cmd not found - some features may not work"
    fi
done

# Step 2: Configure Environment
log_info "Setting up environment configuration..."

ENV_FILE="$PROJECT_ROOT/.env.$ENVIRONMENT"

if [ ! -f "$ENV_FILE" ]; then
    log_info "Creating $ENV_FILE from template..."
    
    cat > "$ENV_FILE" << EOF
# VPN Service Environment Configuration

# Environment
ENVIRONMENT=$ENVIRONMENT
ENV=production

# Database
DATABASE_URL=postgresql://vpn_user:vpn_password@localhost:5432/vpn_db
POSTGRES_USER=vpn_user
POSTGRES_PASSWORD=vpn_password
POSTGRES_DB=vpn_db

# Redis
REDIS_URL=redis://localhost:6379
REDIS_HOST=localhost
REDIS_PORT=6379

# JWT
JWT_SECRET=$(openssl rand -base64 32)
JWT_EXPIRY=3600

# API
API_PORT=8080
API_HOST=0.0.0.0
API_LOG_LEVEL=info

# Frontend
FRONTEND_PORT=4200
API_URL=http://localhost:8080

# Stripe (leave empty for testing)
STRIPE_PUBLIC_KEY=pk_test_xxx
STRIPE_SECRET_KEY=sk_test_xxx

# OAuth (if using GitHub)
GITHUB_CLIENT_ID=
GITHUB_CLIENT_SECRET=

# Monitoring
PROMETHEUS_PORT=9090
GRAFANA_PORT=3000
GRAFANA_ADMIN_USER=admin
GRAFANA_ADMIN_PASSWORD=$(openssl rand -base64 16)

# Slack Integration (optional)
SLACK_WEBHOOK=https://hooks.slack.com/services/YOUR/WEBHOOK/URL

EOF
    log_info "Created $ENV_FILE - please review and update secrets"
else
    log_info "Using existing $ENV_FILE"
fi

# Load environment
set -a
source "$ENV_FILE"
set +a

# Step 3: Start Infrastructure (Docker Compose)
log_info "Starting infrastructure services..."

if [ "$ENVIRONMENT" == "local" ]; then
    docker-compose up -d postgres redis nginx
    
    log_info "Waiting for PostgreSQL..."
    for i in {1..30}; do
        if pg_isready -h localhost -U "$POSTGRES_USER" &> /dev/null; then
            log_info "PostgreSQL is ready"
            break
        fi
        if [ $i -eq 30 ]; then
            log_error "PostgreSQL startup timeout"
            exit 1
        fi
        sleep 1
    done
    
    log_info "Waiting for Redis..."
    for i in {1..30}; do
        if redis-cli ping &> /dev/null; then
            log_info "Redis is ready"
            break
        fi
        if [ $i -eq 30 ]; then
            log_error "Redis startup timeout"
            exit 1
        fi
        sleep 1
    done
fi

# Step 4: Setup Database
log_info "Setting up database..."

# Check if tables exist
if psql "$DATABASE_URL" -c "SELECT 1 FROM pg_tables WHERE tablename='users' LIMIT 1;" 2>/dev/null | grep -q 1; then
    log_info "Database already initialized"
else
    log_info "Running database migrations..."
    cd "$PROJECT_ROOT"
    cargo sqlx database create
    cargo sqlx migrate run
    log_info "Database migration complete"
fi

# Step 5: Build Backend
log_info "Building backend..."

cd "$PROJECT_ROOT"
log_info "Running backend tests..."
cargo test --release

log_info "Building backend binary..."
cargo build --release

BINARY_SIZE=$(du -h target/release/vpn-api | cut -f1)
log_info "Backend built successfully (size: $BINARY_SIZE)"

# Step 6: Build Frontend
log_info "Building frontend..."

cd "$PROJECT_ROOT/crates/frontend"
npm install
npm run build

FRONTEND_SIZE=$(du -sh dist/ | cut -f1)
log_info "Frontend built successfully (size: $FRONTEND_SIZE)"

# Step 7: Start Application
log_info "Starting application services..."

if [ "$ENVIRONMENT" == "local" ]; then
    # Backend
    log_info "Starting backend..."
    cd "$PROJECT_ROOT"
    ./target/release/vpn-api > /tmp/vpn-api.log 2>&1 &
    API_PID=$!
    echo $API_PID > /tmp/vpn-api.pid
    
    # Frontend dev server
    log_info "Starting frontend..."
    cd "$PROJECT_ROOT/crates/frontend"
    npm start > /tmp/vpn-frontend.log 2>&1 &
    FRONTEND_PID=$!
    echo $FRONTEND_PID > /tmp/vpn-frontend.pid
    
    sleep 3
fi

# Step 8: Start Monitoring (Optional)
log_info "Starting monitoring stack..."

if [ "$ENVIRONMENT" == "local" ]; then
    cd "$PROJECT_ROOT"
    docker-compose -f docker-compose.monitoring.yml up -d prometheus grafana alertmanager loki
    
    log_info "Waiting for monitoring services..."
    sleep 5
fi

# Step 9: Health Checks
log_info "Running health checks..."

# API Health
log_info "Checking API health..."
for i in {1..20}; do
    if curl -s http://localhost:8080/health | grep -q '"status":"ok"'; then
        log_info "✓ API is healthy"
        break
    fi
    if [ $i -eq 20 ]; then
        log_error "API health check failed"
        exit 1
    fi
    sleep 1
done

# Database Health
log_info "Checking database health..."
if psql "$DATABASE_URL" -c "SELECT 1;" &> /dev/null; then
    log_info "✓ Database is healthy"
else
    log_error "Database health check failed"
    exit 1
fi

# Redis Health
log_info "Checking Redis health..."
if redis-cli -h localhost ping | grep -q PONG; then
    log_info "✓ Redis is healthy"
else
    log_error "Redis health check failed"
    exit 1
fi

# Step 10: Generate Summary
echo ""
log_info "✅ Setup Complete!"
echo ""
echo "========================================"
echo "🎉 VPN Service is Ready"
echo "========================================"
echo ""
echo "Service URLs:"
echo "  API:           http://localhost:8080"
echo "  Frontend:      http://localhost:4200"
echo "  Prometheus:    http://localhost:9090"
echo "  Grafana:       http://localhost:3000 (admin/${GRAFANA_ADMIN_PASSWORD})"
echo "  Nginx:         http://localhost"
echo ""
echo "Database:"
echo "  Host:          localhost"
echo "  Port:          5432"
echo "  User:          $POSTGRES_USER"
echo "  Database:      $POSTGRES_DB"
echo ""
echo "Redis:"
echo "  Host:          localhost"
echo "  Port:          6379"
echo ""
echo "Useful Commands:"
echo "  View API logs:       tail -f /tmp/vpn-api.log"
echo "  View Frontend logs:  tail -f /tmp/vpn-frontend.log"
echo "  Stop API:            kill \$(cat /tmp/vpn-api.pid)"
echo "  Stop Frontend:       kill \$(cat /tmp/vpn-frontend.pid)"
echo "  DB Connection:       psql \"$DATABASE_URL\""
echo "  Redis CLI:           redis-cli"
echo ""
echo "Testing:"
echo "  $ curl http://localhost:8080/health"
echo "  $ k6 run tests/smoke-test.js"
echo "  $ npm test (in frontend directory)"
echo ""
echo "Documentation:"
echo "  Deployment:  DEPLOYMENT_GUIDE.md"
echo "  API Docs:    API_DOCUMENTATION.md"
echo "  Architecture: PROJECT_COMPLETION_SUMMARY.md"
echo ""

# Create script to stop all services
cat > "$PROJECT_ROOT/scripts/stop-services.sh" << 'STOP_SCRIPT'
#!/bin/bash
echo "Stopping VPN services..."
kill $(cat /tmp/vpn-api.pid) 2>/dev/null || true
kill $(cat /tmp/vpn-frontend.pid) 2>/dev/null || true
docker-compose down
docker-compose -f docker-compose.monitoring.yml down
echo "Services stopped"
STOP_SCRIPT

chmod +x "$PROJECT_ROOT/scripts/stop-services.sh"

echo "To stop all services: ./scripts/stop-services.sh"
echo ""
