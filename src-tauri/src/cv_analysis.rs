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
pub fn parse_response(stdout: &str) -> CvAnalysis {
    let (start, end) = match (stdout.find('{'), stdout.rfind('}')) {
        (Some(s), Some(e)) if e > s => (s, e),
        _ => return CvAnalysis::default(),
    };
    serde_json::from_str::<CvAnalysis>(&stdout[start..=end]).unwrap_or_default()
}

/// Flags for the one-shot CV analysis invocation (everything after `-p`).
/// This is a pure text-extraction call: it needs no tools, no MCP servers, no
/// hooks/skills from the user's own Claude Code setup, and no session file on
/// disk — skipping all of that cuts startup from many seconds to a few.
pub fn cv_args(model: &str) -> Vec<String> {
    vec![
        "--model".to_string(),
        model.to_string(),
        "--strict-mcp-config".to_string(),
        "--setting-sources".to_string(),
        "".to_string(),
        "--disable-slash-commands".to_string(),
        "--tools".to_string(),
        "".to_string(),
        "--no-session-persistence".to_string(),
    ]
}

pub fn analyze(cv_text: &str, model: &str) -> CvAnalysis {
    if cv_text.trim().is_empty() {
        return CvAnalysis::default();
    }
    let prompt = build_prompt(cv_text);
    let mut cmd = std::process::Command::new("claude");
    // `-p` with no prompt argument: the prompt (which embeds the whole CV) is
    // piped through stdin to stay clear of the ~32 KB argv limit on Windows.
    cmd.arg("-p");
    for a in cv_args(model) {
        cmd.arg(a);
    }
    // Spawn from a neutral cwd so `claude` does not load directory-scoped
    // context/hooks from the project tree. See agent::runner::agent_working_dir.
    cmd.current_dir(crate::agent::runner::agent_working_dir());
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    // Suppress the blank console window the `claude` shim would otherwise pop
    // on Windows when launched from the GUI .exe. See agent::runner::spawn_agent.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return CvAnalysis::default(),
    };
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        // The CLI reads all of stdin before producing output, so writing the
        // whole prompt here (then closing the pipe) cannot deadlock.
        let _ = stdin.write_all(prompt.as_bytes());
    }
    match child.wait_with_output() {
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
    fn cv_args_isolate_run_from_user_config() {
        let args = cv_args("haiku");
        let has = |f: &str| args.iter().any(|a| a == f);
        assert!(has("--strict-mcp-config"));
        assert!(has("--disable-slash-commands"));
        assert!(has("--no-session-persistence"));
        let pos = args.iter().position(|a| a == "--setting-sources").expect("--setting-sources present");
        assert_eq!(args[pos + 1], "");
        let pos = args.iter().position(|a| a == "--tools").expect("--tools present");
        assert_eq!(args[pos + 1], "");
        let pos = args.iter().position(|a| a == "--model").expect("--model present");
        assert_eq!(args[pos + 1], "haiku");
    }

    #[test]
    fn parse_response_returns_default_on_garbage() {
        assert_eq!(parse_response("no json here"), CvAnalysis::default());
        assert_eq!(parse_response("{not valid json}"), CvAnalysis::default());
    }
}
