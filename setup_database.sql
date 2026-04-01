-- Drop everything if it exists
DROP TABLE IF EXISTS server_metrics CASCADE;
DROP TABLE IF EXISTS servers CASCADE;
DROP TABLE IF EXISTS users CASCADE;
DROP TYPE IF EXISTS server_status;
DROP TYPE IF EXISTS game_type;
DROP TYPE IF EXISTS user_role;

-- Create types
CREATE TYPE user_role AS ENUM ('admin', 'user');
CREATE TYPE game_type AS ENUM (
    'minecraft',
    'valheim', 
    'rust',
    'ark',
    'fivem',
    'counterstrike',
    'custom'
);
CREATE TYPE server_status AS ENUM (
    'online',
    'offline',
    'starting',
    'stopping',
    'error'
);

-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role user_role NOT NULL DEFAULT 'user',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create servers table
CREATE TABLE servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    game_type game_type NOT NULL,
    ip_address INET NOT NULL,
    port INTEGER NOT NULL CHECK (port > 0 AND port < 65536),
    status server_status NOT NULL DEFAULT 'offline',
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create server metrics table
CREATE TABLE server_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    cpu_usage DECIMAL(5,2) NOT NULL CHECK (cpu_usage >= 0 AND cpu_usage <= 100),
    memory_usage DECIMAL(5,2) NOT NULL CHECK (memory_usage >= 0 AND memory_usage <= 100),
    disk_usage DECIMAL(5,2) NOT NULL CHECK (disk_usage >= 0 AND disk_usage <= 100),
    network_in BIGINT NOT NULL DEFAULT 0,
    network_out BIGINT NOT NULL DEFAULT 0,
    player_count INTEGER,
    max_players INTEGER,
    recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_servers_owner_id ON servers(owner_id);
CREATE INDEX idx_servers_status ON servers(status);
CREATE INDEX idx_servers_game_type ON servers(game_type);
CREATE INDEX idx_server_metrics_server_id ON server_metrics(server_id);
CREATE INDEX idx_server_metrics_recorded_at ON server_metrics(recorded_at);

-- Insert sample admin user (password: admin123)
INSERT INTO users (username, email, password_hash, role) VALUES 
('admin', 'admin@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj6QJw/2Ej7W', 'admin');

-- Success message
SELECT 'Database setup completed successfully!' as status;
