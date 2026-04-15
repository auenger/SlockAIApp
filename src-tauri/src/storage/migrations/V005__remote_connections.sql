-- V005__remote_connections.sql
-- Remote A2A connections + agents table extension for connection_mode

-- Remote connection endpoints
CREATE TABLE IF NOT EXISTS remote_connections (
    id                      TEXT PRIMARY KEY,
    name                    TEXT NOT NULL,
    endpoint_url            TEXT NOT NULL,
    auth_type               TEXT NOT NULL DEFAULT 'none',
    status                  TEXT NOT NULL DEFAULT 'unknown',
    cached_agent_card       TEXT,
    last_health_check_at    TEXT,
    created_at              TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at              TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Extend agents table with connection mode
ALTER TABLE agents ADD COLUMN connection_mode TEXT NOT NULL DEFAULT 'local';
ALTER TABLE agents ADD COLUMN remote_connection_id TEXT;

-- Indexes
CREATE INDEX IF NOT EXISTS idx_remote_connections_status ON remote_connections(status);
CREATE INDEX IF NOT EXISTS idx_remote_connections_name ON remote_connections(name);
CREATE INDEX IF NOT EXISTS idx_agents_connection_mode ON agents(connection_mode);
