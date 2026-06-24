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

pub mod jobs;

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
