//! Stdout marker protocol the agent uses to report results to the app.
//! The agent prints one marker per line; everything else is ignored chatter.

use serde::{Deserialize, Serialize};

pub const JOB: &str = "SIFT_JOB";
pub const PENDING: &str = "SIFT_PENDING";
pub const LOGIN_REQUIRED: &str = "SIFT_LOGIN_REQUIRED";
pub const DONE: &str = "SIFT_DONE";
pub const STATUS: &str = "SIFT_STATUS";
pub const SUBMITTED: &str = "SIFT_SUBMITTED";

/// Parse a `SIFT_STATUS` line and return the status text, or `None` if the
/// line is not a status marker or the text after stripping the prefix is empty.
pub fn parse_status(line: &str) -> Option<String> {
    let rest = line.trim().strip_prefix(STATUS)?;
    let text = rest.trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Answer {
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct JobReport {
    pub title: String,
    pub company: String,
    pub url: String,
    #[serde(default)]
    pub match_summary: String,
    #[serde(default)]
    pub cover_letter: String,
    #[serde(default)]
    pub answers: Vec<Answer>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PendingReport {
    pub category: String,
    pub description: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub questions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentEvent {
    Job(JobReport),
    Pending(PendingReport),
    LoginRequired,
    Done,
    Submitted(i64),
}

/// Parse a single line of agent stdout into an event, or None if it is not a marker.
pub fn parse_line(line: &str) -> Option<AgentEvent> {
    let line = line.trim();
    if line == LOGIN_REQUIRED {
        return Some(AgentEvent::LoginRequired);
    }
    if line == DONE {
        return Some(AgentEvent::Done);
    }
    if let Some(rest) = line.strip_prefix(SUBMITTED) {
        return rest.trim().parse::<i64>().ok().map(AgentEvent::Submitted);
    }
    if let Some(rest) = line.strip_prefix(JOB) {
        return serde_json::from_str::<JobReport>(rest.trim()).ok().map(AgentEvent::Job);
    }
    if let Some(rest) = line.strip_prefix(PENDING) {
        return serde_json::from_str::<PendingReport>(rest.trim()).ok().map(AgentEvent::Pending);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_job_marker() {
        let line = r#"SIFT_JOB {"title":"Backend Engineer","company":"Acme","url":"https://linkedin.com/jobs/1","match_summary":"3/4 must-haves","cover_letter":"Dear...","answers":[{"question":"Years of Rust?","answer":"8"}]}"#;
        match parse_line(line).unwrap() {
            AgentEvent::Job(j) => {
                assert_eq!(j.title, "Backend Engineer");
                assert_eq!(j.answers.len(), 1);
                assert_eq!(j.answers[0].answer, "8");
            }
            _ => panic!("expected Job"),
        }
    }

    #[test]
    fn parses_pending_and_signals() {
        assert_eq!(parse_line("SIFT_LOGIN_REQUIRED"), Some(AgentEvent::LoginRequired));
        assert_eq!(parse_line("  SIFT_DONE  "), Some(AgentEvent::Done));
        let p = parse_line(r#"SIFT_PENDING {"category":"external_application","description":"redirects to company site","url":"https://acme.com/apply"}"#).unwrap();
        match p {
            AgentEvent::Pending(pr) => {
                assert_eq!(pr.category, "external_application");
                assert_eq!(pr.url.as_deref(), Some("https://acme.com/apply"));
                assert!(pr.questions.is_empty());
            }
            _ => panic!("expected Pending"),
        }
        // questions field is deserialized when present
        let pq = parse_line(r#"SIFT_PENDING {"category":"missing_answer","description":"English level?","questions":["English level?","Visa?"]}"#).unwrap();
        match pq {
            AgentEvent::Pending(pr) => {
                assert_eq!(pr.questions, vec!["English level?", "Visa?"]);
            }
            _ => panic!("expected Pending"),
        }
    }

    #[test]
    fn ignores_non_markers_and_bad_json() {
        assert_eq!(parse_line("I am now searching LinkedIn..."), None);
        assert_eq!(parse_line("SIFT_JOB {not json}"), None);
        assert_eq!(parse_line(""), None);
        // STATUS lines must NOT be treated as AgentEvents by parse_line
        assert_eq!(parse_line("SIFT_STATUS Buscando vagas no LinkedIn..."), None);
    }

    #[test]
    fn parses_submitted_marker() {
        assert_eq!(parse_line("SIFT_SUBMITTED 7"), Some(AgentEvent::Submitted(7)));
        assert_eq!(parse_line("SIFT_SUBMITTED notanumber"), None);
    }

    #[test]
    fn parses_status_text() {
        assert_eq!(
            parse_status("SIFT_STATUS Buscando vagas no LinkedIn..."),
            Some("Buscando vagas no LinkedIn...".to_string())
        );
        assert_eq!(
            parse_status("  SIFT_STATUS   Com espaços extras  "),
            Some("Com espaços extras".to_string())
        );
        // Empty after prefix → None
        assert_eq!(parse_status("SIFT_STATUS"), None);
        assert_eq!(parse_status("SIFT_STATUS   "), None);
        // Non-status line → None
        assert_eq!(parse_status("SIFT_JOB {...}"), None);
        assert_eq!(parse_status(""), None);
    }
}
