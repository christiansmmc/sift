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
    pub questions: Vec<String>,
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

/// Like `create` but also stores the list of unanswered questions as JSON.
pub fn create_with_questions(
    conn: &Connection,
    job_id: Option<i64>,
    category: &str,
    description: &str,
    questions: &[String],
) -> rusqlite::Result<i64> {
    let questions_json = serde_json::to_string(questions).unwrap_or_else(|_| "[]".into());
    conn.execute(
        "INSERT INTO pending_actions (job_id, category, description, questions_json) \
         VALUES (?1, ?2, ?3, ?4)",
        (job_id, category, description, &questions_json),
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_open(conn: &Connection) -> rusqlite::Result<Vec<PendingAction>> {
    let mut stmt = conn.prepare(
        "SELECT id, job_id, category, description, resolved, created_at, questions_json \
         FROM pending_actions WHERE resolved = 0 ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        let questions_json: Option<String> = r.get(6)?;
        let questions: Vec<String> = questions_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();
        Ok(PendingAction {
            id: r.get(0)?,
            job_id: r.get(1)?,
            category: r.get(2)?,
            description: r.get(3)?,
            resolved: r.get::<_, i64>(4)? != 0,
            created_at: r.get(5)?,
            questions,
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

    #[test]
    fn create_with_questions_roundtrips() {
        let conn = open_in_memory();
        let qs = vec!["English level?".to_string(), "Visa?".to_string()];
        create_with_questions(&conn, None, "missing_answer", "unanswered", &qs).unwrap();
        let open = list_open(&conn).unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].questions, qs);
    }

    #[test]
    fn create_without_questions_has_empty_vec() {
        let conn = open_in_memory();
        create(&conn, None, "captcha", "captcha hit").unwrap();
        let open = list_open(&conn).unwrap();
        assert!(open[0].questions.is_empty());
    }
}
