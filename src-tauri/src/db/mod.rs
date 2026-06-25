use rusqlite::Connection;
use std::path::Path;

const SCHEMA: &str = include_str!("schema.sql");

fn configure(conn: &Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "foreign_keys", true)?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(())
}

pub fn apply_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)
}

/// Add columns introduced after the initial schema. Idempotent: ignores
/// "duplicate column" errors so it is safe to run on every open.
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    for stmt in [
        "ALTER TABLE applications ADD COLUMN cover_letter TEXT",
        "ALTER TABLE applications ADD COLUMN answers_json TEXT",
        "ALTER TABLE profile ADD COLUMN screening_json TEXT NOT NULL DEFAULT '{}'",
        "ALTER TABLE pending_actions ADD COLUMN questions_json TEXT NOT NULL DEFAULT '[]'",
    ] {
        match conn.execute(stmt, []) {
            Ok(_) => {}
            Err(rusqlite::Error::SqliteFailure(_, Some(msg))) if msg.contains("duplicate column name") => {}
            Err(e) => return Err(e),
        }
    }
    // Create answers table if it doesn't exist yet (CREATE IF NOT EXISTS is idempotent).
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS answers (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            question     TEXT NOT NULL UNIQUE,
            answer       TEXT NOT NULL,
            updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

pub fn open_at(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    configure(&conn)?;
    apply_schema(&conn)?;
    migrate(&conn)?;
    Ok(conn)
}

pub mod jobs;
pub mod applications;
pub mod pending;
pub mod profile;
pub mod answers;
pub mod settings;

#[cfg(test)]
pub fn open_in_memory() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    configure(&conn).expect("configure connection");
    apply_schema(&conn).expect("apply schema");
    migrate(&conn).expect("migrate");
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
                 AND name IN ('jobs','applications','pending_actions','profile','answers')",
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

    #[test]
    fn migration_adds_generated_columns_idempotently() {
        let conn = open_in_memory();
        // second run must not error
        migrate(&conn).expect("idempotent migrate");
        let cols: Vec<String> = {
            let mut stmt = conn.prepare("PRAGMA table_info(applications)").unwrap();
            let rows = stmt.query_map([], |r| r.get::<_, String>(1)).unwrap();
            rows.map(|r| r.unwrap()).collect()
        };
        assert!(cols.contains(&"cover_letter".to_string()));
        assert!(cols.contains(&"answers_json".to_string()));
        // profile gets screening_json
        let profile_cols: Vec<String> = {
            let mut stmt = conn.prepare("PRAGMA table_info(profile)").unwrap();
            let rows = stmt.query_map([], |r| r.get::<_, String>(1)).unwrap();
            rows.map(|r| r.unwrap()).collect()
        };
        assert!(profile_cols.contains(&"screening_json".to_string()));
        // answers table exists
        let answers_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM answers", [], |r| r.get(0))
            .unwrap();
        assert_eq!(answers_count, 0);
    }

    #[test]
    fn foreign_keys_are_enforced() {
        let conn = open_in_memory();
        // applications.job_id REFERENCES jobs(id); 999 does not exist.
        let result = conn.execute(
            "INSERT INTO applications (job_id, status) VALUES (999, 'awaiting_approval')",
            [],
        );
        assert!(result.is_err(), "insert with bogus job_id must fail when FKs are enforced");
    }
}
