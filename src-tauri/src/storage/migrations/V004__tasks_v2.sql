-- V004__tasks_v2.sql
-- Rebuild tasks table with full Task data model for Agent Task System.
-- Drop existing minimal tasks table (no production data) and recreate.

DROP TABLE IF EXISTS tasks;

CREATE TABLE tasks (
    id              TEXT PRIMARY KEY,          -- UUID
    title           TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    status          TEXT NOT NULL DEFAULT 'todo'
                    CHECK(status IN ('todo','in_progress','in_review','done','blocked','cancelled')),
    priority        INTEGER NOT NULL DEFAULT 3 CHECK(priority BETWEEN 1 AND 5),
    creator_type    TEXT NOT NULL DEFAULT 'user',   -- user | agent
    creator_id      TEXT NOT NULL DEFAULT '',        -- user_id or agent_id, always populated for audit
    assignee_id     TEXT,                           -- agent_id (disk directory name, app-layer validation)
    channel_id      TEXT,                           -- bound Channel ID (JSON store, app-layer validation)
    thread_id       TEXT,                           -- bound Thread (optional)
    parent_task_id  TEXT,                           -- parent Task (optional)
    execution_mode  TEXT NOT NULL DEFAULT 'realtime'
                    CHECK(execution_mode IN ('realtime','async')),
    source          TEXT NOT NULL DEFAULT 'manual'
                    CHECK(source IN ('manual','conversation','agent_created','subtask')),
    source_message_id TEXT,                         -- source message ID (if generated from conversation)
    result          TEXT,                           -- execution result summary
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at    TEXT,
    FOREIGN KEY (parent_task_id) REFERENCES tasks(id) ON DELETE SET NULL
);

-- Indexes: high-frequency query fields
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_assignee ON tasks(assignee_id);
CREATE INDEX idx_tasks_channel ON tasks(channel_id);
CREATE INDEX idx_tasks_parent ON tasks(parent_task_id);
CREATE INDEX idx_tasks_source ON tasks(source);

-- Task dependency table
CREATE TABLE IF NOT EXISTS task_dependencies (
    task_id         TEXT NOT NULL,
    depends_on_id   TEXT NOT NULL,  -- task_id depends on depends_on_id to complete
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (task_id, depends_on_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (depends_on_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE INDEX idx_task_deps_depends ON task_dependencies(depends_on_id);

-- Task history table (status change history)
CREATE TABLE IF NOT EXISTS task_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id     TEXT NOT NULL,
    field       TEXT NOT NULL,      -- status | assignee_id | priority | ...
    old_value   TEXT,
    new_value   TEXT,
    changed_by  TEXT NOT NULL,      -- user:{id} | agent:{agent_id}
    changed_at  TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE INDEX idx_task_history_task ON task_history(task_id);
