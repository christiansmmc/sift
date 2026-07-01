mod agent;
mod claude_cli;
mod commands;
mod cv_analysis;
mod db;
mod resume;
mod state;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
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
            commands::parse_resume,
            commands::analyze_cv,
            commands::start_search_batch,
            commands::stop_agent,
            commands::agent_running,
            commands::list_answers,
            commands::save_answer,
            commands::list_review_queue,
            commands::list_found_jobs,
            commands::approve_application,
            commands::reject_application,
            commands::update_application_content,
            commands::count_approved,
            commands::list_approved,
            commands::submit_approved,
            commands::get_setting,
            commands::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
