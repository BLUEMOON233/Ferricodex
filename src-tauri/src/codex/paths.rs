use std::path::{Component, Path, PathBuf};

pub(super) fn paths_match(left: &Path, right: &Path) -> bool {
    compare_key(left) == compare_key(right)
}

fn compare_key(path: &Path) -> String {
    let key = comparable_path(path).to_string_lossy().into_owned();

    #[cfg(windows)]
    {
        key.to_lowercase()
    }

    #[cfg(not(windows))]
    {
        key
    }
}

fn comparable_path(path: &Path) -> PathBuf {
    path.canonicalize()
        .unwrap_or_else(|_| normalize_lexically(path))
}

fn normalize_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push("..");
                }
            }
        }
    }

    normalized
}
