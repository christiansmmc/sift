use rusqlite::{Connection, OptionalExtension};

pub fn get(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
        .optional()
}

pub fn set(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = ?2",
        (key, value),
    )?;
    Ok(())
}

pub fn get_or(conn: &Connection, key: &str, default: &str) -> rusqlite::Result<String> {
    Ok(get(conn, key)?.unwrap_or_else(|| default.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn set_get_roundtrip_and_default() {
        let conn = open_in_memory();
        assert_eq!(get(&conn, "k").unwrap(), None);
        assert_eq!(get_or(&conn, "k", "def").unwrap(), "def");
        set(&conn, "k", "v1").unwrap();
        set(&conn, "k", "v2").unwrap();
        assert_eq!(get(&conn, "k").unwrap().as_deref(), Some("v2"));
        assert_eq!(get_or(&conn, "k", "def").unwrap(), "v2");
    }
}
