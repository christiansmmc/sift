use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use tauri::Manager;

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub agent: Mutex<Option<crate::agent::runner::AgentHandle>>,
}

pub fn init(app: &tauri::App) -> AppState {
    let data_dir = app
        .path()
        .app_data_dir()
        .expect("resolve app data dir");
    std::fs::create_dir_all(&data_dir).expect("create app data dir");
    let conn = crate::db::open_at(&data_dir.join("sift.db")).expect("open sift.db");
    AppState { db: Arc::new(Mutex::new(conn)), agent: Mutex::new(None) }
}
