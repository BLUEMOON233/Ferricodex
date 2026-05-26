use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use std::collections::HashSet;

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

    read_threads_from_connection(&connection)
}

fn read_threads_from_connection(connection: &Connection) -> Result<Vec<CodexThread>, CodexError> {
    let columns = thread_columns(connection)?;
    let query = thread_query(&columns);
    let mut statement = connection.prepare(&query)?;

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

fn thread_columns(connection: &Connection) -> Result<HashSet<String>, CodexError> {
    let mut statement = connection.prepare("PRAGMA table_info(threads)")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;

    rows.collect::<Result<HashSet<_>, _>>()
        .map_err(CodexError::from)
}

fn thread_query(columns: &HashSet<String>) -> String {
    format!(
        r#"
        SELECT
            {id} AS id,
            {title} AS title,
            {cwd} AS cwd,
            {preview} AS preview,
            {rollout_path} AS rollout_path,
            {created_at} AS created_at,
            {updated_at} AS updated_at,
            {created_at_ms} AS created_at_ms,
            {updated_at_ms} AS updated_at_ms,
            {model} AS model,
            {archived} AS archived
        FROM threads
        ORDER BY updated_at_ms DESC, id DESC
        "#,
        id = text_column_or_empty(columns, "id"),
        title = text_column_or_empty(columns, "title"),
        cwd = text_column_or_empty(columns, "cwd"),
        preview = preview_expr(columns),
        rollout_path = text_column_or_empty(columns, "rollout_path"),
        created_at = seconds_expr(columns, "created_at", "created_at_ms"),
        updated_at = seconds_expr(columns, "updated_at", "updated_at_ms"),
        created_at_ms = millis_expr(columns, "created_at_ms", "created_at"),
        updated_at_ms = millis_expr(columns, "updated_at_ms", "updated_at"),
        model = optional_text_column(columns, "model"),
        archived = integer_column_or_zero(columns, "archived"),
    )
}

fn text_column_or_empty(columns: &HashSet<String>, column: &str) -> String {
    if columns.contains(column) {
        format!("COALESCE({column}, '')")
    } else {
        "''".to_string()
    }
}

fn preview_expr(columns: &HashSet<String>) -> String {
    match (columns.contains("preview"), columns.contains("first_user_message")) {
        (true, true) => "COALESCE(preview, first_user_message, '')".to_string(),
        (true, false) => "COALESCE(preview, '')".to_string(),
        (false, true) => "COALESCE(first_user_message, '')".to_string(),
        (false, false) => "''".to_string(),
    }
}

fn optional_text_column(columns: &HashSet<String>, column: &str) -> String {
    if columns.contains(column) {
        column.to_string()
    } else {
        "NULL".to_string()
    }
}

fn integer_column_or_zero(columns: &HashSet<String>, column: &str) -> String {
    if columns.contains(column) {
        format!("CAST(COALESCE({column}, 0) AS INTEGER)")
    } else {
        "0".to_string()
    }
}

fn seconds_expr(columns: &HashSet<String>, seconds_column: &str, millis_column: &str) -> String {
    match (columns.contains(seconds_column), columns.contains(millis_column)) {
        (true, true) => format!(
            "CAST(COALESCE({seconds_column}, {millis_column} / 1000, 0) AS INTEGER)"
        ),
        (true, false) => format!("CAST(COALESCE({seconds_column}, 0) AS INTEGER)"),
        (false, true) => format!("CAST(COALESCE({millis_column} / 1000, 0) AS INTEGER)"),
        (false, false) => "0".to_string(),
    }
}

fn millis_expr(columns: &HashSet<String>, millis_column: &str, seconds_column: &str) -> String {
    match (columns.contains(millis_column), columns.contains(seconds_column)) {
        (true, true) => format!(
            "CAST(COALESCE({millis_column}, {seconds_column} * 1000, 0) AS INTEGER)"
        ),
        (true, false) => format!("CAST(COALESCE({millis_column}, 0) AS INTEGER)"),
        (false, true) => format!("CAST(COALESCE({seconds_column} * 1000, 0) AS INTEGER)"),
        (false, false) => "0".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_threads_with_windows_schema_without_preview_column() {
        let connection = Connection::open_in_memory().expect("open in-memory db");
        connection
            .execute(
                r#"
                CREATE TABLE threads (
                    id TEXT PRIMARY KEY,
                    rollout_path TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    source TEXT NOT NULL,
                    model_provider TEXT NOT NULL,
                    cwd TEXT NOT NULL,
                    title TEXT NOT NULL,
                    sandbox_policy TEXT NOT NULL,
                    approval_mode TEXT NOT NULL,
                    tokens_used INTEGER NOT NULL DEFAULT 0,
                    has_user_event INTEGER NOT NULL DEFAULT 0,
                    archived INTEGER NOT NULL DEFAULT 0,
                    archived_at INTEGER,
                    git_sha TEXT,
                    git_branch TEXT,
                    git_origin_url TEXT,
                    cli_version TEXT NOT NULL DEFAULT '',
                    first_user_message TEXT NOT NULL DEFAULT '',
                    agent_nickname TEXT,
                    agent_role TEXT,
                    memory_mode TEXT NOT NULL DEFAULT 'enabled',
                    model TEXT,
                    reasoning_effort TEXT,
                    agent_path TEXT,
                    created_at_ms INTEGER,
                    updated_at_ms INTEGER
                )
                "#,
                [],
            )
            .expect("create threads table");
        connection
            .execute(
                r#"
                INSERT INTO threads (
                    id,
                    rollout_path,
                    created_at,
                    updated_at,
                    source,
                    model_provider,
                    cwd,
                    title,
                    sandbox_policy,
                    approval_mode,
                    first_user_message,
                    model,
                    created_at_ms,
                    updated_at_ms,
                    archived
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                "#,
                (
                    "thread-1",
                    "C:/Users/user/.codex/sessions/rollout.jsonl",
                    10_i64,
                    20_i64,
                    "codex-cli",
                    "openai",
                    "C:/Users/user/project",
                    "Windows schema",
                    "workspace-write",
                    "on-request",
                    "Preview fallback from first user message",
                    "gpt-5.1-codex",
                    10_500_i64,
                    20_500_i64,
                    0_i64,
                ),
            )
            .expect("insert thread");

        let threads = read_threads_from_connection(&connection).expect("read threads");

        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].id, "thread-1");
        assert_eq!(threads[0].preview, "Preview fallback from first user message");
        assert_eq!(threads[0].created_at_ms, 10_500);
        assert_eq!(threads[0].updated_at_ms, 20_500);
        assert_eq!(threads[0].model.as_deref(), Some("gpt-5.1-codex"));
        assert!(!threads[0].archived);
    }
}
