use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::error::CodexError;
use super::home::{expand_tilde, resolve_codex_home};

struct ThreadArchiveState {
    rollout_path: String,
    archived: bool,
}

struct ArchiveTransition {
    source: PathBuf,
    destination: PathBuf,
}

pub fn set_thread_archive_state(thread_id: String, archived: bool) -> Result<(), CodexError> {
    let thread_id = thread_id.trim();
    if thread_id.is_empty() {
        return Err(CodexError::ArchiveOperation(
            "Thread id is unavailable".to_string(),
        ));
    }

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

    let transaction = connection.transaction()?;
    let current_state = transaction
        .query_row(
            "SELECT rollout_path, archived FROM threads WHERE id = ?1",
            [thread_id],
            |row| {
                Ok(ThreadArchiveState {
                    rollout_path: row.get(0)?,
                    archived: row.get::<_, i64>(1)? != 0,
                })
            },
        )
        .optional()?
        .ok_or_else(|| CodexError::ThreadNotFound(thread_id.to_string()))?;

    if current_state.archived == archived {
        return Ok(());
    }

    let transition = plan_archive_transition(&home.path, &current_state.rollout_path, archived)?;
    let destination_parent = transition.destination.parent().ok_or_else(|| {
        CodexError::ArchiveOperation(format!(
            "Archive destination has no parent directory: {}",
            transition.destination.display()
        ))
    })?;

    fs::create_dir_all(destination_parent).map_err(|source| {
        CodexError::ArchiveDirectoryCreate {
            path: destination_parent.to_path_buf(),
            source,
        }
    })?;

    if transition.destination.exists() {
        return Err(CodexError::ArchiveOperation(format!(
            "Archive destination already exists: {}",
            transition.destination.display()
        )));
    }

    let archived_value = if archived { 1_i64 } else { 0_i64 };
    let archived_at = if archived { Some(now_ms()?) } else { None };
    let destination_value = transition.destination.to_string_lossy().into_owned();

    transaction.execute(
        r#"
        UPDATE threads
        SET archived = ?1,
            archived_at = ?2,
            rollout_path = ?3
        WHERE id = ?4
        "#,
        params![archived_value, archived_at, destination_value, thread_id],
    )?;

    fs::rename(&transition.source, &transition.destination).map_err(|source| {
        CodexError::ArchiveFileMove {
            from: transition.source.clone(),
            to: transition.destination.clone(),
            source,
        }
    })?;

    if let Err(error) = transaction.commit() {
        if let Err(rollback_error) = fs::rename(&transition.destination, &transition.source) {
            return Err(CodexError::ArchiveOperation(format!(
                "Could not commit Codex archive state: {error}; also failed to move transcript back from {} to {}: {rollback_error}",
                transition.destination.display(),
                transition.source.display()
            )));
        }

        return Err(CodexError::from(error));
    }

    Ok(())
}

fn plan_archive_transition(
    codex_home: &Path,
    rollout_path: &str,
    archived: bool,
) -> Result<ArchiveTransition, CodexError> {
    let source = expand_rollout_path(codex_home, rollout_path);
    let sessions_root = codex_home.join("sessions");
    let archived_root = codex_home.join("archived_sessions");

    if archived {
        ensure_existing_file_under(&source, &sessions_root, "active session transcript")?;
        let file_name = source
            .file_name()
            .ok_or_else(|| {
                CodexError::ArchiveOperation(format!(
                    "Transcript path has no filename: {}",
                    source.display()
                ))
            })?
            .to_owned();
        let destination = archived_root.join(&file_name);

        return Ok(ArchiveTransition {
            source,
            destination,
        });
    }

    ensure_existing_file_under(&source, &archived_root, "archived session transcript")?;
    let file_name = source
        .file_name()
        .ok_or_else(|| {
            CodexError::ArchiveOperation(format!(
                "Transcript path has no filename: {}",
                source.display()
            ))
        })?
        .to_owned();
    let destination = active_session_destination(&sessions_root, &file_name)?;

    Ok(ArchiveTransition {
        source,
        destination,
    })
}

fn expand_rollout_path(codex_home: &Path, rollout_path: &str) -> PathBuf {
    let expanded = expand_tilde(rollout_path.trim());

    if expanded.is_absolute() {
        expanded
    } else {
        codex_home.join(expanded)
    }
}

fn ensure_existing_file_under(path: &Path, root: &Path, label: &str) -> Result<(), CodexError> {
    let metadata = fs::metadata(path).map_err(|source| {
        CodexError::ArchiveOperation(format!(
            "Could not read {label} at {}: {source}",
            path.display()
        ))
    })?;

    if !metadata.is_file() {
        return Err(CodexError::ArchiveOperation(format!(
            "Expected {label} to be a file: {}",
            path.display()
        )));
    }

    let canonical_path = path.canonicalize().map_err(|source| {
        CodexError::ArchiveOperation(format!(
            "Could not resolve {label} at {}: {source}",
            path.display()
        ))
    })?;
    let canonical_root = root.canonicalize().map_err(|source| {
        CodexError::ArchiveOperation(format!(
            "Could not resolve Codex archive root {}: {source}",
            root.display()
        ))
    })?;

    if !canonical_path.starts_with(&canonical_root) {
        return Err(CodexError::ArchiveOperation(format!(
            "Refusing to move {label} outside expected Codex directory: {}",
            canonical_path.display()
        )));
    }

    Ok(())
}

fn active_session_destination(
    sessions_root: &Path,
    file_name: &std::ffi::OsStr,
) -> Result<PathBuf, CodexError> {
    let file_name = file_name.to_str().ok_or_else(|| {
        CodexError::ArchiveOperation("Transcript filename is not valid UTF-8".to_string())
    })?;
    let (year, month, day) = rollout_date_parts(file_name)?;

    Ok(sessions_root
        .join(year)
        .join(month)
        .join(day)
        .join(file_name))
}

fn rollout_date_parts(file_name: &str) -> Result<(&str, &str, &str), CodexError> {
    let date = file_name
        .strip_prefix("rollout-")
        .and_then(|rest| rest.get(0..10))
        .ok_or_else(|| {
            CodexError::ArchiveOperation(format!(
                "Could not infer session date from transcript filename: {file_name}"
            ))
        })?;

    let is_valid_date_shape = date.len() == 10
        && date.as_bytes()[0..4].iter().all(u8::is_ascii_digit)
        && date.as_bytes()[4] == b'-'
        && date.as_bytes()[5..7].iter().all(u8::is_ascii_digit)
        && date.as_bytes()[7] == b'-'
        && date.as_bytes()[8..10].iter().all(u8::is_ascii_digit);

    if !is_valid_date_shape {
        return Err(CodexError::ArchiveOperation(format!(
            "Could not infer session date from transcript filename: {file_name}"
        )));
    }

    Ok((&date[0..4], &date[5..7], &date[8..10]))
}

fn now_ms() -> Result<i64, CodexError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            CodexError::ArchiveOperation(format!("System time is invalid: {error}"))
        })?;

    Ok(duration.as_millis() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollout_date_parts_extracts_sessions_directory_parts() {
        let (year, month, day) = rollout_date_parts(
            "rollout-2026-04-30T10-38-59-019ddc40-fd28-7ca3-b05a-5de11ca24124.jsonl",
        )
        .expect("valid rollout filename");

        assert_eq!(year, "2026");
        assert_eq!(month, "04");
        assert_eq!(day, "30");
    }

    #[test]
    fn rollout_date_parts_rejects_unknown_filename_shape() {
        assert!(rollout_date_parts("session.jsonl").is_err());
    }
}
