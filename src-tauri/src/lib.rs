mod codex;

use codex::{CodexHomeStatus, CodexThread};

#[tauri::command]
fn get_app_summary() -> String {
    "Codex History Manager keeps Codex session reading isolated from app metadata.".to_string()
}

#[tauri::command]
fn get_codex_home_status() -> CodexHomeStatus {
    codex::home_status()
}

#[tauri::command]
fn list_codex_threads() -> Result<Vec<CodexThread>, String> {
    codex::list_threads().map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_app_summary,
            get_codex_home_status,
            list_codex_threads
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
