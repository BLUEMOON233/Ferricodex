mod codex;

use codex::{
    CodexHomeStatus, CodexSearchQuery, CodexSearchResponse, CodexThread, CodexTranscript,
    WorkspaceMetadata,
};
use std::path::PathBuf;
use tauri_plugin_opener::OpenerExt;

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

#[tauri::command]
fn get_codex_transcript(path: String) -> Result<CodexTranscript, String> {
    codex::read_transcript(path).map_err(|error| error.to_string())
}

#[tauri::command]
fn search_codex_history(query: CodexSearchQuery) -> Result<CodexSearchResponse, String> {
    codex::search_history(query).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_workspace_metadata(path: String) -> Result<WorkspaceMetadata, String> {
    codex::workspace_metadata(path).map_err(|error| error.to_string())
}

#[tauri::command]
fn set_thread_archive_state(thread_id: String, archived: bool) -> Result<(), String> {
    codex::set_thread_archive_state(thread_id, archived).map_err(|error| error.to_string())
}

#[tauri::command]
fn move_thread_to_trash(thread_id: String) -> Result<(), String> {
    codex::move_thread_to_trash(thread_id).map_err(|error| error.to_string())
}

#[tauri::command]
fn move_threads_to_trash(thread_ids: Vec<String>) -> Result<(), String> {
    codex::move_threads_to_trash(thread_ids).map_err(|error| error.to_string())
}

#[tauri::command]
fn move_generated_workspace_session_to_trash(
    thread_id: String,
    save_workspace_copy: bool,
) -> Result<(), String> {
    codex::move_generated_workspace_session_to_trash(thread_id, save_workspace_copy)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn open_local_path(app: tauri::AppHandle, path: String) -> Result<(), String> {
    let directory = normalize_openable_directory(&path)?;
    let opener_path = strip_unc_prefix(directory);
    let display_path = opener_path.display().to_string();

    app.opener()
        .open_path(opener_path.to_string_lossy().into_owned(), None::<&str>)
        .map_err(|error| format!("Unable to open {display_path}: {error}"))
}

fn strip_unc_prefix(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        if let Some(value) = path.to_str() {
            if let Some(rest) = value.strip_prefix(r"\\?\UNC\") {
                return PathBuf::from(format!(r"\\{rest}"));
            }
            if let Some(rest) = value.strip_prefix(r"\\?\") {
                return PathBuf::from(rest);
            }
        }
    }
    path
}

fn normalize_openable_directory(path: &str) -> Result<PathBuf, String> {
    let trimmed_path = path.trim();
    if trimmed_path.is_empty() {
        return Err("Path is unavailable".to_string());
    }

    let requested_path = expand_tilde(trimmed_path);
    let directory = requested_path
        .canonicalize()
        .map_err(|error| format!("Unable to resolve {}: {error}", requested_path.display()))?;

    let home = user_home_dir()?;
    let canonical_home = home.canonicalize().map_err(|error| {
        format!(
            "Unable to resolve home directory {}: {error}",
            home.display()
        )
    })?;

    if !directory.starts_with(&canonical_home) {
        return Err(format!(
            "Not allowed to open path outside current user home: {}",
            directory.display()
        ));
    }

    if !directory.is_dir() {
        return Err(format!("Path is not a directory: {}", directory.display()));
    }

    Ok(directory)
}

fn user_home_dir() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
        .ok_or_else(|| "Could not determine the current user's home directory".to_string())
}

fn expand_tilde(value: &str) -> PathBuf {
    if value == "~" {
        return user_home_dir().unwrap_or_else(|_| PathBuf::from(value));
    }

    if let Some(rest) = value.strip_prefix("~/") {
        if let Ok(home) = user_home_dir() {
            return home.join(rest);
        }
    }

    PathBuf::from(value)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_app_summary,
            get_codex_home_status,
            list_codex_threads,
            get_codex_transcript,
            search_codex_history,
            get_workspace_metadata,
            set_thread_archive_state,
            move_thread_to_trash,
            move_threads_to_trash,
            move_generated_workspace_session_to_trash,
            open_local_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
