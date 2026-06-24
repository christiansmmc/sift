use std::sync::Mutex;
use rusqlite::Connection;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Connection>,
}

pub fn init(app: &tauri::App) -> AppState {
    let data_dir = app
        .path()
        .app_data_dir()
        .expect("resolve app data dir");
    std::fs::create_dir_all(&data_dir).expect("create app data dir");
    let conn = crate::db::open_at(&data_dir.join("applybot.db")).expect("open applybot.db");
    AppState { db: Mutex::new(conn) }
}
