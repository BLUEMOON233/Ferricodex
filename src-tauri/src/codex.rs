use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

const WORKSPACE_SCAN_ENTRY_LIMIT: u64 = 20_000;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexHomeStatus {
    pub path: String,
    pub exists: bool,
    pub state_db_exists: bool,
    pub source: CodexHomeSource,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CodexHomeSource {
    Env,
    Default,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexThread {
    pub id: String,
    pub title: String,
    pub cwd: String,
    pub preview: String,
    pub rollout_path: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub model: Option<String>,
    pub archived: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceMetadata {
    pub path: String,
    pub exists: bool,
    pub is_directory: bool,
    pub is_file: bool,
    pub size_bytes: Option<u64>,
    pub file_count: Option<u64>,
    pub directory_count: Option<u64>,
    pub modified_at_ms: Option<u64>,
    pub scan_truncated: bool,
}

#[derive(Debug)]
pub enum CodexError {
    HomeNotFound(PathBuf),
    StateDbNotFound(PathBuf),
    HomeDirUnavailable,
    Sqlite(rusqlite::Error),
}

impl fmt::Display for CodexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HomeNotFound(path) => {
                write!(formatter, "Codex home was not found at {}", path.display())
            }
            Self::StateDbNotFound(path) => {
                write!(
                    formatter,
                    "Codex state database was not found at {}",
                    path.display()
                )
            }
            Self::HomeDirUnavailable => {
                write!(
                    formatter,
                    "Could not determine the current user's home directory"
                )
            }
            Self::Sqlite(error) => {
                write!(formatter, "Could not read Codex state database: {error}")
            }
        }
    }
}

impl Error for CodexError {}

impl From<rusqlite::Error> for CodexError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

struct CodexHome {
    path: PathBuf,
    source: CodexHomeSource,
}

pub fn home_status() -> CodexHomeStatus {
    match resolve_codex_home() {
        Ok(home) => {
            let state_db_path = home.path.join("state_5.sqlite");

            CodexHomeStatus {
                path: home.path.to_string_lossy().into_owned(),
                exists: home.path.exists(),
                state_db_exists: state_db_path.exists(),
                source: home.source,
            }
        }
        Err(CodexError::HomeDirUnavailable) => CodexHomeStatus {
            path: String::new(),
            exists: false,
            state_db_exists: false,
            source: CodexHomeSource::Default,
        },
        Err(_) => CodexHomeStatus {
            path: String::new(),
            exists: false,
            state_db_exists: false,
            source: CodexHomeSource::Default,
        },
    }
}

pub fn list_threads() -> Result<Vec<CodexThread>, CodexError> {
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

    let mut statement = connection.prepare(
        r#"
        SELECT
            id,
            title,
            cwd,
            preview,
            rollout_path,
            created_at,
            updated_at,
            COALESCE(created_at_ms, created_at * 1000),
            COALESCE(updated_at_ms, updated_at * 1000),
            model,
            archived
        FROM threads
        ORDER BY updated_at_ms DESC, id DESC
        "#,
    )?;

    let rows = statement.query_map([], |row| {
        Ok(CodexThread {
            id: row.get(0)?,
            title: row.get(1)?,
            cwd: row.get(2)?,
            preview: row.get(3)?,
            rollout_path: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
            created_at_ms: row.get(7)?,
            updated_at_ms: row.get(8)?,
            model: row.get(9)?,
            archived: row.get::<_, i64>(10)? != 0,
        })
    })?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(CodexError::from)
}

pub fn workspace_metadata(path: String) -> WorkspaceMetadata {
    let path_buf = expand_tilde(path.trim());

    match fs::metadata(&path_buf) {
        Ok(metadata) => {
            let modified_at_ms = metadata.modified().ok().and_then(|modified| {
                modified
                    .duration_since(UNIX_EPOCH)
                    .ok()
                    .map(|duration| duration.as_millis() as u64)
            });

            if metadata.is_dir() {
                let scan = scan_directory(&path_buf, WORKSPACE_SCAN_ENTRY_LIMIT);

                return WorkspaceMetadata {
                    path: path_buf.to_string_lossy().into_owned(),
                    exists: true,
                    is_directory: true,
                    is_file: false,
                    size_bytes: Some(scan.size_bytes),
                    file_count: Some(scan.file_count),
                    directory_count: Some(scan.directory_count),
                    modified_at_ms,
                    scan_truncated: scan.truncated,
                };
            }

            WorkspaceMetadata {
                path: path_buf.to_string_lossy().into_owned(),
                exists: true,
                is_directory: false,
                is_file: metadata.is_file(),
                size_bytes: Some(metadata.len()),
                file_count: Some(u64::from(metadata.is_file())),
                directory_count: Some(0),
                modified_at_ms,
                scan_truncated: false,
            }
        }
        Err(_) => WorkspaceMetadata {
            path: path_buf.to_string_lossy().into_owned(),
            exists: false,
            is_directory: false,
            is_file: false,
            size_bytes: None,
            file_count: None,
            directory_count: None,
            modified_at_ms: None,
            scan_truncated: false,
        },
    }
}

fn resolve_codex_home() -> Result<CodexHome, CodexError> {
    if let Ok(value) = env::var("CODEX_HOME") {
        if !value.trim().is_empty() {
            return Ok(CodexHome {
                path: expand_tilde(value.trim()),
                source: CodexHomeSource::Env,
            });
        }
    }

    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .ok_or(CodexError::HomeDirUnavailable)?;

    Ok(CodexHome {
        path: home.join(".codex"),
        source: CodexHomeSource::Default,
    })
}

struct DirectoryScan {
    size_bytes: u64,
    file_count: u64,
    directory_count: u64,
    visited_entries: u64,
    truncated: bool,
}

fn scan_directory(path: &Path, entry_limit: u64) -> DirectoryScan {
    let mut scan = DirectoryScan {
        size_bytes: 0,
        file_count: 0,
        directory_count: 0,
        visited_entries: 0,
        truncated: false,
    };
    let mut pending = vec![path.to_path_buf()];

    while let Some(current_path) = pending.pop() {
        let entries = match fs::read_dir(current_path) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            if scan.visited_entries >= entry_limit {
                scan.truncated = true;
                return scan;
            }

            scan.visited_entries += 1;

            let metadata = match entry.path().symlink_metadata() {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };

            if metadata.is_dir() {
                scan.directory_count += 1;
                pending.push(entry.path());
            } else if metadata.is_file() {
                scan.file_count += 1;
                scan.size_bytes = scan.size_bytes.saturating_add(metadata.len());
            }
        }
    }

    scan
}

fn expand_tilde(value: &str) -> PathBuf {
    if value == "~" {
        return env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(value));
    }

    if let Some(rest) = value.strip_prefix("~/") {
        if let Some(home) = env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }

    PathBuf::from(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_workspace() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        env::temp_dir().join(format!("codex-history-manager-test-{suffix}"))
    }

    #[test]
    fn workspace_metadata_counts_directory_contents() {
        let root = temp_workspace();
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("test directory should be created");

        let mut first = File::create(root.join("first.txt")).expect("test file should be created");
        first
            .write_all(b"hello")
            .expect("test file should be writable");

        let mut second =
            File::create(nested.join("second.txt")).expect("nested test file should be created");
        second
            .write_all(b"world!")
            .expect("nested test file should be writable");

        let metadata = workspace_metadata(root.to_string_lossy().into_owned());

        assert!(metadata.exists);
        assert!(metadata.is_directory);
        assert!(!metadata.is_file);
        assert_eq!(metadata.file_count, Some(2));
        assert_eq!(metadata.directory_count, Some(1));
        assert_eq!(metadata.size_bytes, Some(11));
        assert!(metadata.modified_at_ms.is_some());
        assert!(!metadata.scan_truncated);

        fs::remove_dir_all(root).expect("test directory should be removed");
    }

    #[test]
    fn workspace_metadata_reports_missing_paths() {
        let metadata = workspace_metadata(
            temp_workspace()
                .join("missing")
                .to_string_lossy()
                .into_owned(),
        );

        assert!(!metadata.exists);
        assert!(!metadata.is_directory);
        assert!(!metadata.is_file);
        assert_eq!(metadata.file_count, None);
        assert_eq!(metadata.directory_count, None);
        assert_eq!(metadata.size_bytes, None);
    }
}
