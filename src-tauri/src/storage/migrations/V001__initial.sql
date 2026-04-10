-- V001__initial.sql
-- Initial table creation for SQLite metadata store.

-- Agent configuration and status
CREATE TABLE IF NOT EXISTS agents (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    emoji           TEXT NOT NULL DEFAULT 'robot',
    avatar_path     TEXT,
    enabled         INTEGER NOT NULL DEFAULT 1,
    runtime_type    TEXT NOT NULL DEFAULT 'claude-code',
    description     TEXT NOT NULL DEFAULT '',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Channel metadata
CREATE TABLE IF NOT EXISTS channels (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Channel <-> Agent many-to-many relationship
CREATE TABLE IF NOT EXISTS channel_members (
    channel_id      TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    agent_id        TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    role            TEXT NOT NULL DEFAULT 'member',
    joined_at       TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (channel_id, agent_id)
);

-- Thread metadata + JSONL pointer
CREATE TABLE IF NOT EXISTS threads (
    id              TEXT PRIMARY KEY,
    agent_id        TEXT NOT NULL,
    title           TEXT NOT NULL DEFAULT '',
    session_id      TEXT,
    message_count   INTEGER NOT NULL DEFAULT 0,
    jsonl_path      TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Task kanban board
CREATE TABLE IF NOT EXISTS tasks (
    id              TEXT PRIMARY KEY,
    title           TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'todo',
    assignee        TEXT,
    thread_id       TEXT,
    description     TEXT NOT NULL DEFAULT '',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Skill configuration
CREATE TABLE IF NOT EXISTS skills (
    id              TEXT PRIMARY KEY,
    agent_id        TEXT NOT NULL,
    name            TEXT NOT NULL,
    skill_type      TEXT NOT NULL DEFAULT 'tool',
    status          TEXT NOT NULL DEFAULT 'active',
    config_json     TEXT NOT NULL DEFAULT '{}',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Activity timeline index
CREATE TABLE IF NOT EXISTS activity_log (
    id              TEXT PRIMARY KEY,
    timestamp       TEXT NOT NULL DEFAULT (datetime('now')),
    activity_type   TEXT NOT NULL DEFAULT 'system',
    agent_id        TEXT,
    workspace_id    TEXT,
    summary         TEXT NOT NULL DEFAULT '',
    details_json    TEXT NOT NULL DEFAULT '{}'
);

-- Indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_threads_agent_id ON threads(agent_id);
CREATE INDEX IF NOT EXISTS idx_threads_updated_at ON threads(updated_at);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_assignee ON tasks(assignee);
CREATE INDEX IF NOT EXISTS idx_skills_agent_id ON skills(agent_id);
CREATE INDEX IF NOT EXISTS idx_activity_log_timestamp ON activity_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_activity_log_agent_id ON activity_log(agent_id);
CREATE INDEX IF NOT EXISTS idx_activity_log_type ON activity_log(activity_type);
