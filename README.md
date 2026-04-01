# Game Server Panel Backend

A Rust-based backend API for managing game servers and VPS hosting.

## Features

- **User Management**: Registration, authentication, and role-based access control
- **Server Management**: Create, update, delete, and monitor game servers
- **Multi-Game Support**: Support for various game types (Minecraft, Valheim, Rust, etc.)
- **Real-time Monitoring**: CPU, memory, disk usage, and player statistics
- **Server Control**: Start, stop, restart, and execute commands on servers
- **RESTful API**: Clean and well-documented API endpoints

## Tech Stack

- **Language**: Rust
- **Web Framework**: Axum
- **Database**: PostgreSQL with SQLx
- **Authentication**: JWT tokens
- **Async Runtime**: Tokio
- **Logging**: Tracing

## Getting Started

### Prerequisites

- Rust 1.70+
- PostgreSQL 12+
- Docker (optional)

### Installation

1. Clone the repository
2. Copy `.env.example` to `.env` and configure your settings
3. Install dependencies:
   ```bash
   cargo build
   ```

4. Run database migrations:
   ```bash
   cargo run -- --database-url "postgresql://user:password@localhost/database"
   ```

5. Start the server:
   ```bash
   cargo run
   ```

### Environment Variables

```env
DATABASE_URL=postgresql://username:password@localhost/database
JWT_SECRET=your-secret-key-here
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
RUST_LOG=info
```

## API Endpoints

### Authentication
- `POST /api/v1/auth/register` - Register a new user
- `POST /api/v1/auth/login` - Login and get JWT token

### Servers
- `GET /api/v1/servers` - List user's servers
- `POST /api/v1/servers` - Create a new server
- `GET /api/v1/servers/:id` - Get server details
- `PUT /api/v1/servers/:id` - Update server
- `DELETE /api/v1/servers/:id` - Delete server
- `POST /api/v1/servers/:id/start` - Start server
- `POST /api/v1/servers/:id/stop` - Stop server
- `POST /api/v1/servers/:id/restart` - Restart server
- `GET /api/v1/servers/:id/metrics` - Get server metrics

### Health
- `GET /health` - Health check endpoint

## Database Schema

### Users Table
- `id`: UUID primary key
- `username`: Unique username
- `email`: Unique email
- `password_hash`: Bcrypt hashed password
- `role`: User role (admin/user)
- `created_at`: Creation timestamp
- `updated_at`: Last update timestamp

### Servers Table
- `id`: UUID primary key
- `name`: Server name
- `description`: Optional description
- `game_type`: Type of game server
- `ip_address`: Server IP address
- `port`: Server port
- `status`: Current server status
- `owner_id`: Foreign key to users table
- `created_at`: Creation timestamp
- `updated_at`: Last update timestamp

### Server Metrics Table
- `id`: UUID primary key
- `server_id`: Foreign key to servers table
- `cpu_usage`: CPU usage percentage
- `memory_usage`: Memory usage percentage
- `disk_usage`: Disk usage percentage
- `network_in`: Network input bytes
- `network_out`: Network output bytes
- `player_count`: Current player count
- `max_players`: Maximum player count
- `recorded_at`: Metrics timestamp

## Development

### Running Tests
```bash
cargo test
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

## Architecture

The backend follows a layered architecture:

1. **Models**: Data structures and database entities
2. **Services**: Business logic and data access
3. **Handlers**: HTTP request/response handling
4. **Middleware**: Authentication, CORS, logging
5. **Database**: PostgreSQL with migrations

## Security

- Password hashing with bcrypt
- JWT-based authentication
- Role-based access control
- Input validation
- SQL injection prevention with parameterized queries

## Deployment

### Docker
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/backend /usr/local/bin/backend
CMD ["backend"]
```

### Environment Setup
Ensure the following environment variables are set in production:
- `DATABASE_URL`: PostgreSQL connection string
- `JWT_SECRET`: Secure JWT secret key
- `RUST_LOG`: Appropriate log level

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License.
