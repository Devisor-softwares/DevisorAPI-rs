CREATE TYPE IF NOT EXISTS game_type AS ENUM (
    'minecraft',
    'valheim', 
    'rust',
    'ark',
    'fivem',
    'counterstrike',
    'custom'
);

CREATE TYPE IF NOT EXISTS server_status AS ENUM (
    'online',
    'offline',
    'starting',
    'stopping',
    'error'
);

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

CREATE INDEX idx_servers_owner_id ON servers(owner_id);
CREATE INDEX idx_servers_status ON servers(status);
CREATE INDEX idx_servers_game_type ON servers(game_type);
