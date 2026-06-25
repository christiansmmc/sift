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
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use super::protocol::parse_line;
use super::sink::{apply_event, EventOutcome};

/// Opt-in debug instrumentation: when the `APPLYBOT_DEBUG` env var is set,
/// append a line to `applybot-agent.log` in the temp dir so we can see exactly
/// what the agent process emits. Off by default — it would otherwise write the
/// prompt (which includes the CV) and job data to a plaintext temp file.
fn dbg_log(msg: &str) {
    if std::env::var_os("APPLYBOT_DEBUG").is_none() {
        return;
    }
    use std::io::Write;
    let path = std::env::temp_dir().join("applybot-agent.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{msg}");
    }
}

/// Static flags for the headless agent invocation. The dynamic `-p <prompt>`
/// is added by `start`. `stream-json` requires `--verbose`.
pub fn agent_args() -> Vec<String> {
    vec![
        "--chrome".to_string(),
        "--dangerously-skip-permissions".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--verbose".to_string(),
    ]
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
    Some(outcome)
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
    let mut cmd = Command::new("claude");
    cmd.arg("-p").arg(&prompt);
    for a in agent_args() {
        cmd.arg(a);
    }
    cmd.stdin(Stdio::null())
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
    if std::env::var_os("APPLYBOT_DEBUG").is_some() {
        let _ = std::fs::write(std::env::temp_dir().join("applybot-prompt.txt"), &prompt);
    }

    let stdout = child.stdout.take().ok_or("no child stdout")?;
    let stderr = child.stderr.take().ok_or("no child stderr")?;

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
    let prompt = crate::agent::prompt::build_system_prompt(&profile, &answers, &mode, batch_size);
    spawn_agent(db, app, prompt)
}

/// Build the submission prompt and spawn the agent.
pub fn start_submit(
    db: Arc<Mutex<Connection>>,
    app: tauri::AppHandle,
    items: Vec<crate::db::applications::SubmitItem>,
) -> Result<AgentHandle, String> {
    let prompt = crate::agent::prompt::build_submit_prompt(&items);
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
        let line = r#"APPLYBOT_JOB {"title":"Dev","company":"Acme","url":"https://linkedin.com/jobs/1","match_summary":"ok","cover_letter":"Hi","answers":[]}"#;
        let mut emitted = Vec::new();
        let outcome = process_line_with(line, &db, |ev, p| emitted.push((ev.to_string(), p)));
        assert_eq!(outcome, Some(EventOutcome::Queued));
        assert_eq!(applications::list(&db.lock().unwrap()).unwrap().len(), 1);
        assert_eq!(emitted.len(), 1);
        assert_eq!(emitted[0].0, "agent://event");
    }

    #[test]
    fn process_line_ignores_chatter() {
        let db = Mutex::new(open_in_memory());
        let outcome = process_line_with("just thinking out loud", &db, |_e, _p| {});
        assert_eq!(outcome, None);
    }

    #[test]
    fn extract_text_lines_pulls_assistant_text() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"line one\nAPPLYBOT_DONE"}]}}"#;
        let lines = extract_text_lines(line);
        assert_eq!(lines, vec!["line one".to_string(), "APPLYBOT_DONE".to_string()]);
    }

    #[test]
    fn extract_text_lines_handles_result_and_ignores_other() {
        let result = r#"{"type":"result","subtype":"success","result":"APPLYBOT_DONE"}"#;
        assert_eq!(extract_text_lines(result), vec!["APPLYBOT_DONE".to_string()]);
        assert!(extract_text_lines(r#"{"type":"system","subtype":"init"}"#).is_empty());
        assert!(extract_text_lines("not json at all").is_empty());
    }
}
