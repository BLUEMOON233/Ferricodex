use serde::Serialize;
use std::env;
use std::path::PathBuf;

use super::error::CodexError;

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

pub(super) struct CodexHome {
    pub(super) path: PathBuf,
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

pub(super) fn resolve_codex_home() -> Result<CodexHome, CodexError> {
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

pub(super) fn expand_tilde(value: &str) -> PathBuf {
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
