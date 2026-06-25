use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub id: i64,
    pub job_id: i64,
    pub status: String,
    pub submitted_at: Option<String>,
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

/// Mark an application submitted ONLY if it is currently `approved`. Guards
/// against a stray/hallucinated id from the agent flipping an unrelated row to
/// "submitted" without anything actually being sent. Returns true if a row changed.
pub fn mark_submitted(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    let n = conn.execute(
        "UPDATE applications SET status = 'submitted', submitted_at = datetime('now') \
         WHERE id = ?1 AND status = 'approved'",
        [id],
    )?;
    Ok(n > 0)
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Application>> {
    let mut stmt = conn.prepare(
        "SELECT id, job_id, status, submitted_at FROM applications ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Application {
            id: r.get(0)?,
            job_id: r.get(1)?,
            status: r.get(2)?,
            submitted_at: r.get(3)?,
        })
    })?;
    rows.collect()
}

/// Vacancy-level dedup guard: returns true if ANY application already exists
/// for a vacancy identified by normalized URL OR by title+company match,
/// in ANY status (including `discarded` so rejected vacancies don't resurface).
pub fn has_open_application_for_vacancy(
    conn: &Connection,
    title: &str,
    company: &str,
    normalized_url: &str,
) -> rusqlite::Result<bool> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM applications a JOIN jobs j ON a.job_id = j.id \
         WHERE a.status IN ('awaiting_approval','approved','submitted','discarded') \
         AND (j.url = ?1 OR (j.title = ?2 AND j.company = ?3))",
        (normalized_url, title, company),
        |r| r.get(0),
    )?;
    Ok(n > 0)
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReviewItem {
    pub application_id: i64,
    pub job_title: String,
    pub company: String,
    pub url: String,
    pub cover_letter: String,
    pub answers_json: String,
}

pub fn review_queue(conn: &Connection) -> rusqlite::Result<Vec<ReviewItem>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, j.title, j.company, j.url, \
                COALESCE(a.cover_letter,''), COALESCE(a.answers_json,'[]') \
         FROM applications a JOIN jobs j ON a.job_id = j.id \
         WHERE a.status = 'awaiting_approval' ORDER BY a.id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(ReviewItem {
            application_id: r.get(0)?,
            job_title: r.get(1)?,
            company: r.get(2)?,
            url: r.get(3)?,
            cover_letter: r.get(4)?,
            answers_json: r.get(5)?,
        })
    })?;
    rows.collect()
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SubmitItem {
    pub application_id: i64,
    pub url: String,
    pub cover_letter: String,
    pub answers_json: String,
}

pub fn approved_for_submit(conn: &Connection) -> rusqlite::Result<Vec<SubmitItem>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, j.url, COALESCE(a.cover_letter,''), COALESCE(a.answers_json,'[]') \
         FROM applications a JOIN jobs j ON a.job_id = j.id \
         WHERE a.status = 'approved' ORDER BY a.id ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(SubmitItem {
            application_id: r.get(0)?,
            url: r.get(1)?,
            cover_letter: r.get(2)?,
            answers_json: r.get(3)?,
        })
    })?;
    rows.collect()
}

/// Overwrite the generated content of an application (user edits before approval).
pub fn update_content(
    conn: &Connection,
    id: i64,
    cover_letter: &str,
    answers_json: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE applications SET cover_letter = ?2, answers_json = ?3 WHERE id = ?1",
        (id, cover_letter, answers_json),
    )?;
    Ok(())
}

pub fn count_approved(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM applications WHERE status='approved'", [], |r| r.get(0))
}

pub fn approved_queue(conn: &Connection) -> rusqlite::Result<Vec<ReviewItem>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, j.title, j.company, j.url, \
                COALESCE(a.cover_letter,''), COALESCE(a.answers_json,'[]') \
         FROM applications a JOIN jobs j ON a.job_id = j.id \
         WHERE a.status = 'approved' ORDER BY a.id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(ReviewItem {
            application_id: r.get(0)?,
            job_title: r.get(1)?,
            company: r.get(2)?,
            url: r.get(3)?,
            cover_letter: r.get(4)?,
            answers_json: r.get(5)?,
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
    fn approved_for_submit_lists_only_approved() {
        let conn = open_in_memory();
        let j = jobs::insert(&conn, &jobs::NewJob{title:"D".into(),company:"A".into(),url:"u1".into(),source:"linkedin".into()}).unwrap();
        let id = create_with_content(&conn, j, "cl", "[]").unwrap();
        assert_eq!(count_approved(&conn).unwrap(), 0); // awaiting_approval, not approved
        set_status(&conn, id, "approved").unwrap();
        let items = approved_for_submit(&conn).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].application_id, id);
        assert_eq!(items[0].url, "u1");
        assert_eq!(count_approved(&conn).unwrap(), 1);
    }

    #[test]
    fn create_defaults_to_awaiting_approval() {
        let conn = open_in_memory();
        let job_id = job(&conn);
        create_with_content(&conn, job_id, "cl", "[]").unwrap();
        let app = &list(&conn).unwrap()[0];
        assert_eq!(app.status, "awaiting_approval");
        assert!(app.submitted_at.is_none());
    }

    #[test]
    fn submitting_stamps_submitted_at() {
        let conn = open_in_memory();
        let job_id = job(&conn);
        let id = create_with_content(&conn, job_id, "cl", "[]").unwrap();
        set_status(&conn, id, "submitted").unwrap();
        let app = &list(&conn).unwrap()[0];
        assert_eq!(app.status, "submitted");
        assert!(app.submitted_at.is_some());
    }

    #[test]
    fn update_content_overwrites_letter_and_answers() {
        let conn = open_in_memory();
        let j = jobs::insert(&conn, &jobs::NewJob{title:"D".into(),company:"A".into(),url:"u1".into(),source:"linkedin".into()}).unwrap();
        let id = create_with_content(&conn, j, "old", "[]").unwrap();
        update_content(&conn, id, "new letter", r#"[{"question":"Q","answer":"A"}]"#).unwrap();
        let q = review_queue(&conn).unwrap();
        assert_eq!(q[0].cover_letter, "new letter");
        assert!(q[0].answers_json.contains("\"answer\":\"A\""));
    }

    #[test]
    fn review_queue_returns_awaiting_items_with_content() {
        let conn = open_in_memory();
        let job_id = jobs::insert(&conn, &jobs::NewJob {
            title: "Dev".into(), company: "Acme".into(),
            url: "https://linkedin.com/jobs/1".into(), source: "linkedin".into(),
        }).unwrap();
        create_with_content(&conn, job_id, "Dear Acme", r#"[{"question":"Q","answer":"A"}]"#).unwrap();
        let q = review_queue(&conn).unwrap();
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].company, "Acme");
        assert_eq!(q[0].cover_letter, "Dear Acme");
    }

    #[test]
    fn has_open_application_for_vacancy_finds_by_normalized_url() {
        let conn = open_in_memory();
        let job_id = jobs::insert(&conn, &NewJob {
            title: "Dev".into(), company: "Acme".into(),
            url: "https://linkedin.com/jobs/1".into(), source: "linkedin".into(),
        }).unwrap();
        create_with_content(&conn, job_id, "cl", "[]").unwrap();
        assert!(has_open_application_for_vacancy(
            &conn, "Dev", "Acme", "https://linkedin.com/jobs/1"
        ).unwrap());
        // different URL → no match via URL branch (title+company mismatch too)
        assert!(!has_open_application_for_vacancy(
            &conn, "Other", "Other", "https://linkedin.com/jobs/999"
        ).unwrap());
    }

    #[test]
    fn has_open_application_for_vacancy_finds_by_title_and_company() {
        let conn = open_in_memory();
        let job_id = jobs::insert(&conn, &NewJob {
            title: "Backend Dev".into(), company: "Techco".into(),
            url: "https://linkedin.com/jobs/1".into(), source: "linkedin".into(),
        }).unwrap();
        create_with_content(&conn, job_id, "cl", "[]").unwrap();
        // Different URL but same title+company → should find it
        assert!(has_open_application_for_vacancy(
            &conn, "Backend Dev", "Techco", "https://linkedin.com/jobs/different"
        ).unwrap());
    }

    #[test]
    fn has_open_application_for_vacancy_includes_discarded_status() {
        let conn = open_in_memory();
        let job_id = jobs::insert(&conn, &NewJob {
            title: "Dev".into(), company: "Acme".into(),
            url: "https://linkedin.com/jobs/1".into(), source: "linkedin".into(),
        }).unwrap();
        let app_id = create_with_content(&conn, job_id, "cl", "[]").unwrap();
        set_status(&conn, app_id, "discarded").unwrap();
        // discarded must count as "have seen this vacancy"
        assert!(has_open_application_for_vacancy(
            &conn, "Dev", "Acme", "https://linkedin.com/jobs/1"
        ).unwrap());
    }
}
