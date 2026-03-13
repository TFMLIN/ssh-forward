mod commands;
mod ssh;
mod types;

use std::sync::Arc;
use commands::mod_commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(AppState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::mod_commands::test_connection,
            commands::mod_commands::start_forward_cmd,
            commands::mod_commands::stop_forward_cmd,
            commands::mod_commands::get_forward_status,
            commands::mod_commands::get_all_statuses,
            commands::mod_commands::import_ssh_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
