use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum CodexError {
    HomeNotFound(PathBuf),
    StateDbNotFound(PathBuf),
    HomeDirUnavailable,
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
