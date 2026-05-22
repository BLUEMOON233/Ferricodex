use serde::Serialize;
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use super::home::expand_tilde;

const WORKSPACE_SCAN_ENTRY_LIMIT: u64 = 20_000;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
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
