mod commands;
mod db;
mod state;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_state = state::init(app);
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_onboarding_status,
            commands::get_profile,
            commands::save_profile,
            commands::list_jobs,
            commands::list_applications,
            commands::list_pending,
            commands::resolve_pending,
            commands::dashboard_counts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
