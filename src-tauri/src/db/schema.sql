CREATE TABLE IF NOT EXISTS jobs (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    title         TEXT NOT NULL,
    company       TEXT NOT NULL,
    url           TEXT NOT NULL UNIQUE,
    source        TEXT NOT NULL DEFAULT 'linkedin',
    status        TEXT NOT NULL DEFAULT 'discovered',
    match_summary TEXT,
    discovered_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS applications (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id            INTEGER NOT NULL REFERENCES jobs(id),
    folder_path       TEXT,
    cv_path           TEXT,
    cover_letter_path TEXT,
    status            TEXT NOT NULL DEFAULT 'awaiting_approval',
    submitted_at      TEXT
);

CREATE TABLE IF NOT EXISTS pending_actions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id      INTEGER REFERENCES jobs(id),
    category    TEXT NOT NULL,
    description TEXT NOT NULL,
    resolved    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS profile (
    id            INTEGER PRIMARY KEY CHECK (id = 1),
    full_name     TEXT NOT NULL DEFAULT '',
    email         TEXT NOT NULL DEFAULT '',
    phone         TEXT NOT NULL DEFAULT '',
    location      TEXT NOT NULL DEFAULT '',
    cv_text       TEXT NOT NULL DEFAULT '',
    criteria_json TEXT NOT NULL DEFAULT '{}',
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sessions (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at       TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at         TEXT,
    found_count      INTEGER NOT NULL DEFAULT 0,
    submitted_count  INTEGER NOT NULL DEFAULT 0,
    end_reason       TEXT
);
