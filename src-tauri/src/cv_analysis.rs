//! One-shot CV analysis via the `claude -p` CLI. Extracts personal data and
//! infers search criteria. Degrades to an empty result on any failure so the
//! UI can fall back to manual entry.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PersonalData {
    pub full_name: String,
    pub email: String,
    pub phone: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Criteria {
    pub role: String,
    pub seniority: String,
    pub work_model: String,
    pub locations: Vec<String>,
    pub salary_min: Option<i64>,
    pub red_lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CvAnalysis {
    pub personal: PersonalData,
    pub criteria: Criteria,
}

pub fn build_prompt(cv_text: &str) -> String {
    format!(
        "You are analyzing a job candidate's CV. Extract their personal data and infer the kind of role they should search for. \
Return ONLY a JSON object, no prose, no markdown fences, with EXACTLY this shape: \
{{\"personal\": {{\"full_name\": string, \"email\": string, \"phone\": string, \"location\": string}}, \
\"criteria\": {{\"role\": string, \"seniority\": string (junior/mid/senior/lead or \"\"), \"work_model\": string (remote/hybrid/onsite or \"\"), \"locations\": array of strings, \"salary_min\": integer or null, \"red_lines\": array of strings}}}}. \
For personal: extract exactly what is in the CV; use \"\" when a field is absent. \
For criteria: infer conservatively; use \"\" / [] / null when unknown. \
CV:\n---\n{cv_text}\n---"
    )
}

/// Extract the first top-level JSON object from arbitrary CLI stdout.
///
/// Scans for a brace-balanced object (respecting string literals and escapes)
/// starting at each `{`, and returns the first one that deserializes. This is
/// more robust than a naive first-`{`-to-last-`}` slice, which breaks on any
/// stray brace in surrounding prose. Degrades to an empty result on failure.
pub fn parse_response(stdout: &str) -> CvAnalysis {
    let bytes = stdout.as_bytes();
    for (start, _) in stdout.match_indices('{') {
        if let Some(end) = balanced_object_end(bytes, start) {
            if let Ok(a) = serde_json::from_str::<CvAnalysis>(&stdout[start..=end]) {
                return a;
            }
        }
    }
    CvAnalysis::default()
}

/// Given `bytes[start] == b'{'`, return the index of the matching `}`, tracking
/// nesting depth while skipping braces inside JSON string literals. Returns
/// `None` if the object never closes.
fn balanced_object_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        if in_string {
            match b {
                _ if escaped => escaped = false,
                b'\\' => escaped = true,
                b'"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

pub fn analyze(cv_text: &str, model: &str) -> CvAnalysis {
    if cv_text.trim().is_empty() {
        return CvAnalysis::default();
    }
    let prompt = build_prompt(cv_text);
    let mut cmd = crate::claude_cli::command();
    cmd.arg("-p").arg(&prompt).arg("--model").arg(model);
    let output = cmd.output();
    match output {
        Ok(o) if o.status.success() => parse_response(&String::from_utf8_lossy(&o.stdout)),
        _ => CvAnalysis::default(),
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
    fn parse_response_extracts_personal_and_criteria() {
        let out = "Result:\n{\"personal\":{\"full_name\":\"Ada Lovelace\",\"email\":\"ada@x.com\",\"phone\":\"\",\"location\":\"London\"},\"criteria\":{\"role\":\"Backend Engineer\",\"seniority\":\"senior\",\"work_model\":\"remote\",\"locations\":[\"Brazil\"],\"salary_min\":12000,\"red_lines\":[]}}\nDone.";
        let a = parse_response(out);
        assert_eq!(a.personal.full_name, "Ada Lovelace");
        assert_eq!(a.personal.email, "ada@x.com");
        assert_eq!(a.personal.location, "London");
        assert_eq!(a.criteria.role, "Backend Engineer");
        assert_eq!(a.criteria.work_model, "remote");
        assert_eq!(a.criteria.salary_min, Some(12000));
    }

    #[test]
    fn parse_response_returns_default_on_garbage() {
        assert_eq!(parse_response("no json here"), CvAnalysis::default());
        assert_eq!(parse_response("{not valid json}"), CvAnalysis::default());
    }

    #[test]
    fn parse_response_ignores_trailing_braces_in_prose() {
        // A naive first-`{`-to-last-`}` slice would swallow the trailing `{ok}`
        // and fail to parse; brace-matching stops at the object's real close.
        let out = "Here you go: {\"personal\":{\"full_name\":\"Ada\",\"email\":\"\",\"phone\":\"\",\"location\":\"\"},\"criteria\":{\"role\":\"Dev\",\"seniority\":\"\",\"work_model\":\"\",\"locations\":[],\"salary_min\":null,\"red_lines\":[]}} — done {ok}";
        let a = parse_response(out);
        assert_eq!(a.personal.full_name, "Ada");
        assert_eq!(a.criteria.role, "Dev");
    }
}
