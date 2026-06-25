use rusqlite::Connection;

use super::protocol::AgentEvent;
use crate::db::{applications, jobs, pending};

#[derive(Debug, PartialEq)]
pub enum EventOutcome {
    Queued,
    Pending,
    LoginRequired,
    Done,
    Submitted,
}

pub fn apply_event(conn: &Connection, event: &AgentEvent) -> rusqlite::Result<EventOutcome> {
    match event {
        AgentEvent::Job(j) => {
            let job_id = jobs::insert(
                conn,
                &jobs::NewJob {
                    title: j.title.clone(),
                    company: j.company.clone(),
                    url: j.url.clone(),
                    source: "linkedin".into(),
                },
            )?;
            jobs::set_status(conn, job_id, "analyzed", Some(&j.match_summary))?;
            // Scan mode reports jobs with no cover letter → save the job only.
            // Revisar mode includes a cover letter → also queue an application.
            if !j.cover_letter.trim().is_empty()
                && !applications::has_open_application(conn, job_id)?
            {
                let answers_json = serde_json::to_string(&j.answers).unwrap_or_else(|_| "[]".into());
                applications::create_with_content(conn, job_id, &j.cover_letter, &answers_json)?;
            }
            Ok(EventOutcome::Queued)
        }
        AgentEvent::Pending(p) => {
            let desc = match &p.url {
                Some(u) => format!("{} ({})", p.description, u),
                None => p.description.clone(),
            };
            if p.questions.is_empty() {
                pending::create(conn, None, &p.category, &desc)?;
            } else {
                pending::create_with_questions(conn, None, &p.category, &desc, &p.questions)?;
            }
            Ok(EventOutcome::Pending)
        }
        AgentEvent::LoginRequired => {
            pending::create(
                conn,
                None,
                "login_required",
                "Você não está logado no LinkedIn. Faça login no Chrome e tente novamente.",
            )?;
            Ok(EventOutcome::LoginRequired)
        }
        AgentEvent::Done => Ok(EventOutcome::Done),
        AgentEvent::Submitted(id) => {
            applications::set_status(conn, *id, "submitted")?;
            Ok(EventOutcome::Submitted)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::protocol::{Answer, JobReport, PendingReport};
    use crate::db::{jobs, open_in_memory};

    #[test]
    fn submitted_event_marks_application_submitted() {
        let conn = open_in_memory();
        let job_id = jobs::insert(&conn, &jobs::NewJob {
            title:"D".into(), company:"A".into(), url:"https://linkedin.com/jobs/1".into(), source:"linkedin".into()
        }).unwrap();
        let app_id = applications::create_with_content(&conn, job_id, "cl", "[]").unwrap();
        apply_event(&conn, &AgentEvent::Submitted(app_id)).unwrap();
        let a = &applications::list(&conn).unwrap()[0];
        assert_eq!(a.status, "submitted");
        assert!(a.submitted_at.is_some());
    }

    #[test]
    fn job_event_queues_application_with_content() {
        let conn = open_in_memory();
        let ev = AgentEvent::Job(JobReport {
            title: "Backend Engineer".into(),
            company: "Acme".into(),
            url: "https://linkedin.com/jobs/1".into(),
            match_summary: "good".into(),
            cover_letter: "Dear Acme...".into(),
            answers: vec![Answer { question: "Rust years?".into(), answer: "8".into() }],
        });
        assert_eq!(apply_event(&conn, &ev).unwrap(), EventOutcome::Queued);
        let apps = applications::list(&conn).unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].status, "awaiting_approval");
        // verify content persisted
        let cl: String = conn.query_row("SELECT cover_letter FROM applications WHERE id=?1", [apps[0].id], |r| r.get(0)).unwrap();
        assert_eq!(cl, "Dear Acme...");
    }

    #[test]
    fn login_required_creates_pending() {
        let conn = open_in_memory();
        assert_eq!(apply_event(&conn, &AgentEvent::LoginRequired).unwrap(), EventOutcome::LoginRequired);
        let p = pending::list_open(&conn).unwrap();
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].category, "login_required");
    }

    #[test]
    fn duplicate_job_event_does_not_duplicate_application() {
        let conn = open_in_memory();
        let ev = AgentEvent::Job(JobReport {
            title: "Dev".into(), company: "Acme".into(),
            url: "https://linkedin.com/jobs/1".into(),
            match_summary: "ok".into(), cover_letter: "Hi".into(), answers: vec![],
        });
        apply_event(&conn, &ev).unwrap();
        apply_event(&conn, &ev).unwrap(); // same job reported again
        assert_eq!(applications::list(&conn).unwrap().len(), 1);
    }

    #[test]
    fn scan_job_without_cover_letter_saves_job_only() {
        let conn = open_in_memory();
        let ev = AgentEvent::Job(JobReport {
            title: "Dev".into(), company: "Acme".into(),
            url: "https://linkedin.com/jobs/9".into(),
            match_summary: "strong".into(),
            cover_letter: "".into(), answers: vec![],
        });
        apply_event(&conn, &ev).unwrap();
        assert_eq!(jobs::list(&conn).unwrap().len(), 1);
        assert_eq!(applications::list(&conn).unwrap().len(), 0);
    }

    #[test]
    fn pending_event_persists_with_url() {
        let conn = open_in_memory();
        let ev = AgentEvent::Pending(PendingReport {
            category: "external_application".into(),
            description: "redirects to site".into(),
            url: Some("https://acme.com/apply".into()),
            questions: vec![],
        });
        apply_event(&conn, &ev).unwrap();
        let p = pending::list_open(&conn).unwrap();
        assert!(p[0].description.contains("acme.com"));
        assert!(p[0].questions.is_empty());
    }

    #[test]
    fn pending_event_with_questions_stores_them() {
        let conn = open_in_memory();
        let ev = AgentEvent::Pending(PendingReport {
            category: "missing_answer".into(),
            description: "English level?".into(),
            url: None,
            questions: vec!["English level?".into(), "Visa status?".into()],
        });
        apply_event(&conn, &ev).unwrap();
        let p = pending::list_open(&conn).unwrap();
        assert_eq!(p[0].questions, vec!["English level?", "Visa status?"]);
    }
}
