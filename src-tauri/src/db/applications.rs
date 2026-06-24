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

/// Create an application already carrying generated content, awaiting approval.
pub fn create_with_content(
    conn: &Connection,
    job_id: i64,
    cover_letter: &str,
    answers_json: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO applications (job_id, status, cover_letter, answers_json) \
         VALUES (?1, 'awaiting_approval', ?2, ?3)",
        (job_id, cover_letter, answers_json),
    )?;
    Ok(conn.last_insert_rowid())
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
