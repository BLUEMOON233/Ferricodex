use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

use super::error::CodexError;
use super::home::resolve_codex_home;

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
