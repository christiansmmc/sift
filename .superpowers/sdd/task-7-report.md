# Task 7 Report — App state and database initialization on startup

## Files Created

### `src-tauri/src/state.rs` (new)

```rust
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
```

## Files Modified

### `src-tauri/src/lib.rs` — final shape of the `run()` builder chain

```rust
mod db;
mod state;

use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_state = state::init(app);
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## cargo build result

Build succeeded with only pre-existing dead-code warnings (unused structs/functions in the data layer not yet wired to Tauri commands). No errors.

## Notes

- `use tauri::Manager;` was required in `lib.rs` (not just `state.rs`) because `app.manage()` is called in the `.setup()` closure inside `lib.rs`. The spec's `state.rs` already imports it for the `app.path()` call there, but `lib.rs` needed it too for `app.manage()`.
- The `open_at` dead-code warning from prior tasks is now gone (function is used via `state::init`).
