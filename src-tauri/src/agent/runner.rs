//! AgentRunner: spawn `claude --chrome` over a PTY, feed it the system prompt,
//! and process stdout lines into the DB via the marker protocol.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use super::protocol::parse_line;
use super::sink::{apply_event, EventOutcome};

/// Parse one stdout line → persist → emit. Returns the outcome if it was a
/// marker line (None for ordinary agent chatter). Pure seam: no PTY, no Tauri.
pub fn process_line_with(
    line: &str,
    db: &Mutex<Connection>,
    mut emit: impl FnMut(&str, String),
) -> Option<EventOutcome> {
    let event = parse_line(line)?;
    let outcome = {
        let conn = match db.lock() {
            Ok(c) => c,
            Err(poisoned) => poisoned.into_inner(),
        };
        apply_event(&conn, &event).ok()?
    };
    emit("agent://event", format!("{:?}", outcome));
    Some(outcome)
}

/// True when this outcome means the run is finished.
pub fn is_terminal(outcome: &EventOutcome) -> bool {
    matches!(outcome, EventOutcome::Done | EventOutcome::LoginRequired)
}

/// The CLI command + args to launch the agent. Isolated so a test could swap it.
pub fn agent_command() -> (String, Vec<String>) {
    (
        "claude".to_string(),
        vec![
            "--chrome".to_string(),
            "--dangerously-skip-permissions".to_string(),
        ],
    )
}

/// Handle to a running agent subprocess.
///
/// Fields beyond `stop` and `child`:
/// - `_master`: the PTY master side must remain alive for the duration of the run.
///   On Windows (ConPTY), closing the master terminates the child immediately, so we
///   keep it here rather than dropping it after `spawn_command`.
/// - `_writer`: the write end of the PTY master. `take_writer` moves it out of the
///   master, so we must store it separately to keep the underlying handle open.
pub struct AgentHandle {
    stop: Arc<AtomicBool>,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
    /// Keeps the PTY master alive so the child process is not killed by an early drop.
    _master: Box<dyn portable_pty::MasterPty + Send>,
    /// Keeps the PTY write-end open for the same reason.
    _writer: Box<dyn std::io::Write + Send>,
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

/// Spawn `claude --chrome` over a PTY, write the system prompt, and start the
/// reader thread. Returns an `AgentHandle` that the caller can use to stop the run.
pub fn start(
    db: Arc<Mutex<Connection>>,
    app: tauri::AppHandle,
    profile: crate::db::profile::Profile,
    batch_size: u32,
) -> Result<AgentHandle, String> {
    use portable_pty::{native_pty_system, CommandBuilder, PtySize};

    let pty = native_pty_system();
    let pair = pty
        .openpty(PtySize {
            rows: 40,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    // Build the command from the isolated function so tests can verify args.
    let (cmd_name, args) = agent_command();
    let mut cmd = CommandBuilder::new(cmd_name);
    for a in args {
        cmd.arg(a);
    }

    // Spawn into the PTY slave side.
    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| e.to_string())?;

    // Claim the master reader and writer BEFORE moving anything into the thread.
    let reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let mut writer = pair.master.take_writer().map_err(|e| e.to_string())?;

    // Write the system prompt so the agent starts immediately.
    let prompt = crate::agent::prompt::build_system_prompt(&profile, batch_size);
    {
        use std::io::Write;
        writer
            .write_all(prompt.as_bytes())
            .map_err(|e| e.to_string())?;
        writer.write_all(b"\n").map_err(|e| e.to_string())?;
        writer.flush().ok();
    }

    let stop = Arc::new(AtomicBool::new(false));
    let child = Arc::new(Mutex::new(child));

    // Spawn the reader thread. Clones of the Arcs are moved in.
    let stop_t = stop.clone();
    let child_t = child.clone();
    std::thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        let buf = BufReader::new(reader);
        for line in buf.lines() {
            if stop_t.load(Ordering::SeqCst) {
                break;
            }
            let Ok(line) = line else {
                break;
            };
            let app2 = app.clone();
            let outcome = process_line_with(&line, &db, |ev, payload| {
                use tauri::Emitter;
                let _ = app2.emit(ev, payload);
            });
            if let Some(o) = outcome {
                if is_terminal(&o) {
                    stop_t.store(true, Ordering::SeqCst);
                    if let Ok(mut c) = child_t.lock() {
                        let _ = c.kill();
                    }
                    break;
                }
            }
        }
        // Kill/reap the child unconditionally so a stream-close exit (no terminal
        // marker) never leaves a zombie or leaked `claude` process. Killing an
        // already-dead child is a no-op.
        if let Ok(mut c) = child_t.lock() {
            let _ = c.kill();
            let _ = c.wait();
        }
        // Mark finished when the process ends or the stream closes.
        stop_t.store(true, Ordering::SeqCst);
        {
            use tauri::Emitter;
            let _ = app.emit("agent://event", "Done".to_string());
        }
    });

    // Keep the PTY master and writer alive in the handle so ConPTY (Windows)
    // does not terminate the child when they go out of scope.
    Ok(AgentHandle {
        stop,
        child,
        _master: pair.master,
        _writer: writer,
    })
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
}
