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
    let has_creds = crate::credentials::has_linkedin();
    profile::is_onboarding_complete(&conn, has_creds).map_err(err)
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

#[tauri::command]
pub fn parse_resume(path: String) -> CmdResult<String> {
    crate::resume::extract_from_path(&path)
}

#[tauri::command]
pub fn save_linkedin_credentials(username: String, password: String) -> CmdResult<()> {
    crate::credentials::save_linkedin(&username, &password)
}

#[tauri::command]
pub fn has_linkedin_credentials() -> CmdResult<bool> {
    Ok(crate::credentials::has_linkedin())
}

#[tauri::command]
pub fn get_linkedin_username() -> CmdResult<Option<String>> {
    Ok(crate::credentials::current_username())
}

#[tauri::command]
pub fn analyze_cv(cv_text: String) -> CmdResult<crate::cv_analysis::CvAnalysis> {
    Ok(crate::cv_analysis::analyze(&cv_text))
}
