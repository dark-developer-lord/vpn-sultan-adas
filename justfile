set dotenv-load

# Build all crates
build:
    cargo build --release

# Run tests
test:
    cargo test --all

# Format code
fmt:
    cargo fmt --all

# Lint code
lint:
    cargo clippy --all -- -D warnings

# Start local development environment
dev-up:
    docker-compose up -d

# Stop development environment
dev-down:
    docker-compose down

# View database logs
db-logs:
    docker-compose logs -f postgres

# View API logs
api-logs:
    docker-compose logs -f api

# Connect to database
db-shell:
    docker-compose exec postgres psql -U vpn -d vpn_service

# Create fresh database
db-reset:
    docker-compose down
    docker-compose up postgres -d
    sleep 5
    docker-compose exec postgres psql -U vpn -d vpn_service -f /app/migrations/20260327_001_init_schema.sql

# Check API health
check-api:
    curl http://localhost:3000/health

# Run API locally (without Docker)
api-local:
    cargo run --bin vpn-api

# Run agent locally
agent-local:
    cargo run --bin vpn-agent

# Clean build artifacts
clean:
    cargo clean

# Full development setup
setup-dev: dev-up
    echo "✅ Development environment is up!"
    echo "📊 Database: postgresql://vpn:vpn@localhost:5432/vpn_service"
    echo "🌐 API: http://localhost:3000"
    echo "🔧 PgAdmin: http://localhost:5050"
