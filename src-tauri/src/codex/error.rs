use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum CodexError {
    HomeNotFound(PathBuf),
    StateDbNotFound(PathBuf),
    HomeDirUnavailable,
    ThreadNotFound(String),
    ArchiveOperation(String),
    ArchiveDirectoryCreate {
        path: PathBuf,
        source: std::io::Error,
    },
    ArchiveFileMove {
        from: PathBuf,
        to: PathBuf,
        source: std::io::Error,
    },
    TrashOperation(String),
    TranscriptPathUnavailable,
    TranscriptRead {
        path: PathBuf,
        source: std::io::Error,
    },
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
            Self::ThreadNotFound(thread_id) => {
                write!(formatter, "Codex thread was not found: {thread_id}")
            }
            Self::ArchiveOperation(message) => {
                write!(formatter, "Could not change Codex archive state: {message}")
            }
            Self::ArchiveDirectoryCreate { path, source } => {
                write!(
                    formatter,
                    "Could not create archive directory {}: {source}",
                    path.display()
                )
            }
            Self::ArchiveFileMove { from, to, source } => {
                write!(
                    formatter,
                    "Could not move transcript from {} to {}: {source}",
                    from.display(),
                    to.display()
                )
            }
            Self::TrashOperation(message) => {
                write!(
                    formatter,
                    "Could not complete Codex Trash operation: {message}"
                )
            }
            Self::TranscriptPathUnavailable => {
                write!(formatter, "Transcript path is unavailable")
            }
            Self::TranscriptRead { path, source } => {
                write!(
                    formatter,
                    "Could not read transcript at {}: {source}",
                    path.display()
                )
            }
            Self::Sqlite(error) => {
                write!(formatter, "Could not read Codex state database: {error}")
            }
        }
    }
}

impl Error for CodexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ArchiveDirectoryCreate { source, .. } => Some(source),
            Self::ArchiveFileMove { source, .. } => Some(source),
            Self::TranscriptRead { source, .. } => Some(source),
            Self::Sqlite(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for CodexError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}
