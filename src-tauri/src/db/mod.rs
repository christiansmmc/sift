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

pub fn open_at(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    configure(&conn)?;
    apply_schema(&conn)?;
    Ok(conn)
}

pub mod jobs;
pub mod applications;
pub mod pending;
pub mod profile;

#[cfg(test)]
pub fn open_in_memory() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    configure(&conn).expect("configure connection");
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
