use serde::Serialize;
use std::process::Stdio;
use tokio::process::Command;

use crate::error::{Error, Result};

/// Raw output from a CLI invocation.
#[derive(Debug, Clone, Serialize)]
pub struct CliOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub json: Option<serde_json::Value>,
}

/// Decode bytes that may be UTF-16LE (with BOM) into a UTF-8 string.
///
/// `wsl.exe` on Windows outputs UTF-16LE with a BOM for most management
/// commands (`--list`, `--status`, `--version`).  When running inside WSL
/// the output is already UTF-8, so we try UTF-16LE only when a BOM is
/// present.
pub fn decode_output(bytes: &[u8]) -> String {
    // UTF-16LE BOM: 0xFF 0xFE
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        let u16_iter = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]));
        let decoded: String = char::decode_utf16(u16_iter)
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect();
        // Strip null characters that wsl.exe sometimes pads with
        decoded.replace('\0', "")
    } else {
        String::from_utf8_lossy(bytes).to_string()
    }
}

/// Run `wsl.exe` with the given arguments, capturing stdout/stderr/exit_code.
pub async fn run_wsl(args: &[&str], parse_json: bool) -> Result<CliOutput> {
    let output = Command::new("wsl.exe")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::WslNotFound
            } else {
                Error::Io(e)
            }
        })?;

    let stdout = decode_output(&output.stdout);
    let stderr = decode_output(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);

    let json = if parse_json {
        serde_json::from_str(&stdout).ok()
    } else {
        None
    };

    Ok(CliOutput {
        exit_code,
        stdout,
        stderr,
        json,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_utf8_passthrough() {
        let input = b"hello world";
        assert_eq!(decode_output(input), "hello world");
    }

    #[test]
    fn test_decode_utf16le_with_bom() {
        // UTF-16LE BOM + "Hi"
        let input: Vec<u8> = vec![0xFF, 0xFE, b'H', 0, b'i', 0];
        assert_eq!(decode_output(&input), "Hi");
    }

    #[test]
    fn test_decode_utf16le_strips_nulls() {
        // UTF-16LE BOM + "A\0" (null-padded)
        let input: Vec<u8> = vec![0xFF, 0xFE, b'A', 0, 0, 0];
        assert_eq!(decode_output(&input), "A");
    }
}
