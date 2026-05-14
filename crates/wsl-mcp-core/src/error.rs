use thiserror::Error;

/// Unified error type for wsl-mcp-core.
#[derive(Debug, Error)]
pub enum Error {
    #[error("wsl.exe not found. WSL must be installed: https://learn.microsoft.com/en-us/windows/wsl/install")]
    WslNotFound,

    #[error("WSL command failed (exit code {exit_code}): {stderr}")]
    WslCommand { exit_code: i32, stderr: String },

    #[error("File read error: {0}")]
    FileRead(String),

    #[error("File edit error: {0}")]
    FileEdit(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
