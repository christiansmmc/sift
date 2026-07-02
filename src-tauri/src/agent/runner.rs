//! Agent runner: launches the Claude Code CLI headless (`claude -p`) with the
//! Claude-in-Chrome integration, streams its JSON output, extracts the agent's
//! text, parses our marker protocol from it, and persists results to the DB.
//!
//! Headless `-p` (print) mode is used deliberately: interactive `claude`
//! renders a full terminal UI (ANSI control codes) that carries no clean
//! marker lines. `-p --output-format stream-json` emits one JSON event per
//! line, including every assistant message across turns, from which we recover
//! the marker lines the agent prints.

use std::io::{BufRead, BufReader};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use super::protocol::parse_line;
use super::sink::{apply_event, EventOutcome};

/// Opt-in debug instrumentation: when the `SIFT_DEBUG` env var is set,
/// append a line to `sift-agent.log` in the temp dir so we can see exactly
/// what the agent process emits. Off by default — it would otherwise write the
/// prompt (which includes the CV) and job data to a plaintext temp file.
fn dbg_log(msg: &str) {
    if std::env::var_os("SIFT_DEBUG").is_none() {
        return;
    }
    use std::io::Write;
    let path = std::env::temp_dir().join("sift-agent.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{msg}");
    }
}

/// Static flags for the headless agent invocation. The prompt itself is piped
/// through stdin by `spawn_agent`. `stream-json` requires `--verbose`.
///
/// The isolation flags keep the run independent from the user's personal
/// Claude Code setup, which otherwise inflates startup and every turn:
/// `--strict-mcp-config` skips their MCP servers (the Chrome integration comes
/// from `--chrome`, not mcp-config, so it survives), `--setting-sources ""`
/// skips settings/hooks/CLAUDE.md, `--disable-slash-commands` skips skills,
/// `--tools "ToolSearch"` drops the heavy built-in tools (Bash/Edit/Write/…)
/// but keeps ToolSearch — the claude-in-chrome tools are *deferred* and only
/// ToolSearch can load them, so `--tools ""` would leave the agent with no way
/// to reach the browser at all. `--no-session-persistence` avoids writing the
/// CV-bearing session to disk.
pub fn agent_args() -> Vec<String> {
    vec![
        "--chrome".to_string(),
        "--dangerously-skip-permissions".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--verbose".to_string(),
        "--strict-mcp-config".to_string(),
        "--setting-sources".to_string(),
        "".to_string(),
        "--disable-slash-commands".to_string(),
        "--tools".to_string(),
        "ToolSearch".to_string(),
        "--no-session-persistence".to_string(),
    ]
}

/// Neutral working directory for spawning `claude`. The GUI process inherits an
/// arbitrary cwd (the project repo in dev, wherever the .exe sits in prod). If
/// `claude` runs from inside a directory tree that contains a `.claude/` folder
/// or other directory-scoped config, it loads that context/hooks into the run,
/// which pollutes the session and breaks tool invocation (the model starts
/// emitting tool calls as plain text). `temp_dir` has no such config.
pub fn agent_working_dir() -> std::path::PathBuf {
    std::env::temp_dir()
}

/// Pull the agent's plain-text lines out of a single `stream-json` event line.
/// Returns the individual text lines from an `assistant` message or the final
/// `result`, or an empty vec for any other event / non-JSON line.
pub fn extract_text_lines(json_line: &str) -> Vec<String> {
    let v: serde_json::Value = match serde_json::from_str(json_line.trim()) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    match v.get("type").and_then(|t| t.as_str()) {
        Some("assistant") => {
            if let Some(content) = v.pointer("/message/content").and_then(|c| c.as_array()) {
                for block in content {
                    if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                        if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                            out.extend(text.lines().map(|l| l.to_string()));
                        }
                    }
                }
            }
        }
        Some("result") => {
            if let Some(text) = v.get("result").and_then(|t| t.as_str()) {
                out.extend(text.lines().map(|l| l.to_string()));
            }
        }
        _ => {}
    }
    out
}

/// Parse one plain-text line → persist → emit. Returns the outcome if it was a
/// marker line (None for ordinary chatter). Pure seam: no process, no Tauri.
pub fn process_line_with(
    line: &str,
    db: &Mutex<Connection>,
    mut emit: impl FnMut(&str, String),
) -> Option<EventOutcome> {
    let event = parse_line(line)?;
    let outcome = {
        // Recover a poisoned guard rather than silently dropping every event.
        let conn = match db.lock() {
            Ok(c) => c,
            Err(poisoned) => poisoned.into_inner(),
        };
        apply_event(&conn, &event).ok()?
    };
    emit("agent://event", format!("{outcome:?}"));
    // Also surface the milestone in the activity feed. The feed is otherwise fed
    // only by voluntary `SIFT_STATUS` narration from the model, which is sparse
    // and inconsistent; these deterministic lifecycle lines make it reliable.
    if let Some(msg) = feed_message(&event, &outcome) {
        emit("agent://status", msg);
    }
    Some(outcome)
}

/// Human-readable activity-feed line for a persisted lifecycle event, or `None`
/// for outcomes that shouldn't surface in the feed.
fn feed_message(event: &super::protocol::AgentEvent, outcome: &EventOutcome) -> Option<String> {
    use super::protocol::AgentEvent as E;
    let msg = match (event, outcome) {
        (E::Job(j), EventOutcome::Queued) => {
            format!("Vaga adicionada para revisão: {} — {}", j.title, j.company)
        }
        (E::Job(j), EventOutcome::Recorded) => {
            format!("Vaga encontrada: {} — {}", j.title, j.company)
        }
        // A re-reported vacancy is not a fresh find — stay silent in the feed.
        (E::Job(_), EventOutcome::Duplicate) => return None,
        (E::Pending(p), EventOutcome::Pending) => {
            format!("Pendência registrada: {}", p.description)
        }
        (_, EventOutcome::LoginRequired) => "Login no LinkedIn necessário.".to_string(),
        (_, EventOutcome::Submitted) => "Candidatura enviada.".to_string(),
        (_, EventOutcome::Done) => "Busca concluída.".to_string(),
        _ => return None,
    };
    Some(msg)
}

/// True when this outcome means the run is finished.
pub fn is_terminal(outcome: &EventOutcome) -> bool {
    matches!(outcome, EventOutcome::Done | EventOutcome::LoginRequired)
}

pub struct AgentHandle {
    stop: Arc<AtomicBool>,
    child: Arc<Mutex<std::process::Child>>,
}

impl AgentHandle {
    pub fn is_running(&self) -> bool {
        !self.stop.load(Ordering::SeqCst)
    }
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Ok(mut c) = self.child.lock() {
            let _ = c.kill();
        }
    }
}

impl Drop for AgentHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Spawn the headless agent with `prompt` and start streaming its output into the DB.
fn spawn_agent(
    db: Arc<Mutex<Connection>>,
    app: tauri::AppHandle,
    prompt: String,
) -> Result<AgentHandle, String> {
    let mut cmd = crate::claude_cli::command();
    // `-p` with no prompt argument: the prompt is piped through stdin below.
    // As an argv value it would hit the ~32 KB command-line limit on Windows
    // once the CV and answer bank grow, and it would show up in process lists.
    cmd.arg("-p");
    for a in agent_args() {
        cmd.arg(a);
    }
    let model = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        crate::db::settings::get_or(&conn, "agent_model", "sonnet").map_err(|e| e.to_string())?
    };
    cmd.arg("--model").arg(&model);
    // Spawn from a neutral cwd so `claude` does not pick up directory-scoped
    // context/hooks from the project tree. See agent_working_dir.
    cmd.current_dir(agent_working_dir());
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Falha ao iniciar o agente (claude): {e}"))?;

    dbg_log(&format!(
        "=== START: claude -p (prompt {} bytes) args={:?} ===",
        prompt.len(),
        agent_args()
    ));
    if std::env::var_os("SIFT_DEBUG").is_some() {
        let _ = std::fs::write(std::env::temp_dir().join("sift-prompt.txt"), &prompt);
    }

    let stdout = child.stdout.take().ok_or("no child stdout")?;
    let stderr = child.stderr.take().ok_or("no child stderr")?;
    let mut stdin = child.stdin.take().ok_or("no child stdin")?;

    // Feed the prompt and close the pipe (EOF marks the end of the prompt).
    // A dedicated thread avoids blocking here if the prompt exceeds the pipe buffer.
    std::thread::spawn(move || {
        use std::io::Write;
        let _ = stdin.write_all(prompt.as_bytes());
    });

    let stop = Arc::new(AtomicBool::new(false));
    let child = Arc::new(Mutex::new(child));

    // stderr → debug log (helps diagnose CLI/connection errors)
    std::thread::spawn(move || {
        let buf = BufReader::new(stderr);
        for line in buf.lines().map_while(Result::ok) {
            dbg_log(&format!("STDERR: {line}"));
        }
    });

    // stdout → stream-json events → marker lines → DB
    let stop_t = stop.clone();
    let child_t = child.clone();
    std::thread::spawn(move || {
        dbg_log("=== READER THREAD STARTED ===");
        let buf = BufReader::new(stdout);
        'outer: for json_line in buf.lines() {
            if stop_t.load(Ordering::SeqCst) {
                break;
            }
            let json_line = match json_line {
                Ok(l) => l,
                Err(e) => {
                    dbg_log(&format!("READ ERROR: {e}"));
                    break;
                }
            };
            dbg_log(&format!("RAW: {json_line}"));
            for text_line in extract_text_lines(&json_line) {
                dbg_log(&format!("TEXT: {text_line}"));
                if let Some(status) = super::protocol::parse_status(&text_line) {
                    use tauri::Emitter;
                    let _ = app.clone().emit("agent://status", status);
                    continue;
                }
                let app2 = app.clone();
                let outcome = process_line_with(&text_line, &db, |ev, payload| {
                    use tauri::Emitter;
                    let _ = app2.emit(ev, payload);
                });
                if let Some(o) = outcome {
                    if is_terminal(&o) {
                        stop_t.store(true, Ordering::SeqCst);
                        if let Ok(mut c) = child_t.lock() {
                            let _ = c.kill();
                        }
                        break 'outer;
                    }
                }
            }
        }
        dbg_log("=== READER LOOP EXITED ===");
        // Always reap the child so a stream-close exit never leaks a process.
        if let Ok(mut c) = child_t.lock() {
            let _ = c.kill();
            let _ = c.wait();
        }
        stop_t.store(true, Ordering::SeqCst);
        {
            use tauri::Emitter;
            let _ = app.emit("agent://event", "Done".to_string());
        }
    });

    Ok(AgentHandle { stop, child })
}

/// Build the search prompt and spawn the agent.
pub fn start(
    db: Arc<Mutex<Connection>>,
    app: tauri::AppHandle,
    profile: crate::db::profile::Profile,
    mode: String,
    batch_size: u32,
) -> Result<AgentHandle, String> {
    let answers = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        crate::db::answers::list(&conn).map_err(|e| e.to_string())?
    };
    let (style, custom) = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        (
            crate::db::settings::get_or(&conn, "cover_letter_style", "balanced").map_err(|e| e.to_string())?,
            crate::db::settings::get(&conn, "cover_letter_custom").map_err(|e| e.to_string())?.unwrap_or_default(),
        )
    };
    let cover_letter = crate::agent::prompt::cover_letter_instruction(&style, &custom);
    let prompt = crate::agent::prompt::build_system_prompt(&profile, &answers, &cover_letter, &mode, batch_size);
    spawn_agent(db, app, prompt)
}

/// Build the submission prompt and spawn the agent.
pub fn start_submit(
    db: Arc<Mutex<Connection>>,
    app: tauri::AppHandle,
    items: Vec<crate::db::applications::SubmitItem>,
) -> Result<AgentHandle, String> {
    let follow_company = {
        let conn = db.lock().map_err(|e| e.to_string())?;
        crate::db::settings::get_or(&conn, "follow_company", "false").map_err(|e| e.to_string())? == "true"
    };
    let prompt = crate::agent::prompt::build_submit_prompt(&items, follow_company);
    spawn_agent(db, app, prompt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{applications, open_in_memory};
    use std::sync::Mutex;

    #[test]
    fn process_line_queues_job_and_reports_outcome() {
        let db = Mutex::new(open_in_memory());
        let line = r#"SIFT_JOB {"title":"Dev","company":"Acme","url":"https://linkedin.com/jobs/1","match_summary":"ok","cover_letter":"Hi","answers":[]}"#;
        let mut emitted = Vec::new();
        let outcome = process_line_with(line, &db, |ev, p| emitted.push((ev.to_string(), p)));
        assert_eq!(outcome, Some(EventOutcome::Queued));
        assert_eq!(applications::list(&db.lock().unwrap()).unwrap().len(), 1);
        // Emits both the counts-refresh event and a human-readable feed line.
        assert_eq!(emitted.len(), 2);
        assert_eq!(emitted[0].0, "agent://event");
        assert_eq!(emitted[1].0, "agent://status");
        assert!(emitted[1].1.contains("Dev") && emitted[1].1.contains("Acme"));
    }

    #[test]
    fn process_line_feeds_terminal_milestone() {
        let db = Mutex::new(open_in_memory());
        let mut emitted = Vec::new();
        let outcome =
            process_line_with("SIFT_DONE", &db, |ev, p| emitted.push((ev.to_string(), p)));
        assert_eq!(outcome, Some(EventOutcome::Done));
        // A status feed line is emitted even when the model prints no SIFT_STATUS.
        assert!(emitted.iter().any(|(ev, p)| ev == "agent://status" && p == "Busca concluída."));
    }

    #[test]
    fn process_line_ignores_chatter() {
        let db = Mutex::new(open_in_memory());
        let outcome = process_line_with("just thinking out loud", &db, |_e, _p| {});
        assert_eq!(outcome, None);
    }

    #[test]
    fn agent_working_dir_is_neutral_not_project() {
        // The agent must be spawned from a neutral directory. Running `claude`
        // from inside the project tree makes it discover directory-scoped
        // context/hooks (the repo has a `.claude/`), which pollutes the session
        // and breaks tool invocation. temp_dir is the proven-neutral location.
        let d = agent_working_dir();
        assert!(d.is_absolute());
        assert_eq!(d, std::env::temp_dir());
    }

    #[test]
    fn agent_args_isolate_run_from_user_config() {
        let args = agent_args();
        let has = |f: &str| args.iter().any(|a| a == f);
        assert!(has("--chrome"));
        assert!(has("--dangerously-skip-permissions"));
        // Headless runs must not inherit the user's own Claude Code setup:
        // no personal MCP servers, no hooks/settings, no skills, no session files.
        assert!(has("--strict-mcp-config"));
        assert!(has("--disable-slash-commands"));
        assert!(has("--no-session-persistence"));
        let pos = args.iter().position(|a| a == "--setting-sources").expect("--setting-sources present");
        assert_eq!(args[pos + 1], "", "--setting-sources takes an empty value");
        // ToolSearch must stay enabled: the claude-in-chrome tools are deferred
        // and are only loaded by ToolSearch. `--tools ""` would strip it and the
        // browser tools could never be activated.
        let pos = args.iter().position(|a| a == "--tools").expect("--tools present");
        assert_eq!(args[pos + 1], "ToolSearch", "--tools keeps ToolSearch so deferred Chrome tools can load");
    }

    #[test]
    fn extract_text_lines_pulls_assistant_text() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"line one\nSIFT_DONE"}]}}"#;
        let lines = extract_text_lines(line);
        assert_eq!(lines, vec!["line one".to_string(), "SIFT_DONE".to_string()]);
    }

    #[test]
    fn extract_text_lines_handles_result_and_ignores_other() {
        let result = r#"{"type":"result","subtype":"success","result":"SIFT_DONE"}"#;
        assert_eq!(extract_text_lines(result), vec!["SIFT_DONE".to_string()]);
        assert!(extract_text_lines(r#"{"type":"system","subtype":"init"}"#).is_empty());
        assert!(extract_text_lines("not json at all").is_empty());
    }
}
