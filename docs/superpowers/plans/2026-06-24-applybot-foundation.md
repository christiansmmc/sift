# applybot — Plan 1: Foundation & Data Layer

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stand up the applybot Tauri app with a tested SQLite data layer, a profile model, the Tauri command surface, and a navigable React shell that gates usage behind onboarding completion.

**Architecture:** Tauri v2 desktop app. Rust owns a SQLite database (the single source of truth) exposed to the frontend through Tauri commands. React 19 + TypeScript renders the shell: an onboarding gate plus four screen stubs. This plan delivers the skeleton everything else hangs off of — no agent, no parsing, no real screen content yet.

**Tech Stack:** Tauri v2, React 19, TypeScript 5, Vite, Rust 2021, rusqlite 0.31 (bundled), chrono 0.4, serde/serde_json 1.

## Global Constraints

- Platform: Windows 11. App is desktop-only (Tauri), not web.
- All code — identifiers, comments, DB schema, commands — is written in **English**.
- All user-facing UI strings are in **natural Brazilian Portuguese (pt-BR)**.
- SQLite is the **single source of truth** for app state. No YAML profile files.
- Crate versions match the proven baseline: `rusqlite = { version = "0.31", features = ["bundled"] }`, `chrono = { version = "0.4", features = ["serde"] }`, `tauri = "2"`, `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`.
- Status values are English enums: `discovered`, `analyzed`, `awaiting_approval`, `submitted`, `skipped`, `discarded`, `pending_review`.
- Frequent commits: one per task minimum, following Conventional Commits.

---

### Task 1: Scaffold the Tauri + React + TypeScript project

**Files:**
- Create: the full Tauri scaffold under `C:\Users\csequ\projects\applybot\` (`package.json`, `src/`, `src-tauri/`, `vite.config.ts`, etc.)
- Modify: `src-tauri/Cargo.toml` (package name + base deps)
- Modify: `src-tauri/tauri.conf.json` (product name, window title)

**Interfaces:**
- Consumes: nothing (greenfield).
- Produces: a runnable `npm run tauri dev` app named `applybot`. Rust crate lib name `applybot_lib`.

- [ ] **Step 1: Scaffold into the existing folder**

The folder already exists with a `docs/` dir and a git repo. Scaffold Tauri into it using the React+TS template. Run from `C:\Users\csequ\projects\`:

```bash
npm create tauri-app@latest applybot -- --template react-ts --manager npm --yes
```

If the CLI refuses because the directory is non-empty, scaffold into a temp dir and copy `src/`, `src-tauri/`, `index.html`, `package.json`, `vite.config.ts`, `tsconfig.json`, `.gitignore` over, preserving the existing `docs/` folder.

- [ ] **Step 2: Set the app identity**

Edit `src-tauri/Cargo.toml` — set `name = "applybot"`, `description = "Job-application agent powered by Claude + Chrome"`, `edition = "2021"`, and the lib block:

```toml
[lib]
name = "applybot_lib"
crate-type = ["staticlib", "cdylib", "rlib"]
```

Edit `src-tauri/tauri.conf.json` — set `"productName": "applybot"` and the main window `"title": "applybot"`.

- [ ] **Step 3: Add the data-layer dependencies**

In `src-tauri/Cargo.toml` under `[dependencies]`, ensure these are present:

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 4: Verify it builds and runs**

Run: `npm install` then `npm run tauri dev`
Expected: the default Tauri window opens titled "applybot". Close it.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "chore: scaffold applybot tauri + react-ts project"
```

---

### Task 2: Database schema and connection

**Files:**
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/schema.sql`
- Modify: `src-tauri/src/lib.rs` (declare `mod db;`)

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `db::open_in_memory() -> rusqlite::Connection` — a connection with the schema applied (for tests).
  - `db::open_at(path: &std::path::Path) -> rusqlite::Result<rusqlite::Connection>` — opens/creates a file DB with the schema applied.
  - `db::apply_schema(conn: &rusqlite::Connection) -> rusqlite::Result<()>` — runs `schema.sql` (idempotent via `IF NOT EXISTS`).

- [ ] **Step 1: Write the schema file**

Create `src-tauri/src/db/schema.sql`:

```sql
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
```

- [ ] **Step 2: Write the failing test for schema application**

Create `src-tauri/src/db/mod.rs`:

```rust
use rusqlite::Connection;
use std::path::Path;

const SCHEMA: &str = include_str!("schema.sql");

pub fn apply_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)
}

pub fn open_at(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    apply_schema(&conn)?;
    Ok(conn)
}

#[cfg(test)]
pub fn open_in_memory() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    apply_schema(&conn).expect("apply schema");
    conn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_creates_all_tables() {
        let conn = open_in_memory();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' \
                 AND name IN ('jobs','applications','pending_actions','profile','sessions')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn apply_schema_is_idempotent() {
        let conn = open_in_memory();
        apply_schema(&conn).expect("second apply must not fail");
    }
}
```

- [ ] **Step 3: Declare the module**

In `src-tauri/src/lib.rs`, add near the top:

```rust
mod db;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd src-tauri && cargo test db::`
Expected: `schema_creates_all_tables` and `apply_schema_is_idempotent` both PASS.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: sqlite schema and connection helpers"
```

---

### Task 3: Jobs store

**Files:**
- Create: `src-tauri/src/db/jobs.rs`
- Modify: `src-tauri/src/db/mod.rs` (add `pub mod jobs;` and shared `Job` types)

**Interfaces:**
- Consumes: `db::open_in_memory()` (tests), `rusqlite::Connection`.
- Produces:
  - `Job { id: i64, title: String, company: String, url: String, source: String, status: String, match_summary: Option<String>, discovered_at: String }` — `#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]`.
  - `NewJob { title: String, company: String, url: String, source: String }` — `#[derive(Debug, Clone, serde::Deserialize)]`.
  - `jobs::insert(conn: &Connection, job: &NewJob) -> rusqlite::Result<i64>` — returns new id; ignores duplicate url (returns existing id).
  - `jobs::list(conn: &Connection) -> rusqlite::Result<Vec<Job>>` — newest first.
  - `jobs::set_status(conn: &Connection, id: i64, status: &str, match_summary: Option<&str>) -> rusqlite::Result<()>`.

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/db/jobs.rs`:

```rust
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: i64,
    pub title: String,
    pub company: String,
    pub url: String,
    pub source: String,
    pub status: String,
    pub match_summary: Option<String>,
    pub discovered_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewJob {
    pub title: String,
    pub company: String,
    pub url: String,
    pub source: String,
}

pub fn insert(conn: &Connection, job: &NewJob) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO jobs (title, company, url, source) VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT(url) DO NOTHING",
        (&job.title, &job.company, &job.url, &job.source),
    )?;
    conn.query_row("SELECT id FROM jobs WHERE url = ?1", [&job.url], |r| r.get(0))
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Job>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, company, url, source, status, match_summary, discovered_at \
         FROM jobs ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Job {
            id: r.get(0)?,
            title: r.get(1)?,
            company: r.get(2)?,
            url: r.get(3)?,
            source: r.get(4)?,
            status: r.get(5)?,
            match_summary: r.get(6)?,
            discovered_at: r.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn set_status(
    conn: &Connection,
    id: i64,
    status: &str,
    match_summary: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE jobs SET status = ?2, match_summary = COALESCE(?3, match_summary) WHERE id = ?1",
        (id, status, match_summary),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn sample() -> NewJob {
        NewJob {
            title: "Backend Engineer".into(),
            company: "Acme".into(),
            url: "https://linkedin.com/jobs/1".into(),
            source: "linkedin".into(),
        }
    }

    #[test]
    fn insert_then_list_returns_the_job() {
        let conn = open_in_memory();
        let id = insert(&conn, &sample()).unwrap();
        let jobs = list(&conn).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].id, id);
        assert_eq!(jobs[0].status, "discovered");
    }

    #[test]
    fn insert_is_idempotent_on_duplicate_url() {
        let conn = open_in_memory();
        let a = insert(&conn, &sample()).unwrap();
        let b = insert(&conn, &sample()).unwrap();
        assert_eq!(a, b);
        assert_eq!(list(&conn).unwrap().len(), 1);
    }

    #[test]
    fn set_status_updates_status_and_summary() {
        let conn = open_in_memory();
        let id = insert(&conn, &sample()).unwrap();
        set_status(&conn, id, "analyzed", Some("covers 3/4 must-haves")).unwrap();
        let job = &list(&conn).unwrap()[0];
        assert_eq!(job.status, "analyzed");
        assert_eq!(job.match_summary.as_deref(), Some("covers 3/4 must-haves"));
    }
}
```

- [ ] **Step 2: Wire the module**

In `src-tauri/src/db/mod.rs`, add after the existing `use`:

```rust
pub mod jobs;
```

- [ ] **Step 3: Run the tests to verify they fail, then pass**

Run: `cd src-tauri && cargo test db::jobs`
Expected: compiles and all three tests PASS. (If you ran before wiring the module, expect a compile error — that is the failing state.)

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: jobs store with insert/list/set_status"
```

---

### Task 4: Applications store

**Files:**
- Create: `src-tauri/src/db/applications.rs`
- Modify: `src-tauri/src/db/mod.rs` (add `pub mod applications;`)

**Interfaces:**
- Consumes: `db::open_in_memory()`, `jobs::insert`.
- Produces:
  - `Application { id: i64, job_id: i64, folder_path: Option<String>, cv_path: Option<String>, cover_letter_path: Option<String>, status: String, submitted_at: Option<String> }` — Serialize/Deserialize.
  - `applications::create(conn, job_id: i64, folder_path: Option<&str>, cv_path: Option<&str>, cover_letter_path: Option<&str>) -> rusqlite::Result<i64>` — inserts with status `awaiting_approval`.
  - `applications::set_status(conn, id: i64, status: &str) -> rusqlite::Result<()>` — when status is `submitted`, stamps `submitted_at = datetime('now')`.
  - `applications::list(conn) -> rusqlite::Result<Vec<Application>>` — newest first.

- [ ] **Step 1: Write the failing tests and implementation**

Create `src-tauri/src/db/applications.rs`:

```rust
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub id: i64,
    pub job_id: i64,
    pub folder_path: Option<String>,
    pub cv_path: Option<String>,
    pub cover_letter_path: Option<String>,
    pub status: String,
    pub submitted_at: Option<String>,
}

pub fn create(
    conn: &Connection,
    job_id: i64,
    folder_path: Option<&str>,
    cv_path: Option<&str>,
    cover_letter_path: Option<&str>,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO applications (job_id, folder_path, cv_path, cover_letter_path, status) \
         VALUES (?1, ?2, ?3, ?4, 'awaiting_approval')",
        (job_id, folder_path, cv_path, cover_letter_path),
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn set_status(conn: &Connection, id: i64, status: &str) -> rusqlite::Result<()> {
    if status == "submitted" {
        conn.execute(
            "UPDATE applications SET status = ?2, submitted_at = datetime('now') WHERE id = ?1",
            (id, status),
        )?;
    } else {
        conn.execute(
            "UPDATE applications SET status = ?2 WHERE id = ?1",
            (id, status),
        )?;
    }
    Ok(())
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Application>> {
    let mut stmt = conn.prepare(
        "SELECT id, job_id, folder_path, cv_path, cover_letter_path, status, submitted_at \
         FROM applications ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Application {
            id: r.get(0)?,
            job_id: r.get(1)?,
            folder_path: r.get(2)?,
            cv_path: r.get(3)?,
            cover_letter_path: r.get(4)?,
            status: r.get(5)?,
            submitted_at: r.get(6)?,
        })
    })?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::jobs::{self, NewJob};
    use crate::db::open_in_memory;

    fn job(conn: &Connection) -> i64 {
        jobs::insert(
            conn,
            &NewJob {
                title: "Dev".into(),
                company: "Acme".into(),
                url: "https://linkedin.com/jobs/1".into(),
                source: "linkedin".into(),
            },
        )
        .unwrap()
    }

    #[test]
    fn create_defaults_to_awaiting_approval() {
        let conn = open_in_memory();
        let job_id = job(&conn);
        create(&conn, job_id, Some("/apps/acme"), None, None).unwrap();
        let app = &list(&conn).unwrap()[0];
        assert_eq!(app.status, "awaiting_approval");
        assert!(app.submitted_at.is_none());
    }

    #[test]
    fn submitting_stamps_submitted_at() {
        let conn = open_in_memory();
        let job_id = job(&conn);
        let id = create(&conn, job_id, None, None, None).unwrap();
        set_status(&conn, id, "submitted").unwrap();
        let app = &list(&conn).unwrap()[0];
        assert_eq!(app.status, "submitted");
        assert!(app.submitted_at.is_some());
    }
}
```

- [ ] **Step 2: Wire the module**

In `src-tauri/src/db/mod.rs` add: `pub mod applications;`

- [ ] **Step 3: Run the tests**

Run: `cd src-tauri && cargo test db::applications`
Expected: both tests PASS.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: applications store"
```

---

### Task 5: Pending actions store

**Files:**
- Create: `src-tauri/src/db/pending.rs`
- Modify: `src-tauri/src/db/mod.rs` (add `pub mod pending;`)

**Interfaces:**
- Consumes: `db::open_in_memory()`, `jobs::insert`.
- Produces:
  - `PendingAction { id: i64, job_id: Option<i64>, category: String, description: String, resolved: bool, created_at: String }` — Serialize/Deserialize.
  - `pending::create(conn, job_id: Option<i64>, category: &str, description: &str) -> rusqlite::Result<i64>`.
  - `pending::list_open(conn) -> rusqlite::Result<Vec<PendingAction>>` — only `resolved = 0`, newest first.
  - `pending::resolve(conn, id: i64) -> rusqlite::Result<()>`.
  - `pending::count_open(conn) -> rusqlite::Result<i64>`.

- [ ] **Step 1: Write the failing tests and implementation**

Create `src-tauri/src/db/pending.rs`:

```rust
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAction {
    pub id: i64,
    pub job_id: Option<i64>,
    pub category: String,
    pub description: String,
    pub resolved: bool,
    pub created_at: String,
}

pub fn create(
    conn: &Connection,
    job_id: Option<i64>,
    category: &str,
    description: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO pending_actions (job_id, category, description) VALUES (?1, ?2, ?3)",
        (job_id, category, description),
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_open(conn: &Connection) -> rusqlite::Result<Vec<PendingAction>> {
    let mut stmt = conn.prepare(
        "SELECT id, job_id, category, description, resolved, created_at \
         FROM pending_actions WHERE resolved = 0 ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(PendingAction {
            id: r.get(0)?,
            job_id: r.get(1)?,
            category: r.get(2)?,
            description: r.get(3)?,
            resolved: r.get::<_, i64>(4)? != 0,
            created_at: r.get(5)?,
        })
    })?;
    rows.collect()
}

pub fn resolve(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("UPDATE pending_actions SET resolved = 1 WHERE id = ?1", [id])?;
    Ok(())
}

pub fn count_open(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM pending_actions WHERE resolved = 0", [], |r| r.get(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn create_then_list_open_shows_unresolved() {
        let conn = open_in_memory();
        create(&conn, None, "captcha", "captcha on linkedin").unwrap();
        assert_eq!(list_open(&conn).unwrap().len(), 1);
        assert_eq!(count_open(&conn).unwrap(), 1);
    }

    #[test]
    fn resolve_hides_from_list_open() {
        let conn = open_in_memory();
        let id = create(&conn, None, "salary", "salary out of range").unwrap();
        resolve(&conn, id).unwrap();
        assert_eq!(list_open(&conn).unwrap().len(), 0);
        assert_eq!(count_open(&conn).unwrap(), 0);
    }
}
```

- [ ] **Step 2: Wire the module**

In `src-tauri/src/db/mod.rs` add: `pub mod pending;`

- [ ] **Step 3: Run the tests**

Run: `cd src-tauri && cargo test db::pending`
Expected: both tests PASS.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: pending actions store"
```

---

### Task 6: Profile store and onboarding-complete logic

**Files:**
- Create: `src-tauri/src/db/profile.rs`
- Modify: `src-tauri/src/db/mod.rs` (add `pub mod profile;`)

**Interfaces:**
- Consumes: `db::open_in_memory()`.
- Produces:
  - `Profile { full_name: String, email: String, phone: String, location: String, cv_text: String, criteria_json: String }` — Serialize/Deserialize.
  - `profile::get(conn) -> rusqlite::Result<Profile>` — returns the row; if absent, returns an all-empty Profile with `criteria_json = "{}"`.
  - `profile::upsert(conn, p: &Profile) -> rusqlite::Result<()>` — writes the single row (id=1), stamping `updated_at`.
  - `profile::is_onboarding_complete(conn, has_linkedin_credentials: bool) -> rusqlite::Result<bool>` — true when `full_name`, `cv_text`, and a non-empty `criteria_json` object are present AND `has_linkedin_credentials` is true. Credentials live in the OS keychain (Plan 3), so the caller passes that flag in.

- [ ] **Step 1: Write the failing tests and implementation**

Create `src-tauri/src/db/profile.rs`:

```rust
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub location: String,
    pub cv_text: String,
    pub criteria_json: String,
}

pub fn get(conn: &Connection) -> rusqlite::Result<Profile> {
    let found = conn
        .query_row(
            "SELECT full_name, email, phone, location, cv_text, criteria_json \
             FROM profile WHERE id = 1",
            [],
            |r| {
                Ok(Profile {
                    full_name: r.get(0)?,
                    email: r.get(1)?,
                    phone: r.get(2)?,
                    location: r.get(3)?,
                    cv_text: r.get(4)?,
                    criteria_json: r.get(5)?,
                })
            },
        )
        .optional()?;
    Ok(found.unwrap_or(Profile {
        criteria_json: "{}".into(),
        ..Default::default()
    }))
}

pub fn upsert(conn: &Connection, p: &Profile) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO profile (id, full_name, email, phone, location, cv_text, criteria_json, updated_at) \
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, datetime('now')) \
         ON CONFLICT(id) DO UPDATE SET \
            full_name=?1, email=?2, phone=?3, location=?4, cv_text=?5, criteria_json=?6, updated_at=datetime('now')",
        (&p.full_name, &p.email, &p.phone, &p.location, &p.cv_text, &p.criteria_json),
    )?;
    Ok(())
}

pub fn is_onboarding_complete(
    conn: &Connection,
    has_linkedin_credentials: bool,
) -> rusqlite::Result<bool> {
    let p = get(conn)?;
    let criteria_present = p.criteria_json.trim() != "{}" && !p.criteria_json.trim().is_empty();
    Ok(!p.full_name.trim().is_empty()
        && !p.cv_text.trim().is_empty()
        && criteria_present
        && has_linkedin_credentials)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn get_on_empty_db_returns_blank_profile() {
        let conn = open_in_memory();
        let p = get(&conn).unwrap();
        assert_eq!(p.full_name, "");
        assert_eq!(p.criteria_json, "{}");
    }

    #[test]
    fn upsert_then_get_roundtrips() {
        let conn = open_in_memory();
        let p = Profile {
            full_name: "Christian".into(),
            email: "c@example.com".into(),
            phone: "".into(),
            location: "Brazil".into(),
            cv_text: "10 years backend".into(),
            criteria_json: r#"{"role":"backend"}"#.into(),
        };
        upsert(&conn, &p).unwrap();
        let got = get(&conn).unwrap();
        assert_eq!(got.full_name, "Christian");
        assert_eq!(got.criteria_json, r#"{"role":"backend"}"#);
    }

    #[test]
    fn upsert_twice_keeps_single_row() {
        let conn = open_in_memory();
        let mut p = Profile { full_name: "A".into(), criteria_json: "{}".into(), ..Default::default() };
        upsert(&conn, &p).unwrap();
        p.full_name = "B".into();
        upsert(&conn, &p).unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM profile", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
        assert_eq!(get(&conn).unwrap().full_name, "B");
    }

    #[test]
    fn onboarding_incomplete_without_all_fields() {
        let conn = open_in_memory();
        // empty profile, no creds
        assert!(!is_onboarding_complete(&conn, false).unwrap());
        // full profile but no creds
        upsert(&conn, &Profile {
            full_name: "C".into(),
            cv_text: "cv".into(),
            criteria_json: r#"{"role":"backend"}"#.into(),
            ..Default::default()
        }).unwrap();
        assert!(!is_onboarding_complete(&conn, false).unwrap());
        // full profile + creds
        assert!(is_onboarding_complete(&conn, true).unwrap());
    }
}
```

- [ ] **Step 2: Wire the module**

In `src-tauri/src/db/mod.rs` add: `pub mod profile;`

- [ ] **Step 3: Run the tests**

Run: `cd src-tauri && cargo test db::profile`
Expected: all four tests PASS.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: profile store and onboarding-complete check"
```

---

### Task 7: App state and database initialization on startup

**Files:**
- Create: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/lib.rs` (declare `mod state;`, set up managed state in the builder)

**Interfaces:**
- Consumes: `db::open_at`, the Tauri `App` handle.
- Produces:
  - `state::AppState { db: std::sync::Mutex<rusqlite::Connection> }`.
  - `state::init(app: &tauri::App) -> AppState` — resolves the app-data dir, ensures it exists, opens `applybot.db` there with the schema applied, returns the state. Used in `.setup()`.

- [ ] **Step 1: Write the state module**

Create `src-tauri/src/state.rs`:

```rust
use std::sync::Mutex;
use rusqlite::Connection;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Connection>,
}

pub fn init(app: &tauri::App) -> AppState {
    let data_dir = app
        .path()
        .app_data_dir()
        .expect("resolve app data dir");
    std::fs::create_dir_all(&data_dir).expect("create app data dir");
    let conn = crate::db::open_at(&data_dir.join("applybot.db")).expect("open applybot.db");
    AppState { db: Mutex::new(conn) }
}
```

- [ ] **Step 2: Register state in the Tauri builder**

In `src-tauri/src/lib.rs`, declare the module (`mod state;`) and inside the `tauri::Builder` chain add a `.setup()` that manages the state. The builder should look like:

```rust
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_state = state::init(app);
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![]) // commands added in Task 8
        .run(tauri::generate_context!())
        .expect("error while running applybot");
}
```

- [ ] **Step 3: Verify it builds and runs**

Run: `npm run tauri dev`
Expected: window opens. Then confirm the DB file exists at
`C:\Users\csequ\AppData\Roaming\com.applybot.app\applybot.db` (or the identifier set in `tauri.conf.json`). Close the window.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: initialize sqlite database on app startup"
```

---

### Task 8: Tauri command surface

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs` (declare `mod commands;`, register handlers)

**Interfaces:**
- Consumes: `state::AppState`, all `db::*` stores.
- Produces these Tauri commands (callable from the frontend via `invoke`):
  - `get_onboarding_status() -> bool`
  - `get_profile() -> Profile`
  - `save_profile(profile: Profile) -> ()`
  - `list_jobs() -> Vec<Job>`
  - `list_applications() -> Vec<Application>`
  - `list_pending() -> Vec<PendingAction>`
  - `resolve_pending(id: i64) -> ()`
  - `dashboard_counts() -> DashboardCounts`
  - `DashboardCounts { found: i64, awaiting_approval: i64, submitted: i64, pending: i64 }` — Serialize.

  Note: `has_linkedin_credentials` is hardcoded to `false` here and replaced by a real keychain check in Plan 3.

- [ ] **Step 1: Write the commands module**

Create `src-tauri/src/commands.rs`:

```rust
use serde::Serialize;
use tauri::State;

use crate::db::{applications, jobs, pending, profile};
use crate::state::AppState;

type CmdResult<T> = Result<T, String>;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

#[derive(Debug, Serialize)]
pub struct DashboardCounts {
    pub found: i64,
    pub awaiting_approval: i64,
    pub submitted: i64,
    pub pending: i64,
}

#[tauri::command]
pub fn get_onboarding_status(state: State<AppState>) -> CmdResult<bool> {
    let conn = state.db.lock().map_err(err)?;
    // Keychain wired in Plan 3; treat credentials as absent for now.
    profile::is_onboarding_complete(&conn, false).map_err(err)
}

#[tauri::command]
pub fn get_profile(state: State<AppState>) -> CmdResult<profile::Profile> {
    let conn = state.db.lock().map_err(err)?;
    profile::get(&conn).map_err(err)
}

#[tauri::command]
pub fn save_profile(state: State<AppState>, profile: profile::Profile) -> CmdResult<()> {
    let conn = state.db.lock().map_err(err)?;
    profile::upsert(&conn, &profile).map_err(err)
}

#[tauri::command]
pub fn list_jobs(state: State<AppState>) -> CmdResult<Vec<jobs::Job>> {
    let conn = state.db.lock().map_err(err)?;
    jobs::list(&conn).map_err(err)
}

#[tauri::command]
pub fn list_applications(state: State<AppState>) -> CmdResult<Vec<applications::Application>> {
    let conn = state.db.lock().map_err(err)?;
    applications::list(&conn).map_err(err)
}

#[tauri::command]
pub fn list_pending(state: State<AppState>) -> CmdResult<Vec<pending::PendingAction>> {
    let conn = state.db.lock().map_err(err)?;
    pending::list_open(&conn).map_err(err)
}

#[tauri::command]
pub fn resolve_pending(state: State<AppState>, id: i64) -> CmdResult<()> {
    let conn = state.db.lock().map_err(err)?;
    pending::resolve(&conn, id).map_err(err)
}

#[tauri::command]
pub fn dashboard_counts(state: State<AppState>) -> CmdResult<DashboardCounts> {
    let conn = state.db.lock().map_err(err)?;
    let found: i64 = conn.query_row("SELECT COUNT(*) FROM jobs", [], |r| r.get(0)).map_err(err)?;
    let awaiting_approval: i64 = conn
        .query_row("SELECT COUNT(*) FROM applications WHERE status = 'awaiting_approval'", [], |r| r.get(0))
        .map_err(err)?;
    let submitted: i64 = conn
        .query_row("SELECT COUNT(*) FROM applications WHERE status = 'submitted'", [], |r| r.get(0))
        .map_err(err)?;
    let pending_count = pending::count_open(&conn).map_err(err)?;
    Ok(DashboardCounts { found, awaiting_approval, submitted, pending: pending_count })
}
```

- [ ] **Step 2: Register the handlers**

In `src-tauri/src/lib.rs` declare `mod commands;` and replace the empty `generate_handler!` with:

```rust
.invoke_handler(tauri::generate_handler![
    commands::get_onboarding_status,
    commands::get_profile,
    commands::save_profile,
    commands::list_jobs,
    commands::list_applications,
    commands::list_pending,
    commands::resolve_pending,
    commands::dashboard_counts,
])
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo build`
Expected: compiles with no errors.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: tauri command surface for profile, jobs, applications, pending"
```

---

### Task 9: Frontend — typed command client and app shell with onboarding gate

**Files:**
- Create: `src/lib/api.ts` (typed wrappers around `invoke`)
- Create: `src/types.ts` (TS mirrors of the Rust types)
- Create: `src/App.tsx` (replace scaffold default) — shell, gate, navigation
- Create: `src/screens/Dashboard.tsx`, `src/screens/Jobs.tsx`, `src/screens/Pending.tsx`, `src/screens/Profile.tsx` (stubs, pt-BR)
- Create: `src/screens/Onboarding.tsx` (stub, pt-BR)
- Create: `src/App.css` (minimal layout)

**Interfaces:**
- Consumes: the Task 8 Tauri commands.
- Produces: a running UI that, on launch, calls `get_onboarding_status()` and routes to Onboarding when false, else to the four-screen shell.

- [ ] **Step 1: Define the TS types**

Create `src/types.ts`:

```ts
export interface Profile {
  full_name: string;
  email: string;
  phone: string;
  location: string;
  cv_text: string;
  criteria_json: string;
}

export interface Job {
  id: number;
  title: string;
  company: string;
  url: string;
  source: string;
  status: string;
  match_summary: string | null;
  discovered_at: string;
}

export interface Application {
  id: number;
  job_id: number;
  folder_path: string | null;
  cv_path: string | null;
  cover_letter_path: string | null;
  status: string;
  submitted_at: string | null;
}

export interface PendingAction {
  id: number;
  job_id: number | null;
  category: string;
  description: string;
  resolved: boolean;
  created_at: string;
}

export interface DashboardCounts {
  found: number;
  awaiting_approval: number;
  submitted: number;
  pending: number;
}
```

- [ ] **Step 2: Write the typed API client**

Create `src/lib/api.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type {
  Profile, Job, Application, PendingAction, DashboardCounts,
} from "../types";

export const api = {
  getOnboardingStatus: () => invoke<boolean>("get_onboarding_status"),
  getProfile: () => invoke<Profile>("get_profile"),
  saveProfile: (profile: Profile) => invoke<void>("save_profile", { profile }),
  listJobs: () => invoke<Job[]>("list_jobs"),
  listApplications: () => invoke<Application[]>("list_applications"),
  listPending: () => invoke<PendingAction[]>("list_pending"),
  resolvePending: (id: number) => invoke<void>("resolve_pending", { id }),
  dashboardCounts: () => invoke<DashboardCounts>("dashboard_counts"),
};
```

- [ ] **Step 3: Write the screen stubs (pt-BR copy)**

Create each file with a minimal placeholder. Example `src/screens/Dashboard.tsx`:

```tsx
export default function Dashboard() {
  return (
    <section>
      <h1>Painel</h1>
      <p>Estado da busca e contadores aparecem aqui.</p>
    </section>
  );
}
```

Create `src/screens/Jobs.tsx` (`<h1>Vagas</h1>` / "Vagas encontradas e aprovação de candidaturas."),
`src/screens/Pending.tsx` (`<h1>Pendências</h1>` / "Bloqueios que precisam de você."),
`src/screens/Profile.tsx` (`<h1>Perfil</h1>` / "Seus dados, currículo e critérios de busca."),
`src/screens/Onboarding.tsx` (`<h1>Configuração inicial</h1>` / "Vamos configurar seu perfil antes de começar."), each following the same structure.

- [ ] **Step 4: Write the shell with the onboarding gate**

Create `src/App.tsx`:

```tsx
import { useEffect, useState } from "react";
import { api } from "./lib/api";
import Onboarding from "./screens/Onboarding";
import Dashboard from "./screens/Dashboard";
import Jobs from "./screens/Jobs";
import Pending from "./screens/Pending";
import Profile from "./screens/Profile";
import "./App.css";

type Screen = "dashboard" | "jobs" | "pending" | "profile";

const NAV: { key: Screen; label: string }[] = [
  { key: "dashboard", label: "Painel" },
  { key: "jobs", label: "Vagas" },
  { key: "pending", label: "Pendências" },
  { key: "profile", label: "Perfil" },
];

export default function App() {
  const [onboarded, setOnboarded] = useState<boolean | null>(null);
  const [screen, setScreen] = useState<Screen>("dashboard");

  useEffect(() => {
    api.getOnboardingStatus().then(setOnboarded).catch(() => setOnboarded(false));
  }, []);

  if (onboarded === null) return <div className="loading">Carregando…</div>;
  if (!onboarded) return <Onboarding onDone={() => setOnboarded(true)} />;

  return (
    <div className="app">
      <nav className="sidebar">
        {NAV.map((n) => (
          <button
            key={n.key}
            className={screen === n.key ? "active" : ""}
            onClick={() => setScreen(n.key)}
          >
            {n.label}
          </button>
        ))}
      </nav>
      <main className="content">
        {screen === "dashboard" && <Dashboard />}
        {screen === "jobs" && <Jobs />}
        {screen === "pending" && <Pending />}
        {screen === "profile" && <Profile />}
      </main>
    </div>
  );
}
```

Update `src/screens/Onboarding.tsx` to accept `{ onDone }: { onDone: () => void }` and render a temporary button `<button onClick={onDone}>Concluir (provisório)</button>` so the gate is testable until Plan 2 builds the real wizard.

- [ ] **Step 5: Minimal styles**

Create `src/App.css`:

```css
* { box-sizing: border-box; }
body { margin: 0; font-family: system-ui, sans-serif; }
.app { display: flex; min-height: 100vh; }
.sidebar { width: 180px; background: #1a1a1a; display: flex; flex-direction: column; padding: 12px; gap: 4px; }
.sidebar button { background: transparent; color: #ccc; border: none; text-align: left; padding: 10px 12px; border-radius: 6px; cursor: pointer; font-size: 14px; }
.sidebar button:hover { background: #2a2a2a; color: #fff; }
.sidebar button.active { background: #D97757; color: #fff; }
.content { flex: 1; padding: 24px; }
.loading { padding: 24px; }
```

Ensure `src/main.tsx` renders `<App />` (the scaffold may already do this; remove the default demo markup if present).

- [ ] **Step 6: Verify the gate works end to end**

Run: `npm run tauri dev`
Expected: because no profile exists, the app shows the **Onboarding** stub. Click "Concluir (provisório)" → the four-screen shell appears and the sidebar switches between Painel / Vagas / Pendências / Perfil. (After restart it returns to Onboarding, since the provisional button does not persist anything — that is expected until Plan 2.)

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: frontend shell with onboarding gate and four-screen navigation"
```

---

## Plan 1 Self-Review

- **Spec coverage:** Tauri+React+Rust stack (Task 1) ✓; SQLite as single source of truth with all five tables — jobs, applications, pending_actions, profile, sessions (Tasks 2–6) ✓; English code + pt-BR UI (throughout) ✓; onboarding gate blocking usage (Tasks 6, 9) ✓; Tauri command surface (Task 8) ✓; four-screen shell (Task 9) ✓. Deferred to later plans by design: resume parsing + real onboarding wizard (Plan 2), agent engine + prompt + credentials keychain (Plan 3), real screen content + approval flow (Plan 4). `sessions` table is created here but read/written in Plan 3.
- **Placeholder scan:** No TBD/TODO. Screen stubs are intentionally minimal but each ships real, compiling code. The provisional onboarding button is explicitly labeled as a Plan 2 hand-off, not a hidden placeholder.
- **Type consistency:** `Profile`, `Job`, `Application`, `PendingAction`, `DashboardCounts` field names match across Rust (Tasks 3–8) and TypeScript (Task 9). Command names match between `generate_handler!` (Task 8) and `api.ts` (Task 9). Status string literals match the Global Constraints enum.

---

## Hand-off to Plan 2

Plan 2 (Resume parsing & onboarding) builds on this foundation: the `ResumeParser` Rust module (PDF/DOCX → text), the real multi-step onboarding wizard replacing the stub (with step 3 pre-filled from CV analysis), and the Profile screen in form + Claude-assisted modes. It will replace the provisional `onDone` button with `save_profile` + a real completion check.
