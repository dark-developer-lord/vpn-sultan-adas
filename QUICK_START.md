# Quick Start Guide - VPN Service

## Prerequisites

- Rust 1.70+ ([install](https://rustup.rs/))
- Node.js 18+ ([install](https://nodejs.org/))
- Docker & Docker Compose ([install](https://docs.docker.com/get-docker/))
- PostgreSQL 15+ (or via Docker)

---

## 1️⃣ Start Services with Docker Compose

```bash
cd /Users/sultonshonazarshoev/Documents/vpn-service

# Start all services (PostgreSQL, Redis, Backend, Frontend)
docker-compose up -d

# Verify services are running
docker-compose ps

# Check logs
docker-compose logs -f api
docker-compose logs -f frontend
```

Expected output:
- PostgreSQL: Listening on `localhost:5432`
- Redis: Listening on `localhost:6379`
- Backend API: Listening on `http://localhost:3000`
- Frontend: Listening on `http://localhost:4200`

---

## 2️⃣ Setup Database

```bash
# The database schema is automatically created via migrations in docker-compose.yml

# Verify database connection
docker-compose exec postgres psql -U vpn_user -d vpn_service -c "SELECT * FROM users;"

# Expected: Empty table, no errors
```

---

## 3️⃣ Run Backend (Manual)

If not using Docker Compose:

```bash
cd /Users/sultonshonazarshoev/Documents/vpn-service

# Set environment variables
export DATABASE_URL=postgres://vpn_user:vpn_password@localhost:5432/vpn_service
export JWT_SECRET=your-secret-key-here
export API_PORT=3000

# Run database migrations (if needed)
sqlx migrate run --database-url $DATABASE_URL

# Start the API
cargo run --bin vpn-api

# Expected output:
# [INFO] Server listening on 0.0.0.0:3000
```

---

## 4️⃣ Run Frontend (Manual)

If not using Docker Compose:

```bash
cd /Users/sultonshonazarshoev/Documents/vpn-service/frontend

# Install dependencies
npm install

# Start development server
npm start

# Expected output:
# Application bundle generation complete. [2 files]
# Initial Chunk Files   | Names         |      Size
# ✔ Compiled successfully.
# ⠙ Building...

# Open http://localhost:4200 in your browser
```

---

## 5️⃣ Test the API

### Health Check
```bash
curl http://localhost:3000/health
# Response: {"status":"ok"}
```

### Register New User
```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "SecurePassword123"
  }'

# Response: {"status":"success","data":{"token":"eyJ0eXAiOiJKV1QiLCJhbGc..."}}
```

### Login
```bash
TOKEN=$(curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "SecurePassword123"
  }' | jq -r '.data.token')

echo $TOKEN
```

### Create Peer (VPN Client)
```bash
# First, get a node ID
curl -X GET http://localhost:3000/nodes \
  -H "Authorization: Bearer $TOKEN" | jq '.data[0].id'

NODE_ID="<node-id-from-above>"

# Create peer
curl -X POST http://localhost:3000/peers \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"My Device\",
    \"node_id\": \"$NODE_ID\"
  }"

# Response includes peer configuration
```

### Get WireGuard Config
```bash
curl -X GET http://localhost:3000/peers/<peer-id>/config \
  -H "Authorization: Bearer $TOKEN"

# Response: WireGuard configuration in wg-quick format
```

---

## 6️⃣ Run Tests

### Backend Integration Tests
```bash
cd /Users/sultonshonazarshoev/Documents/vpn-service

# Run all tests
cargo test --test integration_tests

# Expected: running 22 tests
#          test result: ok. 22 passed; 0 failed
```

### Backend Unit Tests
```bash
cargo test --lib

# Tests validation, parsing, and format checks
```

### Frontend Tests (if needed)
```bash
cd frontend

# Run Angular tests
npm test

# Run e2e tests
npm run e2e
```

---

## 7️⃣ Access the Frontend

Open your browser and navigate to:

```
http://localhost:4200
```

### Available Pages

1. **Login** (`/login`)
   - Email: `test@example.com`
   - Password: `SecurePassword123`

2. **Register** (`/register`)
   - Create new account with email/password

3. **Dashboard** (`/dashboard`)
   - View stats: active peers, online nodes, subscription info

4. **Peers** (`/peers`)
   - Create new VPN peer
   - List your peers
   - Download WireGuard configuration
   - Delete peers

5. **Nodes** (`/nodes`)
   - View available VPN nodes
   - Check node status
   - Copy public key

---

## 8️⃣ Stop Services

```bash
# Stop all Docker services
docker-compose down

# Remove volumes (wipes database)
docker-compose down -v

# For manual cleanup
pkill -f "cargo run"  # Kill Rust backend
pkill -f "ng serve"    # Kill frontend dev server
```

---

## 🔍 Troubleshooting

### Backend won't start
```bash
# Check if port 3000 is in use
lsof -i :3000

# Kill existing process
kill -9 <PID>

# Check database connection
export DATABASE_URL=postgres://vpn_user:vpn_password@localhost:5432/vpn_service
sqlx database create
```

### Frontend won't start
```bash
# Check Node.js version
node --version  # Should be 18+

# Clear npm cache
npm cache clean --force

# Reinstall dependencies
rm -rf node_modules package-lock.json
npm install

# Start again
npm start
```

### Can't connect to database
```bash
# Check Docker status
docker-compose ps

# Check PostgreSQL logs
docker-compose logs postgres

# Verify credentials in .env
cat .env | grep DATABASE_URL

# Test connection directly
psql postgres://vpn_user:vpn_password@localhost:5432/vpn_service
```

### JWT token issues
```bash
# Check token format (should be 3 parts separated by dots)
curl http://localhost:3000/peers \
  -H "Authorization: Bearer $TOKEN" \
  -v

# If 401 Unauthorized:
# 1. Verify token is valid
# 2. Check token expiry (15 minutes)
# 3. Try logging in again
```

---

## 📊 Project Structure

```
vpn-service/
├── crates/                 # Rust backend
│   ├── api/               # HTTP server (run this)
│   ├── domain/            # Business logic
│   ├── data/              # Database layer
│   ├── crypto/            # Encryption utilities
│   └── shared/            # Shared types
├── frontend/              # Angular application
│   ├── src/
│   │   ├── app/
│   │   │   ├── core/      # Services, interceptors
│   │   │   ├── features/  # Pages (auth, peers, nodes, dashboard)
│   │   │   └── shared/    # Shared components
│   │   └── main.ts        # Entry point
│   └── package.json
├── migrations/            # Database schema
├── docker-compose.yml     # Development setup
├── Dockerfile             # Production container
└── .env                   # Configuration
```

---

## 🔐 Default Credentials

**For Development Only:**

```env
DB_USER=vpn_user
DB_PASSWORD=vpn_password
DB_NAME=vpn_service
DB_PORT=5432

JWT_SECRET=dev-secret-key-change-in-production
API_PORT=3000
NG_PORT=4200
```

⚠️ **IMPORTANT**: Change these in production!

---

## 📈 Performance

### Response Times (on local machine)
- Health check: ~1ms
- Login: ~200ms (password hashing)
- List peers: ~5ms
- Create peer: ~10ms

### Database Queries
- All queries use sqlx with compile-time verification
- Indices on frequently queried columns (user_id, peer_id, node_id)
- Connection pooling via sqlx

---

## 🎯 Next Steps

1. **Register a new account** in the UI
2. **Create some peers** (you need at least one node)
3. **Download WireGuard config** for a peer
4. **Import into WireGuard client** on your device
5. **Test connection** to the VPN

---

## 📝 Common Tasks

### Create Test Data Programmatically
```bash
# Register multiple users
for i in {1..5}; do
  curl -X POST http://localhost:3000/auth/register \
    -H "Content-Type: application/json" \
    -d "{
      \"email\": \"user$i@example.com\",
      \"password\": \"TestPassword$i\"
    }"
done
```

### Export Database Backup
```bash
docker-compose exec postgres pg_dump -U vpn_user vpn_service > backup.sql
```

### Restore Database from Backup
```bash
docker-compose exec -T postgres psql -U vpn_user vpn_service < backup.sql
```

### Check Database Size
```bash
docker-compose exec postgres psql -U vpn_user vpn_service -c "\l+"
```

### Monitor Live Logs
```bash
# Backend logs
docker-compose logs -f api

# Frontend logs
docker-compose logs -f frontend

# Database logs
docker-compose logs -f postgres
```

---

## 🚀 Deployment

### Production Build
```bash
# Backend
cargo build --release

# Frontend
npm run build:prod

# Docker image
docker build -t vpn-service:latest .
docker push <registry>/vpn-service:latest
```

---

## 📞 Support

For issues or questions:
1. Check [PROJECT_STATUS.md](./PROJECT_STATUS.md) for current state
2. Review [API_DOCUMENTATION.md](./API_DOCUMENTATION.md) for endpoints
3. Check Docker logs: `docker-compose logs`
4. Verify .env configuration
5. Ensure all ports are available (3000, 4200, 5432, 6379)

---

**Last Updated**: 2024  
**Version**: 1.0-MVP
