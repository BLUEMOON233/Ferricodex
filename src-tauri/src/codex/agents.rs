use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use super::error::CodexError;
use super::home::{expand_tilde, resolve_codex_home};
use super::paths::paths_match;
use super::threads::list_threads;

const WORKSPACE_AGENT_DOCUMENT_FILE_NAME: &str = "AGENTS.md";
const GLOBAL_AGENT_DOCUMENT_FILE_NAME: &str = "AGENT.md";
const AGENT_DOCUMENT_MAX_BYTES: u64 = 512 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexAgentDocument {
    pub path: String,
    pub exists: bool,
    pub revision: String,
    pub content: String,
    pub size_bytes: Option<u64>,
    pub modified_at_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexAgentDocumentUpdate {
    pub workspace_path: String,
    pub revision: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexGlobalAgentDocumentUpdate {
    pub revision: String,
    pub content: String,
}

pub fn agent_document(workspace_path: String) -> Result<CodexAgentDocument, CodexError> {
    let workspace_path = resolve_known_workspace_path(&workspace_path)?;
    let document_path = workspace_path.join(WORKSPACE_AGENT_DOCUMENT_FILE_NAME);

    read_agent_document_at(&document_path)
}

pub fn save_agent_document(
    input: CodexAgentDocumentUpdate,
) -> Result<CodexAgentDocument, CodexError> {
    validate_agent_content(&input.content)?;

    let workspace_path = resolve_known_workspace_path(&input.workspace_path)?;
    let document_path = workspace_path.join(WORKSPACE_AGENT_DOCUMENT_FILE_NAME);

    save_agent_document_at(&document_path, &input.revision, &input.content)
}

pub fn global_agent_document() -> Result<CodexAgentDocument, CodexError> {
    let document_path = global_agent_document_path()?;

    read_agent_document_at(&document_path)
}

pub fn save_global_agent_document(
    input: CodexGlobalAgentDocumentUpdate,
) -> Result<CodexAgentDocument, CodexError> {
    validate_agent_content(&input.content)?;

    let document_path = global_agent_document_path()?;
    ensure_global_agent_parent_exists(&document_path)?;

    save_agent_document_at(&document_path, &input.revision, &input.content)
}

fn global_agent_document_path() -> Result<PathBuf, CodexError> {
    Ok(resolve_codex_home()?.path.join(GLOBAL_AGENT_DOCUMENT_FILE_NAME))
}

fn ensure_global_agent_parent_exists(document_path: &Path) -> Result<(), CodexError> {
    let Some(parent) = document_path.parent() else {
        return Err(CodexError::AgentDocumentOperation(
            "Codex home path is unavailable.".to_string(),
        ));
    };

    fs::create_dir_all(parent).map_err(|source| {
        CodexError::AgentDocumentOperation(format!(
            "Could not create Codex home at {}: {source}",
            parent.display()
        ))
    })
}

fn resolve_known_workspace_path(path: &str) -> Result<PathBuf, CodexError> {
    let trimmed_path = path.trim();
    if trimmed_path.is_empty() {
        return Err(CodexError::AgentDocumentOperation(
            "Workspace path is unavailable.".to_string(),
        ));
    }

    let path_buf = expand_tilde(trimmed_path);
    ensure_known_workspace_path(&path_buf)?;
    Ok(path_buf)
}

fn ensure_known_workspace_path(path: &Path) -> Result<(), CodexError> {
    let is_known = list_threads()?.into_iter().any(|thread| {
        let cwd = thread.cwd.trim();
        !cwd.is_empty() && paths_match(path, &expand_tilde(cwd))
    });

    if is_known {
        return Ok(());
    }

    Err(CodexError::PathAccessDenied(format!(
        "Workspace path is not referenced by current Codex history: {}",
        path.display()
    )))
}

fn read_agent_document_at(path: &Path) -> Result<CodexAgentDocument, CodexError> {
    let Some(metadata) = agent_document_metadata(path)? else {
        return Ok(CodexAgentDocument {
            path: path.to_string_lossy().into_owned(),
            exists: false,
            revision: revision_for(""),
            content: String::new(),
            size_bytes: None,
            modified_at_ms: None,
        });
    };

    let content = fs::read_to_string(path).map_err(|source| {
        CodexError::AgentDocumentOperation(format!(
            "Could not read Agent document at {}: {source}",
            path.display()
        ))
    })?;
    let modified_at_ms = metadata.modified().ok().and_then(|modified| {
        modified
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|duration| duration.as_millis() as u64)
    });

    Ok(CodexAgentDocument {
        path: path.to_string_lossy().into_owned(),
        exists: true,
        revision: revision_for(&content),
        content,
        size_bytes: Some(metadata.len()),
        modified_at_ms,
    })
}

fn save_agent_document_at(
    path: &Path,
    revision: &str,
    content: &str,
) -> Result<CodexAgentDocument, CodexError> {
    validate_agent_content(content)?;
    ensure_document_parent_directory_exists(path)?;

    let original = read_agent_text(path)?;
    let current_revision = revision_for(&original);
    if revision != current_revision {
        let document_name = agent_document_name(path);
        return Err(CodexError::AgentDocumentOperation(
            format!("{document_name} changed outside Ferricodex. Reload it before saving."),
        ));
    }

    fs::write(path, content).map_err(|source| {
        CodexError::AgentDocumentOperation(format!(
            "Could not write Agent document at {}: {source}",
            path.display()
        ))
    })?;

    read_agent_document_at(path)
}

fn read_agent_text(path: &Path) -> Result<String, CodexError> {
    let Some(_metadata) = agent_document_metadata(path)? else {
        return Ok(String::new());
    };

    fs::read_to_string(path).map_err(|source| {
        CodexError::AgentDocumentOperation(format!(
            "Could not read Agent document at {}: {source}",
            path.display()
        ))
    })
}

fn agent_document_metadata(path: &Path) -> Result<Option<fs::Metadata>, CodexError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            let document_name = agent_document_name(path);

            if metadata.file_type().is_symlink() {
                return Err(CodexError::AgentDocumentOperation(format!(
                    "{document_name} is a symbolic link and cannot be edited: {}",
                    path.display()
                )));
            }

            if !metadata.is_file() {
                return Err(CodexError::AgentDocumentOperation(format!(
                    "{document_name} is not a regular file: {}",
                    path.display()
                )));
            }

            if metadata.len() > AGENT_DOCUMENT_MAX_BYTES {
                return Err(CodexError::AgentDocumentOperation(format!(
                    "{document_name} is too large to edit. Maximum size is {} bytes.",
                    AGENT_DOCUMENT_MAX_BYTES
                )));
            }

            Ok(Some(metadata))
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(CodexError::AgentDocumentOperation(format!(
            "Could not inspect Agent document at {}: {source}",
            path.display()
        ))),
    }
}

fn ensure_document_parent_directory_exists(document_path: &Path) -> Result<(), CodexError> {
    let Some(workspace_path) = document_path.parent() else {
        return Err(CodexError::AgentDocumentOperation(
            "Agent document parent folder is unavailable.".to_string(),
        ));
    };

    match fs::metadata(workspace_path) {
        Ok(metadata) if metadata.is_dir() => Ok(()),
        Ok(_) => Err(CodexError::AgentDocumentOperation(format!(
            "Agent document parent path is not a directory: {}",
            workspace_path.display()
        ))),
        Err(source) => Err(CodexError::AgentDocumentOperation(format!(
            "Agent document parent folder is unavailable at {}: {source}",
            workspace_path.display()
        ))),
    }
}

fn validate_agent_content(content: &str) -> Result<(), CodexError> {
    if content.len() as u64 > AGENT_DOCUMENT_MAX_BYTES {
        return Err(CodexError::AgentDocumentOperation(format!(
            "AGENTS.md content is too large. Maximum size is {} bytes.",
            AGENT_DOCUMENT_MAX_BYTES
        )));
    }

    Ok(())
}

fn agent_document_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Agent document".to_string())
}

fn revision_for(contents: &str) -> String {
    let mut hasher = DefaultHasher::new();
    contents.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_workspace() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let process_id = std::process::id();
        env::temp_dir().join(format!(
            "ferricodex-agent-test-{process_id}-{counter}-{suffix}"
        ))
    }

    #[test]
    fn reads_missing_agent_document_as_empty() {
        let workspace = temp_workspace();
        fs::create_dir_all(&workspace).expect("workspace should be created");
        let path = workspace.join(WORKSPACE_AGENT_DOCUMENT_FILE_NAME);

        let document = read_agent_document_at(&path).expect("document should be readable");

        assert!(!document.exists);
        assert_eq!(document.content, "");
        assert_eq!(document.size_bytes, None);

        fs::remove_dir_all(workspace).expect("workspace should be removed");
    }

    #[test]
    fn saves_new_agent_document() {
        let workspace = temp_workspace();
        fs::create_dir_all(&workspace).expect("workspace should be created");
        let path = workspace.join(WORKSPACE_AGENT_DOCUMENT_FILE_NAME);
        let revision = revision_for("");

        let document = save_agent_document_at(&path, &revision, "# Project\n")
            .expect("document should be saved");

        assert!(document.exists);
        assert_eq!(document.content, "# Project\n");
        assert_eq!(fs::read_to_string(&path).expect("file should exist"), "# Project\n");

        fs::remove_dir_all(workspace).expect("workspace should be removed");
    }

    #[test]
    fn rejects_stale_revision_on_save() {
        let workspace = temp_workspace();
        fs::create_dir_all(&workspace).expect("workspace should be created");
        let path = workspace.join(WORKSPACE_AGENT_DOCUMENT_FILE_NAME);
        let stale_revision = revision_for("");
        fs::write(&path, "external change").expect("external file should be written");

        let result = save_agent_document_at(&path, &stale_revision, "new content");

        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(&path).expect("file should remain readable"),
            "external change"
        );

        fs::remove_dir_all(workspace).expect("workspace should be removed");
    }

    #[test]
    fn rejects_oversized_content() {
        let content = "x".repeat((AGENT_DOCUMENT_MAX_BYTES + 1) as usize);

        assert!(validate_agent_content(&content).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_agent_document() {
        use std::os::unix::fs::symlink;

        let workspace = temp_workspace();
        fs::create_dir_all(&workspace).expect("workspace should be created");
        let target = workspace.join("target.md");
        let path = workspace.join(WORKSPACE_AGENT_DOCUMENT_FILE_NAME);
        File::create(&target)
            .and_then(|mut file| file.write_all(b"target"))
            .expect("target should be created");
        symlink(&target, &path).expect("symlink should be created");

        assert!(read_agent_document_at(&path).is_err());

        fs::remove_dir_all(workspace).expect("workspace should be removed");
    }
}
