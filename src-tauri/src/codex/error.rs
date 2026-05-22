use std::error::Error;
use std::fmt;
use std::path::PathBuf;

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
