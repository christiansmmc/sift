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

/// Normalise a job URL so that tracking params, fragments, trailing slashes
/// and casing differences all collapse to the same canonical form.
/// Rules (no external crates — plain string ops):
///   1. Trim whitespace
///   2. Drop everything from the first `#` (fragment)
///   3. Drop everything from the first `?` (query string)
///   4. Strip one trailing `/`
///   5. Lowercase the whole result
pub fn normalize_url(url: &str) -> String {
    let s = url.trim();
    let s = if let Some(pos) = s.find('#') { &s[..pos] } else { s };
    let s = if let Some(pos) = s.find('?') { &s[..pos] } else { s };
    let s = s.strip_suffix('/').unwrap_or(s);
    s.to_lowercase()
}

/// Inserts the job if its normalized URL is new. Returns the row id and whether
/// a new row was actually created (`false` = the URL was already known, i.e. a
/// re-report of a vacancy we had already recorded).
pub fn insert_returning_is_new(conn: &Connection, job: &NewJob) -> rusqlite::Result<(i64, bool)> {
    let url = normalize_url(&job.url);
    let changed = conn.execute(
        "INSERT INTO jobs (title, company, url, source) VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT(url) DO NOTHING",
        (&job.title, &job.company, &url, &job.source),
    )?;
    let id = conn.query_row("SELECT id FROM jobs WHERE url = ?1", [&url], |r| r.get(0))?;
    Ok((id, changed > 0))
}

pub fn insert(conn: &Connection, job: &NewJob) -> rusqlite::Result<i64> {
    Ok(insert_returning_is_new(conn, job)?.0)
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

/// Jobs that have no application row yet (Scan-mode discoveries).
pub fn without_application(conn: &Connection) -> rusqlite::Result<Vec<Job>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, company, url, source, status, match_summary, discovered_at \
         FROM jobs WHERE id NOT IN (SELECT job_id FROM applications) ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Job {
            id: r.get(0)?, title: r.get(1)?, company: r.get(2)?, url: r.get(3)?,
            source: r.get(4)?, status: r.get(5)?, match_summary: r.get(6)?, discovered_at: r.get(7)?,
        })
    })?;
    rows.collect()
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
    fn normalize_url_strips_query_fragment_trailing_slash_and_lowercases() {
        // full example from spec
        assert_eq!(
            normalize_url("https://www.LinkedIn.com/jobs/view/123/?utm=x#sec"),
            "https://www.linkedin.com/jobs/view/123"
        );
        // idempotent
        assert_eq!(
            normalize_url("https://www.linkedin.com/jobs/view/123"),
            "https://www.linkedin.com/jobs/view/123"
        );
        // query only
        assert_eq!(normalize_url("https://example.com/path?foo=bar"), "https://example.com/path");
        // fragment only
        assert_eq!(normalize_url("https://example.com/path#frag"), "https://example.com/path");
        // trailing slash only
        assert_eq!(normalize_url("https://example.com/path/"), "https://example.com/path");
        // whitespace trimming
        assert_eq!(normalize_url("  https://example.com/path  "), "https://example.com/path");
        // uppercase hostname
        assert_eq!(normalize_url("HTTPS://EXAMPLE.COM/Path"), "https://example.com/path");
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

    #[test]
    fn without_application_excludes_jobs_that_have_one() {
        let conn = crate::db::open_in_memory();
        let a = insert(&conn, &NewJob { title:"A".into(), company:"X".into(), url:"u1".into(), source:"linkedin".into() }).unwrap();
        let _b = insert(&conn, &NewJob { title:"B".into(), company:"Y".into(), url:"u2".into(), source:"linkedin".into() }).unwrap();
        crate::db::applications::create_with_content(&conn, a, "cl", "[]").unwrap();
        let found = without_application(&conn).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].url, "u2");
    }
}
