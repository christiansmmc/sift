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
