use rusqlite::Connection;

/// Insert or update a question→answer pair.
pub fn upsert(conn: &Connection, question: &str, answer: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO answers (question, answer, updated_at) VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(question) DO UPDATE SET answer=?2, updated_at=datetime('now')",
        (question, answer),
    )?;
    Ok(())
}

/// Return all stored question→answer pairs as (question, answer) tuples.
pub fn list(conn: &Connection) -> rusqlite::Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT question, answer FROM answers ORDER BY id")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn upsert_then_list_roundtrips() {
        let conn = open_in_memory();
        upsert(&conn, "Years of Rust?", "8").unwrap();
        upsert(&conn, "Location?", "Brazil").unwrap();
        let pairs = list(&conn).unwrap();
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], ("Years of Rust?".into(), "8".into()));
    }

    #[test]
    fn upsert_updates_existing_answer() {
        let conn = open_in_memory();
        upsert(&conn, "Years of Rust?", "8").unwrap();
        upsert(&conn, "Years of Rust?", "10").unwrap();
        let pairs = list(&conn).unwrap();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].1, "10");
    }
}
