use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::db::{answers, applications, jobs, pending, profile};
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
pub async fn analyze_cv(state: State<'_, AppState>, cv_text: String) -> CmdResult<crate::cv_analysis::CvAnalysis> {
    let model = {
        let conn = state.db.lock().map_err(err)?;
        crate::db::settings::get_or(&conn, "agent_model", "sonnet").map_err(err)?
    };
    tauri::async_runtime::spawn_blocking(move || crate::cv_analysis::analyze(&cv_text, &model))
        .await
        .map_err(err)
}

#[tauri::command]
pub fn start_search_batch(
    state: State<AppState>,
    app: tauri::AppHandle,
    mode: Option<String>,
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
    let mode = match mode.as_deref() {
        Some("scan") | Some("revisar") => mode.unwrap(),
        _ => "revisar".to_string(),
    };
    let handle = crate::agent::runner::start(
        Arc::clone(&state.db),
        app,
        profile,
        mode,
        batch_size.unwrap_or(10),
    )?;
    // Stop and drop any finished-but-lingering handle before replacing it.
    if let Some(old) = slot.take() {
        old.stop();
    }
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

#[derive(Debug, Serialize)]
pub struct AnswerPair {
    pub question: String,
    pub answer: String,
}

#[tauri::command]
pub fn list_answers(state: State<AppState>) -> CmdResult<Vec<AnswerPair>> {
    let conn = state.db.lock().map_err(err)?;
    let pairs = answers::list(&conn).map_err(err)?;
    Ok(pairs
        .into_iter()
        .map(|(question, answer)| AnswerPair { question, answer })
        .collect())
}

#[tauri::command]
pub fn save_answer(state: State<AppState>, question: String, answer: String) -> CmdResult<()> {
    let conn = state.db.lock().map_err(err)?;
    answers::upsert(&conn, &question, &answer).map_err(err)
}

#[tauri::command]
pub fn list_review_queue(state: State<AppState>) -> CmdResult<Vec<applications::ReviewItem>> {
    let conn = state.db.lock().map_err(err)?;
    applications::review_queue(&conn).map_err(err)
}

#[tauri::command]
pub fn list_found_jobs(state: State<AppState>) -> CmdResult<Vec<jobs::Job>> {
    let conn = state.db.lock().map_err(err)?;
    jobs::without_application(&conn).map_err(err)
}

#[tauri::command]
pub fn approve_application(state: State<AppState>, id: i64) -> CmdResult<()> {
    let conn = state.db.lock().map_err(err)?;
    applications::set_status(&conn, id, "approved").map_err(err)
}

#[tauri::command]
pub fn reject_application(state: State<AppState>, id: i64) -> CmdResult<()> {
    let conn = state.db.lock().map_err(err)?;
    applications::set_status(&conn, id, "discarded").map_err(err)
}

#[tauri::command]
pub fn update_application_content(
    state: State<AppState>,
    id: i64,
    cover_letter: String,
    answers_json: String,
) -> CmdResult<()> {
    let conn = state.db.lock().map_err(err)?;
    applications::update_content(&conn, id, &cover_letter, &answers_json).map_err(err)
}

#[tauri::command]
pub fn count_approved(state: State<AppState>) -> CmdResult<i64> {
    let conn = state.db.lock().map_err(err)?;
    applications::count_approved(&conn).map_err(err)
}

#[tauri::command]
pub fn list_approved(state: State<AppState>) -> CmdResult<Vec<applications::ReviewItem>> {
    let conn = state.db.lock().map_err(err)?;
    applications::approved_queue(&conn).map_err(err)
}

#[tauri::command]
pub fn get_setting(state: State<AppState>, key: String) -> CmdResult<Option<String>> {
    let conn = state.db.lock().map_err(err)?;
    crate::db::settings::get(&conn, &key).map_err(err)
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: String) -> CmdResult<()> {
    let conn = state.db.lock().map_err(err)?;
    crate::db::settings::set(&conn, &key, &value).map_err(err)
}

#[tauri::command]
pub fn submit_approved(state: State<AppState>, app: tauri::AppHandle) -> CmdResult<()> {
    let mut slot = state.agent.lock().map_err(err)?;
    if slot.as_ref().map(|h| h.is_running()).unwrap_or(false) {
        return Err("O agente já está em execução.".into());
    }
    let items = {
        let conn = state.db.lock().map_err(err)?;
        applications::approved_for_submit(&conn).map_err(err)?
    };
    if items.is_empty() {
        return Err("Nenhuma candidatura aprovada para enviar.".into());
    }
    if let Some(old) = slot.take() { old.stop(); }
    let handle = crate::agent::runner::start_submit(state.db.clone(), app, items)?;
    *slot = Some(handle);
    Ok(())
}
