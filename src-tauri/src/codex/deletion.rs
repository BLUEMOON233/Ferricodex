use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction};
use serde_json::Value;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use std::process;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::error::CodexError;
use super::home::{expand_tilde, resolve_codex_home};

struct ThreadTrashState {
    rollout_path: String,
}

struct SessionIndexCleanup {
    path: PathBuf,
    original_contents: String,
    cleaned_contents: String,
}

struct SessionIndexCriteria {
    thread_id: String,
    rollout_path: String,
    expanded_rollout_path: String,
    rollout_file_name: Option<String>,
}

struct ThreadWorkspaceState {
    cwd: String,
    workspace_thread_count: i64,
}

struct GeneratedWorkspacePath {
    path: PathBuf,
    date: String,
    folder_name: String,
}

const GENERATED_WORKSPACE_ROOT_ENV: &str = "CODEX_GENERATED_WORKSPACE_ROOT";
const SAVED_WORKSPACES_ROOT_ENV: &str = "FERRICODEX_SAVED_WORKSPACES_ROOT";

pub fn move_threads_to_trash(thread_ids: Vec<String>) -> Result<(), CodexError> {
    let mut normalized_ids = Vec::new();

    for thread_id in thread_ids {
        let thread_id = normalize_thread_id(&thread_id)?;
        if !normalized_ids.iter().any(|existing| existing == &thread_id) {
            normalized_ids.push(thread_id);
        }
    }

    if normalized_ids.is_empty() {
        return Err(CodexError::TrashOperation(
            "No Codex sessions were selected".to_string(),
        ));
    }

    for thread_id in normalized_ids {
        move_thread_to_trash(thread_id.clone()).map_err(|error| {
            CodexError::TrashOperation(format!(
                "Could not delete selected Codex session {thread_id}: {error}"
            ))
        })?;
    }

    Ok(())
}

pub fn move_generated_workspace_session_to_trash(
    thread_id: String,
    save_workspace_copy: bool,
) -> Result<(), CodexError> {
    let thread_id = normalize_thread_id(&thread_id)?;
    let workspace_state = read_thread_workspace_state(&thread_id)?;
    let workspace_path = expand_tilde(workspace_state.cwd.trim());
    let generated_workspace = generated_workspace_path(&workspace_path)?;
    let workspace_exists = ensure_optional_workspace_directory(&generated_workspace.path)?;

    if workspace_state.workspace_thread_count != 1 {
        return Err(CodexError::TrashOperation(format!(
            "Generated workspace {} is referenced by {} Codex sessions; deletion was stopped for safety",
            generated_workspace.path.display(),
            workspace_state.workspace_thread_count
        )));
    }

    let saved_copy = if save_workspace_copy && workspace_exists {
        Some(save_generated_workspace_copy(&generated_workspace)?)
    } else {
        None
    };

    move_thread_to_trash(thread_id)?;

    if workspace_exists {
        trash::delete(&generated_workspace.path).map_err(|error| {
            let saved_message = saved_copy
                .as_ref()
                .map(|path| format!(" A saved copy exists at {}.", path.display()))
                .unwrap_or_default();

            CodexError::TrashOperation(format!(
                "Session was deleted, but generated workspace folder {} could not be moved to Trash: {error}.{saved_message}",
                generated_workspace.path.display()
            ))
        })?;
    }

    Ok(())
}

pub fn move_thread_to_trash(thread_id: String) -> Result<(), CodexError> {
    let thread_id = normalize_thread_id(&thread_id)?;

    let home = resolve_codex_home()?;
    if !home.path.exists() {
        return Err(CodexError::HomeNotFound(home.path));
    }

    let state_db_path = home.path.join("state_5.sqlite");
    if !state_db_path.exists() {
        return Err(CodexError::StateDbNotFound(state_db_path));
    }

    let mut connection = Connection::open_with_flags(
        state_db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_URI,
    )?;
    connection.busy_timeout(Duration::from_secs(2))?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;

    let transaction = connection.transaction()?;
    let current_state = transaction
        .query_row(
            "SELECT rollout_path FROM threads WHERE id = ?1",
            [&thread_id],
            |row| {
                Ok(ThreadTrashState {
                    rollout_path: row.get(0)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| CodexError::ThreadNotFound(thread_id.clone()))?;

    let transcript = expand_rollout_path(&home.path, &current_state.rollout_path);
    let transcript_exists = ensure_trashable_transcript(&transcript, &home.path)?;
    let session_index_cleanup = plan_session_index_cleanup(
        &home.path,
        &thread_id,
        &current_state.rollout_path,
        &transcript,
    )?;

    clear_thread_spawn_edges(&transaction, &thread_id)?;
    clear_agent_job_assignments(&transaction, &thread_id)?;

    let deleted_count = transaction.execute("DELETE FROM threads WHERE id = ?1", [&thread_id])?;
    if deleted_count != 1 {
        return Err(CodexError::TrashOperation(format!(
            "Expected to delete one Codex thread row, deleted {deleted_count}"
        )));
    }

    ensure_no_known_database_references(&transaction, &thread_id)?;
    apply_session_index_cleanup(session_index_cleanup.as_ref())?;

    if transcript_exists {
        if let Err(error) = trash_existing_transcript_if_present(&transcript) {
            if let Err(restore_error) =
                restore_session_index_cleanup(session_index_cleanup.as_ref())
            {
                return Err(CodexError::TrashOperation(format!(
                    "{error}; also failed to restore session_index.jsonl: {restore_error}"
                )));
            }

            return Err(CodexError::TrashOperation(error));
        }
    }

    if let Err(error) = transaction.commit() {
        if let Err(restore_error) = restore_session_index_cleanup(session_index_cleanup.as_ref()) {
            return Err(CodexError::TrashOperation(format!(
                "Could not commit Codex database cleanup after trashing transcript: {error}; also failed to restore session_index.jsonl: {restore_error}"
            )));
        }

        return Err(CodexError::from(error));
    }

    Ok(())
}

fn normalize_thread_id(thread_id: &str) -> Result<String, CodexError> {
    let thread_id = thread_id.trim();

    if thread_id.is_empty() {
        return Err(CodexError::TrashOperation(
            "Codex session id is unavailable".to_string(),
        ));
    }

    Ok(thread_id.to_string())
}

fn read_thread_workspace_state(thread_id: &str) -> Result<ThreadWorkspaceState, CodexError> {
    let home = resolve_codex_home()?;
    if !home.path.exists() {
        return Err(CodexError::HomeNotFound(home.path));
    }

    let state_db_path = home.path.join("state_5.sqlite");
    if !state_db_path.exists() {
        return Err(CodexError::StateDbNotFound(state_db_path));
    }

    let connection = Connection::open_with_flags(
        state_db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )?;
    connection.busy_timeout(Duration::from_secs(2))?;

    let cwd: String = connection
        .query_row(
            "SELECT cwd FROM threads WHERE id = ?1",
            [thread_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| CodexError::ThreadNotFound(thread_id.to_string()))?;

    if cwd.trim().is_empty() {
        return Err(CodexError::TrashOperation(format!(
            "Codex session {thread_id} does not have a workspace path"
        )));
    }

    let selected_workspace_path = normalize_absolute_path(&expand_tilde(cwd.trim()))?;
    let mut statement = connection.prepare("SELECT cwd FROM threads")?;
    let workspace_paths = statement.query_map([], |row| row.get::<_, String>(0))?;
    let mut workspace_thread_count = 0;

    for workspace_path in workspace_paths {
        let workspace_path = workspace_path?;
        let workspace_path = workspace_path.trim();

        if workspace_path.is_empty() {
            continue;
        }

        let Ok(candidate_workspace_path) = normalize_absolute_path(&expand_tilde(workspace_path))
        else {
            continue;
        };

        if candidate_workspace_path == selected_workspace_path {
            workspace_thread_count += 1;
        }
    }

    Ok(ThreadWorkspaceState {
        cwd,
        workspace_thread_count,
    })
}

fn generated_workspace_path(path: &Path) -> Result<GeneratedWorkspacePath, CodexError> {
    let path = normalize_absolute_path(path)?;

    for root in generated_workspace_roots()? {
        let root = normalize_absolute_path(&root)?;
        let Ok(relative_path) = path.strip_prefix(&root) else {
            continue;
        };

        let Some(components) = normal_component_names(relative_path) else {
            continue;
        };

        if components.len() == 2 && is_date_folder(&components[0]) && !components[1].is_empty() {
            return Ok(GeneratedWorkspacePath {
                path: path.clone(),
                date: components[0].clone(),
                folder_name: components[1].clone(),
            });
        }
    }

    Err(CodexError::TrashOperation(format!(
        "Workspace {} is not a Codex-generated task folder. Expected {}/YYYY-MM-DD/<folder>",
        path.display(),
        default_generated_workspace_root()?.display()
    )))
}

fn generated_workspace_roots() -> Result<Vec<PathBuf>, CodexError> {
    let mut roots = Vec::new();

    if let Ok(value) = env::var(GENERATED_WORKSPACE_ROOT_ENV) {
        for root in env::split_paths(&value) {
            if !root.as_os_str().is_empty() {
                roots.push(expand_tilde(&root.to_string_lossy()));
            }
        }
    }

    roots.push(default_generated_workspace_root()?);
    Ok(roots)
}

fn default_generated_workspace_root() -> Result<PathBuf, CodexError> {
    Ok(user_home_dir()?.join("Documents").join("Codex"))
}

fn ensure_optional_workspace_directory(path: &Path) -> Result<bool, CodexError> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => Ok(true),
        Ok(_) => Err(CodexError::TrashOperation(format!(
            "Expected generated workspace to be a directory: {}",
            path.display()
        ))),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(CodexError::TrashOperation(format!(
            "Could not read generated workspace {}: {error}",
            path.display()
        ))),
    }
}

fn save_generated_workspace_copy(
    generated_workspace: &GeneratedWorkspacePath,
) -> Result<PathBuf, CodexError> {
    let root = saved_workspaces_root()?;
    let date_parent = root.join(&generated_workspace.date);

    fs::create_dir_all(&date_parent).map_err(|error| {
        CodexError::TrashOperation(format!(
            "Could not create saved workspace directory {}: {error}",
            date_parent.display()
        ))
    })?;

    let destination = unique_destination(&date_parent, &generated_workspace.folder_name)?;
    validate_copy_destination(&generated_workspace.path, &destination)?;

    if let Err(error) = copy_dir_all(&generated_workspace.path, &destination) {
        let _ = fs::remove_dir_all(&destination);
        return Err(error);
    }

    Ok(destination)
}

fn saved_workspaces_root() -> Result<PathBuf, CodexError> {
    if let Ok(value) = env::var(SAVED_WORKSPACES_ROOT_ENV) {
        let value = value.trim();

        if !value.is_empty() {
            return normalize_absolute_path(&expand_tilde(value));
        }
    }

    normalize_absolute_path(
        &user_home_dir()?
            .join("Documents")
            .join("Ferricodex Saved Workspaces"),
    )
}

fn unique_destination(parent: &Path, folder_name: &str) -> Result<PathBuf, CodexError> {
    let folder_name = folder_name.trim();

    if folder_name.is_empty() {
        return Err(CodexError::TrashOperation(
            "Saved workspace folder name is unavailable".to_string(),
        ));
    }

    let first_candidate = parent.join(folder_name);
    if !first_candidate.exists() {
        return Ok(first_candidate);
    }

    for index in 2..=9999 {
        let candidate = parent.join(format!("{folder_name} {index}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(CodexError::TrashOperation(format!(
        "Could not find a non-conflicting saved workspace name under {}",
        parent.display()
    )))
}

fn validate_copy_destination(source: &Path, destination: &Path) -> Result<(), CodexError> {
    let source = normalize_absolute_path(source)?;
    let destination = normalize_absolute_path(destination)?;

    if source == destination || destination.starts_with(&source) || source.starts_with(&destination)
    {
        return Err(CodexError::TrashOperation(format!(
            "Refusing to copy workspace {} to overlapping destination {}",
            source.display(),
            destination.display()
        )));
    }

    Ok(())
}

fn copy_dir_all(source: &Path, destination: &Path) -> Result<(), CodexError> {
    let metadata = fs::metadata(source).map_err(|error| {
        CodexError::TrashOperation(format!(
            "Could not read workspace before saving {}: {error}",
            source.display()
        ))
    })?;

    if !metadata.is_dir() {
        return Err(CodexError::TrashOperation(format!(
            "Expected generated workspace to be a directory before saving: {}",
            source.display()
        )));
    }

    fs::create_dir(destination).map_err(|error| {
        CodexError::TrashOperation(format!(
            "Could not create saved workspace copy {}: {error}",
            destination.display()
        ))
    })?;

    for entry in fs::read_dir(source).map_err(|error| {
        CodexError::TrashOperation(format!(
            "Could not read workspace directory {} while saving: {error}",
            source.display()
        ))
    })? {
        let entry = entry.map_err(|error| {
            CodexError::TrashOperation(format!(
                "Could not read workspace entry under {} while saving: {error}",
                source.display()
            ))
        })?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type().map_err(|error| {
            CodexError::TrashOperation(format!(
                "Could not read workspace entry type {} while saving: {error}",
                source_path.display()
            ))
        })?;

        if file_type.is_dir() {
            copy_dir_all(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &destination_path).map_err(|error| {
                CodexError::TrashOperation(format!(
                    "Could not copy workspace file {} to {}: {error}",
                    source_path.display(),
                    destination_path.display()
                ))
            })?;
        } else if file_type.is_symlink() {
            copy_symlink(&source_path, &destination_path)?;
        } else {
            return Err(CodexError::TrashOperation(format!(
                "Refusing to copy unsupported filesystem entry {}",
                source_path.display()
            )));
        }
    }

    Ok(())
}

#[cfg(unix)]
fn copy_symlink(source: &Path, destination: &Path) -> Result<(), CodexError> {
    let target = fs::read_link(source).map_err(|error| {
        CodexError::TrashOperation(format!(
            "Could not read symlink {} while saving workspace: {error}",
            source.display()
        ))
    })?;

    std::os::unix::fs::symlink(&target, destination).map_err(|error| {
        CodexError::TrashOperation(format!(
            "Could not copy workspace symlink {} to {}: {error}",
            source.display(),
            destination.display()
        ))
    })
}

#[cfg(windows)]
fn copy_symlink(source: &Path, destination: &Path) -> Result<(), CodexError> {
    let target = fs::read_link(source).map_err(|error| {
        CodexError::TrashOperation(format!(
            "Could not read symlink {} while saving workspace: {error}",
            source.display()
        ))
    })?;
    let target_metadata = fs::metadata(source).map_err(|error| {
        CodexError::TrashOperation(format!(
            "Could not inspect symlink target {} while saving workspace: {error}",
            source.display()
        ))
    })?;
    let result = if target_metadata.is_dir() {
        std::os::windows::fs::symlink_dir(&target, destination)
    } else {
        std::os::windows::fs::symlink_file(&target, destination)
    };

    result.map_err(|error| {
        CodexError::TrashOperation(format!(
            "Could not copy workspace symlink {} to {}: {error}",
            source.display(),
            destination.display()
        ))
    })
}

#[cfg(not(any(unix, windows)))]
fn copy_symlink(source: &Path, destination: &Path) -> Result<(), CodexError> {
    fs::copy(source, destination).map(|_| ()).map_err(|error| {
        CodexError::TrashOperation(format!(
            "Could not copy workspace symlink {} to {}: {error}",
            source.display(),
            destination.display()
        ))
    })
}

fn normalize_absolute_path(path: &Path) -> Result<PathBuf, CodexError> {
    if !path.is_absolute() {
        return Err(CodexError::TrashOperation(format!(
            "Workspace path must be absolute: {}",
            path.display()
        )));
    }

    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(CodexError::TrashOperation(format!(
                    "Workspace path must not contain parent-directory segments: {}",
                    path.display()
                )));
            }
        }
    }

    Ok(normalized)
}

fn normal_component_names(path: &Path) -> Option<Vec<String>> {
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            Component::Normal(value) => components.push(value.to_string_lossy().into_owned()),
            _ => return None,
        }
    }

    Some(components)
}

fn is_date_folder(value: &str) -> bool {
    let bytes = value.as_bytes();

    bytes.len() == 10
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(u8::is_ascii_digit)
}

fn user_home_dir() -> Result<PathBuf, CodexError> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .ok_or(CodexError::HomeDirUnavailable)
}

fn clear_thread_spawn_edges(
    transaction: &Transaction<'_>,
    thread_id: &str,
) -> Result<usize, CodexError> {
    if !table_exists(transaction, "thread_spawn_edges")? {
        return Ok(0);
    }

    transaction
        .execute(
            "DELETE FROM thread_spawn_edges WHERE parent_thread_id = ?1 OR child_thread_id = ?1",
            [thread_id],
        )
        .map_err(CodexError::from)
}

fn clear_agent_job_assignments(
    transaction: &Transaction<'_>,
    thread_id: &str,
) -> Result<usize, CodexError> {
    if !table_has_column(transaction, "agent_job_items", "assigned_thread_id")? {
        return Ok(0);
    }

    transaction
        .execute(
            "UPDATE agent_job_items SET assigned_thread_id = NULL WHERE assigned_thread_id = ?1",
            [thread_id],
        )
        .map_err(CodexError::from)
}

fn ensure_no_known_database_references(
    transaction: &Transaction<'_>,
    thread_id: &str,
) -> Result<(), CodexError> {
    let mut remaining = Vec::new();

    add_remaining_count(
        &mut remaining,
        "threads",
        count_if_table_exists(
            transaction,
            "threads",
            "SELECT COUNT(*) FROM threads WHERE id = ?1",
            thread_id,
        )?,
    );
    add_remaining_count(
        &mut remaining,
        "thread_dynamic_tools",
        count_if_table_exists(
            transaction,
            "thread_dynamic_tools",
            "SELECT COUNT(*) FROM thread_dynamic_tools WHERE thread_id = ?1",
            thread_id,
        )?,
    );
    add_remaining_count(
        &mut remaining,
        "stage1_outputs",
        count_if_table_exists(
            transaction,
            "stage1_outputs",
            "SELECT COUNT(*) FROM stage1_outputs WHERE thread_id = ?1",
            thread_id,
        )?,
    );
    add_remaining_count(
        &mut remaining,
        "thread_goals",
        count_if_table_exists(
            transaction,
            "thread_goals",
            "SELECT COUNT(*) FROM thread_goals WHERE thread_id = ?1",
            thread_id,
        )?,
    );
    add_remaining_count(
        &mut remaining,
        "thread_spawn_edges",
        count_if_table_exists(
            transaction,
            "thread_spawn_edges",
            "SELECT COUNT(*) FROM thread_spawn_edges WHERE parent_thread_id = ?1 OR child_thread_id = ?1",
            thread_id,
        )?,
    );

    if table_has_column(transaction, "agent_job_items", "assigned_thread_id")? {
        add_remaining_count(
            &mut remaining,
            "agent_job_items.assigned_thread_id",
            transaction.query_row(
                "SELECT COUNT(*) FROM agent_job_items WHERE assigned_thread_id = ?1",
                [thread_id],
                |row| row.get::<_, i64>(0),
            )?,
        );
    }

    if remaining.is_empty() {
        return Ok(());
    }

    Err(CodexError::TrashOperation(format!(
        "Known database references remain after cleanup: {}",
        remaining.join(", ")
    )))
}

fn add_remaining_count(remaining: &mut Vec<String>, label: &str, count: i64) {
    if count > 0 {
        remaining.push(format!("{label}={count}"));
    }
}

fn count_if_table_exists(
    transaction: &Transaction<'_>,
    table_name: &str,
    query: &str,
    thread_id: &str,
) -> Result<i64, CodexError> {
    if !table_exists(transaction, table_name)? {
        return Ok(0);
    }

    transaction
        .query_row(query, [thread_id], |row| row.get(0))
        .map_err(CodexError::from)
}

fn table_exists(transaction: &Transaction<'_>, table_name: &str) -> Result<bool, CodexError> {
    transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
            [table_name],
            |row| row.get::<_, bool>(0),
        )
        .map_err(CodexError::from)
}

fn table_has_column(
    transaction: &Transaction<'_>,
    table_name: &str,
    column_name: &str,
) -> Result<bool, CodexError> {
    if !table_exists(transaction, table_name)? {
        return Ok(false);
    }

    transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2)",
            [table_name, column_name],
            |row| row.get::<_, bool>(0),
        )
        .map_err(CodexError::from)
}

fn expand_rollout_path(codex_home: &Path, rollout_path: &str) -> PathBuf {
    let expanded = expand_tilde(rollout_path.trim());

    if expanded.is_absolute() {
        expanded
    } else {
        codex_home.join(expanded)
    }
}

fn ensure_trashable_transcript(path: &Path, codex_home: &Path) -> Result<bool, CodexError> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            ensure_transcript_path_is_in_codex_sessions(path, codex_home)?;
            return Ok(false);
        }
        Err(source) => {
            return Err(CodexError::TrashOperation(format!(
                "Could not read session transcript at {}: {source}",
                path.display()
            )));
        }
    };

    if !metadata.is_file() {
        return Err(CodexError::TrashOperation(format!(
            "Expected session transcript to be a file: {}",
            path.display()
        )));
    }

    let canonical_path = path.canonicalize().map_err(|source| {
        CodexError::TrashOperation(format!(
            "Could not resolve session transcript at {}: {source}",
            path.display()
        ))
    })?;

    let sessions_root = canonical_existing_root(&codex_home.join("sessions"))?;
    let archived_root = canonical_existing_root(&codex_home.join("archived_sessions"))?;
    let under_sessions = sessions_root
        .as_ref()
        .is_some_and(|root| canonical_path.starts_with(root));
    let under_archived = archived_root
        .as_ref()
        .is_some_and(|root| canonical_path.starts_with(root));

    if !under_sessions && !under_archived {
        return Err(CodexError::TrashOperation(format!(
            "Refusing to move transcript outside Codex sessions directories: {}",
            canonical_path.display()
        )));
    }

    Ok(true)
}

fn ensure_transcript_path_is_in_codex_sessions(
    path: &Path,
    codex_home: &Path,
) -> Result<(), CodexError> {
    let path = normalize_transcript_path(path)?;
    let sessions_root = normalize_transcript_path(&codex_home.join("sessions"))?;
    let archived_root = normalize_transcript_path(&codex_home.join("archived_sessions"))?;

    if path.starts_with(&sessions_root) || path.starts_with(&archived_root) {
        return Ok(());
    }

    Err(CodexError::TrashOperation(format!(
        "Refusing to clean missing transcript outside Codex sessions directories: {}",
        path.display()
    )))
}

fn normalize_transcript_path(path: &Path) -> Result<PathBuf, CodexError> {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(CodexError::TrashOperation(format!(
                    "Transcript path must not contain parent-directory segments: {}",
                    path.display()
                )));
            }
        }
    }

    Ok(normalized)
}

fn trash_existing_transcript_if_present(path: &Path) -> Result<(), String> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => {
            return Err(format!(
                "Expected session transcript to be a file: {}",
                path.display()
            ));
        }
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(format!(
                "Could not re-read session transcript at {}: {error}",
                path.display()
            ));
        }
    }

    if let Err(error) = trash::delete(path) {
        if transcript_is_missing(path) {
            // Another process removed the transcript after validation; keep the DB/index cleanup.
            return Ok(());
        }

        return Err(format!(
            "Could not move transcript {} to the system Trash: {error}",
            path.display()
        ));
    }

    Ok(())
}

fn transcript_is_missing(path: &Path) -> bool {
    matches!(fs::metadata(path), Err(error) if error.kind() == ErrorKind::NotFound)
}

fn canonical_existing_root(root: &Path) -> Result<Option<PathBuf>, CodexError> {
    if !root.exists() {
        return Ok(None);
    }

    root.canonicalize().map(Some).map_err(|source| {
        CodexError::TrashOperation(format!(
            "Could not resolve Codex session root {}: {source}",
            root.display()
        ))
    })
}

fn plan_session_index_cleanup(
    codex_home: &Path,
    thread_id: &str,
    rollout_path: &str,
    transcript: &Path,
) -> Result<Option<SessionIndexCleanup>, CodexError> {
    let path = codex_home.join("session_index.jsonl");
    let original_contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(CodexError::TrashOperation(format!(
                "Could not read session_index.jsonl at {}: {error}",
                path.display()
            )));
        }
    };

    let criteria = SessionIndexCriteria {
        thread_id: thread_id.to_string(),
        rollout_path: rollout_path.trim().to_string(),
        expanded_rollout_path: transcript.to_string_lossy().into_owned(),
        rollout_file_name: transcript
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .map(str::to_string),
    };
    let (cleaned_contents, removed_count) =
        clean_session_index_contents(&original_contents, &criteria);

    if removed_count == 0 {
        return Ok(None);
    }

    Ok(Some(SessionIndexCleanup {
        path,
        original_contents,
        cleaned_contents,
    }))
}

fn clean_session_index_contents(
    original_contents: &str,
    criteria: &SessionIndexCriteria,
) -> (String, usize) {
    let mut cleaned_contents = String::with_capacity(original_contents.len());
    let mut removed_count = 0;

    for line_with_ending in original_contents.split_inclusive('\n') {
        if session_index_line_matches(line_with_ending, criteria) {
            removed_count += 1;
        } else {
            cleaned_contents.push_str(line_with_ending);
        }
    }

    (cleaned_contents, removed_count)
}

fn session_index_line_matches(line_with_ending: &str, criteria: &SessionIndexCriteria) -> bool {
    let line = line_with_ending.trim_end_matches(['\r', '\n']);
    if line.trim().is_empty() {
        return false;
    }

    let Ok(value) = serde_json::from_str::<Value>(line.trim()) else {
        return false;
    };

    json_value_matches_session(&value, criteria)
}

fn json_value_matches_session(value: &Value, criteria: &SessionIndexCriteria) -> bool {
    match value {
        Value::String(value) => string_matches_session(value, criteria),
        Value::Array(values) => values
            .iter()
            .any(|value| json_value_matches_session(value, criteria)),
        Value::Object(entries) => entries
            .values()
            .any(|value| json_value_matches_session(value, criteria)),
        _ => false,
    }
}

fn string_matches_session(value: &str, criteria: &SessionIndexCriteria) -> bool {
    if value == criteria.thread_id
        || (!criteria.rollout_path.is_empty() && value == criteria.rollout_path)
        || value == criteria.expanded_rollout_path
    {
        return true;
    }

    criteria
        .rollout_file_name
        .as_ref()
        .is_some_and(|file_name| {
            value == file_name
                || value.ends_with(&format!("/{file_name}"))
                || value.ends_with(&format!("\\{file_name}"))
        })
}

fn apply_session_index_cleanup(cleanup: Option<&SessionIndexCleanup>) -> Result<(), CodexError> {
    let Some(cleanup) = cleanup else {
        return Ok(());
    };

    let current_contents = fs::read_to_string(&cleanup.path).map_err(|error| {
        CodexError::TrashOperation(format!(
            "Could not re-read session_index.jsonl before cleanup at {}: {error}",
            cleanup.path.display()
        ))
    })?;

    if current_contents != cleanup.original_contents {
        return Err(CodexError::TrashOperation(
            "session_index.jsonl changed while preparing deletion; close Codex Desktop and retry"
                .to_string(),
        ));
    }

    replace_file_contents(&cleanup.path, &cleanup.cleaned_contents)
}

fn restore_session_index_cleanup(cleanup: Option<&SessionIndexCleanup>) -> Result<(), CodexError> {
    let Some(cleanup) = cleanup else {
        return Ok(());
    };

    match fs::read_to_string(&cleanup.path) {
        Ok(current_contents) if current_contents == cleanup.original_contents => Ok(()),
        Ok(current_contents) if current_contents == cleanup.cleaned_contents => {
            replace_file_contents(&cleanup.path, &cleanup.original_contents)
        }
        Ok(_) => Err(CodexError::TrashOperation(
            "session_index.jsonl changed after cleanup; refusing to overwrite newer contents"
                .to_string(),
        )),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            replace_file_contents(&cleanup.path, &cleanup.original_contents)
        }
        Err(error) => Err(CodexError::TrashOperation(format!(
            "Could not re-read session_index.jsonl while restoring cleanup at {}: {error}",
            cleanup.path.display()
        ))),
    }
}

fn replace_file_contents(path: &Path, contents: &str) -> Result<(), CodexError> {
    let parent = path.parent().ok_or_else(|| {
        CodexError::TrashOperation(format!(
            "session_index.jsonl path has no parent directory: {}",
            path.display()
        ))
    })?;
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .ok_or_else(|| {
            CodexError::TrashOperation(format!(
                "session_index.jsonl filename is not valid UTF-8: {}",
                path.display()
            ))
        })?;
    let temp_path = parent.join(format!(
        ".{file_name}.ferricodex-{}-{}.tmp",
        process::id(),
        now_ms()?
    ));

    fs::write(&temp_path, contents).map_err(|error| {
        CodexError::TrashOperation(format!(
            "Could not write temporary session_index.jsonl cleanup file {}: {error}",
            temp_path.display()
        ))
    })?;

    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(CodexError::TrashOperation(format!(
            "Could not replace session_index.jsonl at {}: {error}",
            path.display()
        )));
    }

    Ok(())
}

fn now_ms() -> Result<u128, CodexError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| CodexError::TrashOperation(format!("System clock error: {error}")))?;

    Ok(duration.as_millis())
}

#[cfg(test)]
mod tests {
    use super::{clean_session_index_contents, ensure_trashable_transcript, SessionIndexCriteria};
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_codex_home(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ferricodex-deletion-{name}-{}-{timestamp}",
            process::id()
        ));

        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("test Codex home should be created");
        path
    }

    fn criteria() -> SessionIndexCriteria {
        SessionIndexCriteria {
            thread_id: "019e49dc-5acc-74b2-bb51-d4722df727c1".to_string(),
            rollout_path: "/Users/test/.codex/sessions/2026/05/21/rollout-2026-05-21T17-27-18-019e49dc-5acc-74b2-bb51-d4722df727c1.jsonl".to_string(),
            expanded_rollout_path: "/Users/test/.codex/sessions/2026/05/21/rollout-2026-05-21T17-27-18-019e49dc-5acc-74b2-bb51-d4722df727c1.jsonl".to_string(),
            rollout_file_name: Some(
                "rollout-2026-05-21T17-27-18-019e49dc-5acc-74b2-bb51-d4722df727c1.jsonl"
                    .to_string(),
            ),
        }
    }

    #[test]
    fn clean_session_index_contents_removes_matching_id_line() {
        let original = concat!(
            "{\"id\":\"keep\",\"thread_name\":\"Keep\"}\n",
            "{\"id\":\"019e49dc-5acc-74b2-bb51-d4722df727c1\",\"thread_name\":\"Delete\"}\n",
            "not-json\n"
        );

        let (cleaned, removed_count) = clean_session_index_contents(original, &criteria());

        assert_eq!(removed_count, 1);
        assert_eq!(
            cleaned,
            concat!("{\"id\":\"keep\",\"thread_name\":\"Keep\"}\n", "not-json\n")
        );
    }

    #[test]
    fn clean_session_index_contents_matches_nested_rollout_path() {
        let original = concat!(
            "{\"id\":\"keep\"}\n",
            "{\"payload\":{\"rollout_path\":\"/Users/test/.codex/sessions/2026/05/21/rollout-2026-05-21T17-27-18-019e49dc-5acc-74b2-bb51-d4722df727c1.jsonl\"}}\n"
        );

        let (cleaned, removed_count) = clean_session_index_contents(original, &criteria());

        assert_eq!(removed_count, 1);
        assert_eq!(cleaned, "{\"id\":\"keep\"}\n");
    }

    #[test]
    fn ensure_trashable_transcript_allows_missing_file_under_sessions() {
        let home = test_codex_home("missing-session-file");
        let transcript = home.join("sessions/2026/05/06/missing.jsonl");

        let exists = ensure_trashable_transcript(&transcript, &home)
            .expect("missing session transcript should still be cleanable");

        assert!(!exists);
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn ensure_trashable_transcript_allows_existing_file_under_sessions() {
        let home = test_codex_home("existing-session-file");
        let parent = home.join("sessions/2026/05/06");
        fs::create_dir_all(&parent).expect("sessions parent should be created");
        let transcript = parent.join("rollout.jsonl");
        fs::write(&transcript, "{}\n").expect("transcript should be written");

        let exists = ensure_trashable_transcript(&transcript, &home)
            .expect("existing session transcript should be trashable");

        assert!(exists);
        let _ = fs::remove_dir_all(home);
    }

    #[test]
    fn ensure_trashable_transcript_rejects_missing_file_outside_sessions() {
        let home = test_codex_home("missing-outside-sessions");
        let transcript = home.join("other/missing.jsonl");

        let error = ensure_trashable_transcript(&transcript, &home)
            .expect_err("missing transcript outside Codex session roots should be rejected");

        assert!(error
            .to_string()
            .contains("outside Codex sessions directories"));
        let _ = fs::remove_dir_all(home);
    }
}
