use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::db::{applications, jobs, pending, profile};
use crate::state::AppState;

type CmdResult<T> = Result<T, String>;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

#[derive(Debug, Serialize)]
pub struct DashboardCounts {
    pub found: i64,
    pub awaiting_approval: i64,
    pub submitted: i64,
    pub pending: i64,
}

#[tauri::command]
pub fn get_onboarding_status(state: State<AppState>) -> CmdResult<bool> {
    let conn = state.db.lock().map_err(err)?;
    profile::is_onboarding_complete(&conn).map_err(err)
}

#[tauri::command]
pub fn get_profile(state: State<AppState>) -> CmdResult<profile::Profile> {
    let conn = state.db.lock().map_err(err)?;
    profile::get(&conn).map_err(err)
}

#[tauri::command]
pub fn save_profile(state: State<AppState>, profile: profile::Profile) -> CmdResult<()> {
    let conn = state.db.lock().map_err(err)?;
    profile::upsert(&conn, &profile).map_err(err)
}

#[tauri::command]
pub fn list_jobs(state: State<AppState>) -> CmdResult<Vec<jobs::Job>> {
    let conn = state.db.lock().map_err(err)?;
    jobs::list(&conn).map_err(err)
}

#[tauri::command]
pub fn list_applications(state: State<AppState>) -> CmdResult<Vec<applications::Application>> {
    let conn = state.db.lock().map_err(err)?;
    applications::list(&conn).map_err(err)
}

#[tauri::command]
pub fn list_pending(state: State<AppState>) -> CmdResult<Vec<pending::PendingAction>> {
    let conn = state.db.lock().map_err(err)?;
    pending::list_open(&conn).map_err(err)
}

#[tauri::command]
pub fn resolve_pending(state: State<AppState>, id: i64) -> CmdResult<()> {
    let conn = state.db.lock().map_err(err)?;
    pending::resolve(&conn, id).map_err(err)
}

#[tauri::command]
pub fn dashboard_counts(state: State<AppState>) -> CmdResult<DashboardCounts> {
    let conn = state.db.lock().map_err(err)?;
    let found: i64 = conn.query_row("SELECT COUNT(*) FROM jobs", [], |r| r.get(0)).map_err(err)?;
    let awaiting_approval: i64 = conn
        .query_row("SELECT COUNT(*) FROM applications WHERE status = 'awaiting_approval'", [], |r| r.get(0))
        .map_err(err)?;
    let submitted: i64 = conn
        .query_row("SELECT COUNT(*) FROM applications WHERE status = 'submitted'", [], |r| r.get(0))
        .map_err(err)?;
    let pending_count = pending::count_open(&conn).map_err(err)?;
    Ok(DashboardCounts { found, awaiting_approval, submitted, pending: pending_count })
}

// Both commands shell out / read files, which can take seconds. They run on a
// blocking worker thread (spawn_blocking) so the UI thread stays responsive and
// the frontend's "Analisando…" / "Lendo…" spinners keep animating.

#[tauri::command]
pub async fn parse_resume(path: String) -> CmdResult<String> {
    tauri::async_runtime::spawn_blocking(move || crate::resume::extract_from_path(&path))
        .await
        .map_err(err)?
}

#[tauri::command]
pub async fn analyze_cv(cv_text: String) -> CmdResult<crate::cv_analysis::CvAnalysis> {
    tauri::async_runtime::spawn_blocking(move || crate::cv_analysis::analyze(&cv_text))
        .await
        .map_err(err)
}

#[tauri::command]
pub fn start_search_batch(
    state: State<AppState>,
    app: tauri::AppHandle,
    batch_size: Option<u32>,
) -> CmdResult<()> {
    {
        let conn = state.db.lock().map_err(err)?;
        if !profile::is_onboarding_complete(&conn).map_err(err)? {
            return Err("Complete a configuração antes de iniciar a busca.".into());
        }
    }
    let mut slot = state.agent.lock().map_err(err)?;
    if slot.as_ref().map(|h| h.is_running()).unwrap_or(false) {
        return Err("O agente já está em execução.".into());
    }
    let profile = {
        let conn = state.db.lock().map_err(err)?;
        profile::get(&conn).map_err(err)?
    };
    let handle = crate::agent::runner::start(
        Arc::clone(&state.db),
        app,
        profile,
        batch_size.unwrap_or(10),
    )?;
    *slot = Some(handle);
    Ok(())
}

#[tauri::command]
pub fn stop_agent(state: State<AppState>) -> CmdResult<()> {
    if let Some(h) = state.agent.lock().map_err(err)?.as_ref() {
        h.stop();
    }
    Ok(())
}

#[tauri::command]
pub fn agent_running(state: State<AppState>) -> CmdResult<bool> {
    Ok(state
        .agent
        .lock()
        .map_err(err)?
        .as_ref()
        .map(|h| h.is_running())
        .unwrap_or(false))
}
