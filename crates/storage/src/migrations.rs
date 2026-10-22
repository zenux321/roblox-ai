//! Migrații versionate. Versiunea curentă este în `PRAGMA user_version`.
//! O migrație nouă se adaugă la final; cele existente nu se modifică.

pub const MIGRATIONS: &[&str] = &[V1];

const V1: &str = r#"
CREATE TABLE projects (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    root        TEXT NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE TABLE conversations (
    id          TEXT PRIMARY KEY,
    project_id  TEXT REFERENCES projects(id),
    title       TEXT NOT NULL,
    visibility  TEXT NOT NULL CHECK (visibility IN ('private', 'shared')),
    scope       TEXT NOT NULL CHECK (scope IN ('project', 'general')),
    is_new      INTEGER NOT NULL CHECK (is_new IN (0, 1)),
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    position    INTEGER NOT NULL
);

CREATE TABLE messages (
    id              TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    seq             INTEGER NOT NULL,
    kind            TEXT NOT NULL CHECK (kind IN ('person', 'agent', 'assistant')),
    body_json       TEXT NOT NULL,
    search_text     TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    UNIQUE (conversation_id, seq)
);

CREATE TABLE plans (
    id           TEXT NOT NULL,
    version      INTEGER NOT NULL CHECK (version >= 1),
    project_id   TEXT NOT NULL REFERENCES projects(id),
    status       TEXT NOT NULL CHECK (status IN ('draft', 'awaiting_approval', 'approved', 'superseded')),
    plan_json    TEXT NOT NULL,
    plan_sha256  TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    PRIMARY KEY (id, version)
);

CREATE TABLE plan_tasks (
    plan_id      TEXT NOT NULL,
    plan_version INTEGER NOT NULL,
    task_id      TEXT NOT NULL,
    position     INTEGER NOT NULL,
    PRIMARY KEY (plan_id, plan_version, task_id),
    FOREIGN KEY (plan_id, plan_version) REFERENCES plans(id, version)
);

CREATE TABLE approvals (
    id             TEXT PRIMARY KEY,
    plan_id        TEXT NOT NULL,
    plan_version   INTEGER NOT NULL,
    plan_sha256    TEXT NOT NULL,
    approved_by    TEXT NOT NULL,
    identity_kind  TEXT NOT NULL,
    approved_at    TEXT NOT NULL,
    revoked_at     TEXT,
    FOREIGN KEY (plan_id, plan_version) REFERENCES plans(id, version)
);

CREATE TABLE runs (
    id            TEXT PRIMARY KEY,
    plan_id       TEXT NOT NULL,
    plan_version  INTEGER NOT NULL,
    plan_sha256   TEXT NOT NULL,
    approval_id   TEXT NOT NULL REFERENCES approvals(id),
    status        TEXT NOT NULL,
    state_json    TEXT NOT NULL,
    results_json  TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    FOREIGN KEY (plan_id, plan_version) REFERENCES plans(id, version)
);

CREATE TABLE task_attempts (
    run_id          TEXT NOT NULL REFERENCES runs(id),
    task_id         TEXT NOT NULL,
    attempt         INTEGER NOT NULL,
    status          TEXT NOT NULL,
    receipt_sha256  TEXT,
    started_at      TEXT NOT NULL,
    finished_at     TEXT,
    PRIMARY KEY (run_id, task_id, attempt)
);

CREATE TABLE run_events (
    seq           INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id        TEXT NOT NULL REFERENCES runs(id),
    kind          TEXT NOT NULL,
    level         TEXT NOT NULL CHECK (level IN ('info', 'ok', 'warn', 'error')),
    message       TEXT NOT NULL,
    payload_json  TEXT NOT NULL,
    created_at    TEXT NOT NULL
);
CREATE INDEX run_events_by_run ON run_events(run_id, seq);

CREATE TABLE run_file_changes (
    run_id       TEXT NOT NULL REFERENCES runs(id),
    path         TEXT NOT NULL,
    before_text  TEXT,
    after_text   TEXT NOT NULL,
    PRIMARY KEY (run_id, path)
);

CREATE TABLE budget_reservations (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id      TEXT,
    amount_usd  REAL NOT NULL CHECK (amount_usd >= 0),
    state       TEXT NOT NULL CHECK (state IN ('open', 'uncertain', 'committed', 'released')),
    actual_usd  REAL,
    created_at  TEXT NOT NULL
);

CREATE TABLE audit_events (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id       TEXT,
    kind         TEXT NOT NULL,
    detail_json  TEXT NOT NULL,
    created_at   TEXT NOT NULL
);
"#;
