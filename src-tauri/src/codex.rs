use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use std::env;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

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
