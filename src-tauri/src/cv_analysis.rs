//! One-shot CV analysis via the `claude -p` CLI. Degrades to an empty
//! Criteria on any failure so the UI can fall back to manual entry.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Criteria {
    pub role: String,
    pub seniority: String,
    pub work_model: String,
    pub locations: Vec<String>,
    pub salary_min: Option<i64>,
    pub red_lines: Vec<String>,
}

pub fn build_prompt(cv_text: &str) -> String {
    format!(
        "You are analyzing a job candidate's CV to infer the kind of role they should search for. \
Return ONLY a JSON object, no prose, no markdown fences, with EXACTLY these keys: \
role (string, the target job title), seniority (string: junior/mid/senior/lead or \"\"), \
work_model (string: one of remote/hybrid/onsite or \"\"), locations (array of strings), \
salary_min (integer or null), red_lines (array of strings the candidate should avoid). \
Infer conservatively from the CV; use \"\" / [] / null when unknown. \
CV:\n---\n{cv_text}\n---"
    )
}

/// Extract the first top-level JSON object from arbitrary CLI stdout.
pub fn parse_response(stdout: &str) -> Criteria {
    let (start, end) = match (stdout.find('{'), stdout.rfind('}')) {
        (Some(s), Some(e)) if e > s => (s, e),
        _ => return Criteria::default(),
    };
    serde_json::from_str::<Criteria>(&stdout[start..=end]).unwrap_or_default()
}

pub fn analyze(cv_text: &str) -> Criteria {
    if cv_text.trim().is_empty() {
        return Criteria::default();
    }
    let prompt = build_prompt(cv_text);
    let output = std::process::Command::new("claude")
        .arg("-p")
        .arg(&prompt)
        .output();
    match output {
        Ok(o) if o.status.success() => parse_response(&String::from_utf8_lossy(&o.stdout)),
        _ => Criteria::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_prompt_includes_cv_and_demands_json() {
        let p = build_prompt("10 years of Go");
        assert!(p.contains("10 years of Go"));
        assert!(p.contains("ONLY a JSON object"));
    }

    #[test]
    fn parse_response_extracts_json_from_noisy_output() {
        let out = "Here is the result:\n{\"role\":\"Backend Engineer\",\"seniority\":\"senior\",\"work_model\":\"remote\",\"locations\":[\"Brazil\"],\"salary_min\":12000,\"red_lines\":[]}\nDone.";
        let c = parse_response(out);
        assert_eq!(c.role, "Backend Engineer");
        assert_eq!(c.seniority, "senior");
        assert_eq!(c.work_model, "remote");
        assert_eq!(c.locations, vec!["Brazil".to_string()]);
        assert_eq!(c.salary_min, Some(12000));
    }

    #[test]
    fn parse_response_returns_default_on_garbage() {
        assert_eq!(parse_response("no json here"), Criteria::default());
        assert_eq!(parse_response("{not valid json}"), Criteria::default());
    }
}
