# Setup Guide for Game Server Panel Backend

## 🚀 Quick Start (Simple Mode)

The backend is now running in simple mode! You can access:
- **http://127.0.0.1:3000** - API root
- **http://127.0.0.1:3000/health** - Health check  
- **http://127.0.0.1:3000/api/v1/status** - API status

## 🗄️ Full Database Setup

To enable all features (authentication, server management, monitoring), you need PostgreSQL:

### Option 1: Docker PostgreSQL (Recommended)

```bash
# Start PostgreSQL container
docker run --name game-panel-db \
  -e POSTGRES_DB=game_panel \
  -e POSTGRES_USER=panel_user \
  -e POSTGRES_PASSWORD=panel_password \
  -p 5432:5432 \
  -d postgres:15

# Wait for it to start (10 seconds)
sleep 10
```

### Option 2: Local PostgreSQL Installation

**macOS with Homebrew:**
```bash
brew install postgresql@15
brew services start postgresql@15
createuser -s postgres
createdb game_panel
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo -u postgres createuser --interactive
sudo -u postgres createdb game_panel
```

### Configure Environment

Update your `.env` file:
```env
DATABASE_URL=postgresql://panel_user:panel_password@localhost:5432/game_panel
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
RUST_LOG=info
```

### Run Full Backend

```bash
# Stop the simple server first
kill %1 2>/dev/null || true

# Run the full backend
cargo run --bin backend
```

## 🧪 Testing the API

### 1. Health Check
```bash
curl http://127.0.0.1:3000/health
```

### 2. Register a User
```bash
curl -X POST http://127.0.0.1:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "email": "admin@example.com",
    "password": "password123",
    "role": "admin"
  }'
```

### 3. Login
```bash
curl -X POST http://127.0.0.1:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "password123"
  }'
```

### 4. Create a Server (with auth token)
```bash
# Replace YOUR_TOKEN_HERE with the token from login response
curl -X POST http://127.0.0.1:3000/api/v1/servers \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" \
  -d '{
    "name": "My Minecraft Server",
    "description": "A fun Minecraft server",
    "game_type": "minecraft",
    "ip_address": "192.168.1.100",
    "port": 25565
  }'
```

## 🔧 Development Commands

```bash
# Run simple server (no database)
cargo run --bin simple

# Run full server (requires database)
cargo run --bin backend

# Run tests
cargo test

# Check code
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy

# Build for production
cargo build --release
```

## 🐛 Troubleshooting

### Database Connection Issues
```bash
# Check if PostgreSQL is running
pg_isready

# Check connection
psql -h localhost -U panel_user -d game_panel

# View logs
docker logs game-panel-db  # if using Docker
```

### Port Already in Use
```bash
# Find process using port 3000
lsof -i :3000

# Kill it
kill -9 <PID>

# Or use different port
cargo run --bin backend -- --port 3001
```

### Permission Issues (macOS)
If you get permission errors on macOS:
```bash
# Give Terminal/IDE permission to access local network
# System Preferences > Privacy & Security > Local Network > Add your terminal/IDE
```

## 🏗️ Architecture Overview

### Simple Mode (Current)
- ✅ HTTP server
- ✅ CORS support  
- ✅ Basic endpoints
- ❌ No database
- ❌ No authentication
- ❌ No server management

### Full Mode (With Database)
- ✅ All simple mode features
- ✅ PostgreSQL database
- ✅ User authentication
- ✅ Server management
- ✅ Metrics collection
- ✅ Real-time monitoring

## 📱 Frontend Integration

Once the backend is running, your frontend can connect to:

```javascript
// API base URL
const API_BASE = 'http://127.0.0.1:3000/api/v1';

// Example API call
async function login(email, password) {
  const response = await fetch(`${API_BASE}/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password })
  });
  return response.json();
}
```

## 🔒 Security Notes

1. **Change the JWT secret** in production
2. **Use HTTPS** in production
3. **Validate all inputs** (already implemented)
4. **Use environment variables** for secrets
5. **Enable authentication middleware** (prepared but not active)

## 📈 Next Steps

1. **Set up database** using the guide above
2. **Test all endpoints** with Postman or curl
3. **Implement server control logic** (SSH/Docker)
4. **Add real-time monitoring** (WebSocket)
5. **Create frontend integration**
6. **Deploy to production** (Docker, cloud)

---

**Current Status**: ✅ Simple server running at http://127.0.0.1:3000  
**Next Step**: Set up PostgreSQL for full functionality
