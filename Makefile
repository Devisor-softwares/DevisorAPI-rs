# Game Server Panel Backend Makefile
# Compatible with OrbStack Docker

.PHONY: help build run test clean docker-up docker-down docker-logs docker-reset

# Default target
help: ## Show this help message
	@echo "Game Server Panel Backend - Development Commands"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Development commands
build: ## Build the Rust project
	cargo build

build-release: ## Build the project in release mode
	cargo build --release

run-simple: ## Run the simple server (no database)
	cargo run --bin simple

run-full: ## Run the full server (requires database)
	cargo run --bin backend

run-improved: ## Run the improved server with status monitoring
	cargo run --bin improved

run-dev: ## Run in development mode with auto-reload
	cargo watch -x run --bin simple

test: ## Run all tests
	cargo test

test-verbose: ## Run tests with verbose output
	cargo test -- --nocapture

check: ## Check code for errors
	cargo check

fmt: ## Format code
	cargo fmt

lint: ## Run linter
	cargo clippy

clean: ## Clean build artifacts
	cargo clean

# Docker commands (OrbStack compatible)
docker-up: ## Start PostgreSQL database with Docker Compose
	docker-compose up -d postgres
	@echo "PostgreSQL starting up..."
	@echo "Wait 30 seconds for database to be ready..."
	@sleep 30
	@echo "Database should be ready at: postgresql://panel_user:panel_password@localhost:5432/game_panel"

docker-up-all: ## Start PostgreSQL and PgAdmin
	docker-compose up -d
	@echo "Services starting up..."
	@echo "PostgreSQL: postgresql://panel_user:panel_password@localhost:5432/game_panel"
	@echo "PgAdmin: http://localhost:5050 (admin@example.com / admin123)"

docker-down: ## Stop Docker services
	docker-compose down

docker-logs: ## Show Docker logs
	docker-compose logs -f

docker-logs-postgres: ## Show PostgreSQL logs only
	docker-compose logs -f postgres

docker-reset: ## Reset Docker volumes (WARNING: deletes all data)
	docker-compose down -v
	docker volume prune -f

docker-status: ## Show Docker service status
	docker-compose ps

# Database commands
db-wait: ## Wait for database to be ready
	@echo "Waiting for database..."
	@until docker-compose exec -T postgres pg_isready -U panel_user -d game_panel; do \
		echo "Database not ready, waiting..."; \
		sleep 2; \
	done
	@echo "Database is ready!"

db-migrate: ## Run database migrations
	@echo "Running migrations..."
	cargo run --bin backend

db-shell: ## Connect to database shell
	docker-compose exec postgres psql -U panel_user -d game_panel

db-backup: ## Backup database
	docker-compose exec postgres pg_dump -U panel_user game_panel > backup_$(shell date +%Y%m%d_%H%M%S).sql
	@echo "Database backed up to backup_$(shell date +%Y%m%d_%H%M%S).sql"

db-restore: ## Restore database (usage: make db-restore FILE=backup.sql)
	@if [ -z "$(FILE)" ]; then echo "Usage: make db-restore FILE=backup.sql"; exit 1; fi
	docker-compose exec -T postgres psql -U panel_user game_panel < $(FILE)
	@echo "Database restored from $(FILE)"

# Development workflow
dev-setup: ## Set up development environment
	@echo "Setting up development environment..."
	@if ! command -v docker &> /dev/null; then \
		echo "Docker is required but not installed. Please install Docker first."; \
		exit 1; \
	fi
	@if ! command -v docker-compose &> /dev/null; then \
		echo "Docker Compose is required but not installed. Please install Docker Compose first."; \
		exit 1; \
	fi
	@echo "Creating .env file if it doesn't exist..."
	@if [ ! -f .env ]; then \
		cp .env.example .env; \
		echo "Created .env file - please update with your settings"; \
	fi
	@echo "Development environment setup complete!"

dev-start: ## Start development environment (database + simple server)
	make docker-up
	make run-simple

dev-full: ## Start full development environment (database + full server)
	make docker-up
	sleep 30
	make run-full

dev-stop: ## Stop development environment
	make docker-down
	@echo "Development environment stopped"

# Production commands
docker-build: ## Build Docker image
	docker build -t game-panel-backend .

docker-run: ## Run Docker container
	docker run -p 3000:3000 --env-file .env game-panel-backend

# Utility commands
install: ## Install Rust dependencies
	cargo install cargo-watch cargo-audit

audit: ## Security audit
	cargo audit

update: ## Update dependencies
	cargo update

docs: ## Generate documentation
	cargo doc --open

# Quick start commands
quick-test: ## Quick test without database
	make run-simple &
	@sleep 3
	@curl -s http://127.0.0.1:3000/health | jq .
	@curl -s http://127.0.0.1:3000/api/v1/status | jq .
	@pkill -f "cargo run --bin simple" || true

full-test: ## Full test with database
	make docker-up
	make db-wait
	make run-full &
	@sleep 5
	@curl -s http://127.0.0.1:3000/health | jq .
	@make docker-down

improved-test: ## Test improved server with status monitoring
	make docker-up
	make db-wait
	make run-improved &
	@sleep 5
	@curl -s http://127.0.0.1:3000/health | jq .
	@curl -s http://127.0.0.1:3000/status | jq .
	@make docker-down

# Environment management
env-dev: ## Set development environment
	cp .env.example .env.dev
	@echo "Created .env.dev - edit for development settings"

env-prod: ## Set production environment template
	cp .env.example .env.prod
	@echo "Created .env.prod - edit for production settings"
