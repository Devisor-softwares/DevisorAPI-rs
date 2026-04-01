# OrbStack Docker Setup for Game Server Panel

## 🐳 OrbStack Compatible Docker Configuration

This setup is optimized for OrbStack on macOS, providing seamless PostgreSQL integration for the Game Server Panel backend.

## 🚀 Quick Start with OrbStack

### 1. Start Database Services
```bash
# Start PostgreSQL only (recommended for development)
make docker-up

# Or start PostgreSQL + PgAdmin for database management
make docker-up-all
```

### 2. Run the Backend
```bash
# Simple mode (no database) - for testing
make run-simple

# Full mode (with database) - for complete functionality
make run-full
```

### 3. Access Services
- **Backend API**: http://127.0.0.1:3000
- **PostgreSQL**: postgresql://panel_user:panel_password@localhost:5432/game_panel
- **PgAdmin**: http://localhost:5050 (if using `docker-up-all`)

## 📋 Available Make Commands

### Development Commands
```bash
make help           # Show all available commands
make build          # Build Rust project
make run-simple     # Run simple server (no database)
make run-full       # Run full server (with database)
make test           # Run tests
make fmt            # Format code
make lint           # Run linter
```

### Docker Commands (OrbStack Optimized)
```bash
make docker-up      # Start PostgreSQL
make docker-up-all  # Start PostgreSQL + PgAdmin
make docker-down    # Stop services
make docker-logs    # View logs
make docker-reset   # Reset volumes (WARNING: deletes data)
```

### Database Commands
```bash
make db-wait        # Wait for database to be ready
make db-shell       # Connect to PostgreSQL shell
make db-backup      # Backup database
make db-restore     # Restore database
```

### Development Workflow
```bash
make dev-setup      # Set up development environment
make dev-start      # Start database + simple server
make dev-full       # Start database + full server
make dev-stop       # Stop all services
```

## 🗄️ Database Configuration

### PostgreSQL Settings
- **Database**: game_panel
- **User**: panel_user
- **Password**: panel_password
- **Port**: 5432
- **Host**: localhost

### Connection String
```
postgresql://panel_user:panel_password@localhost:5432/game_panel
```

### Environment Variables
Update your `.env` file:
```env
DATABASE_URL=postgresql://panel_user:panel_password@localhost:5432/game_panel
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
RUST_LOG=info
```

## 🐛 OrbStack Troubleshooting

### Common Issues

#### 1. Port Conflicts
```bash
# Check what's using port 5432
lsof -i :5432

# OrbStack usually handles this automatically, but if needed:
make docker-down
make docker-up
```

#### 2. Database Connection Issues
```bash
# Check database status
make docker-status

# Wait for database to be ready
make db-wait

# View database logs
make docker-logs-postgres
```

#### 3. OrbStack Volume Issues
```bash
# Reset OrbStack volumes (last resort)
make docker-reset

# Or manually in OrbStack UI:
# 1. Open OrbStack
# 2. Go to Volumes
# 3. Delete game-panel volumes
# 4. Restart services
```

#### 4. Performance Issues
```bash
# OrbStack performance tips:
# 1. Ensure OrbStack is updated
# 2. Allocate enough resources in OrbStack settings
# 3. Use `make docker-up` instead of `docker-up-all` for better performance
```

## 🔧 OrbStack Optimization

### Resource Allocation
In OrbStack settings, ensure:
- **Memory**: At least 4GB
- **CPU**: At least 2 cores
- **Disk**: At least 10GB free space

### Network Configuration
OrbStack should automatically handle port forwarding, but verify:
- Port 5432 accessible for PostgreSQL
- Port 3000 accessible for backend
- Port 5050 accessible for PgAdmin (if used)

### Volume Management
OrbStack volumes are persistent across restarts:
```bash
# View volumes
docker volume ls | grep game-panel

# Backup before reset
make db-backup
make docker-reset
# Then restore if needed
```

## 🚀 Production Deployment

### Build Docker Image
```bash
make docker-build
```

### Run Production Container
```bash
# Production mode with database
docker run -p 3000:3000 \
  --env-file .env.prod \
  --network game-panel_game-panel-network \
  game-panel-backend:latest
```

### OrbStack Production
```bash
# Use OrbStack for production containers
docker-compose -f docker-compose.prod.yml up -d
```

## 📊 Monitoring

### Health Checks
```bash
# Backend health
curl http://127.0.0.1:3000/health

# Database health
docker-compose exec postgres pg_isready -U panel_user -d game_panel
```

### Logs
```bash
# Backend logs
make docker-logs

# Database logs
make docker-logs-postgres
```

## 🔄 Development Workflow

### Daily Development
```bash
# Start your day
make dev-setup    # First time setup
make dev-start    # Start services

# Work on code...
make fmt          # Format code
make test         # Run tests

# End your day
make dev-stop     # Stop services
```

### Testing Changes
```bash
# Quick test without database
make quick-test

# Full test with database
make full-test

# Development with auto-reload
make run-dev      # Uses cargo-watch
```

## 🎯 Best Practices

### 1. Use Simple Mode for Development
- `make run-simple` for frontend development
- `make run-full` only when testing database features

### 2. Database Management
- Always backup before major changes: `make db-backup`
- Use `make db-shell` for direct database access
- Monitor logs with `make docker-logs-postgres`

### 3. Resource Management
- Stop services when not working: `make dev-stop`
- Use `make docker-reset` only when necessary
- Monitor OrbStack resource usage

### 4. Environment Management
- Keep `.env` out of version control
- Use `make env-dev` and `make env-prod` for different environments
- Update JWT secret for production

## 🆘 Getting Help

### Command Help
```bash
make help  # Show all available commands
```

### Common Issues
1. **Database not ready**: Run `make db-wait`
2. **Port conflicts**: Check OrbStack port settings
3. **Permission issues**: Ensure OrbStack has necessary permissions
4. **Performance issues**: Check OrbStack resource allocation

### Support
- Check OrbStack documentation for container management
- Review Docker Compose logs for service issues
- Use `make docker-logs` for troubleshooting

---

**Note**: This configuration is optimized for OrbStack but works with any Docker installation on macOS.
