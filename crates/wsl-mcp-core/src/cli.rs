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

/// Decode bytes that may be UTF-16LE into a UTF-8 string.
///
/// `wsl.exe` on Windows outputs UTF-16LE for most management commands
/// (`--list`, `--status`, `--version`).  Sometimes a BOM (0xFF 0xFE) is
/// present, sometimes it is not.  When running inside WSL the output is
/// already UTF-8, so we use a heuristic: if the byte stream has even
/// length and every other byte is 0x00 (typical for ASCII-range UTF-16LE),
/// we decode as UTF-16LE.
pub fn decode_output(bytes: &[u8]) -> String {
    if let Some(decoded) = try_decode_utf16le(bytes) {
        decoded
    } else {
        String::from_utf8_lossy(bytes).to_string()
    }
}

/// Try to decode `bytes` as UTF-16LE, returning `Some` on success.
///
/// Handles three cases:
/// 1. Explicit BOM (0xFF 0xFE) — strip BOM and decode.
/// 2. No BOM but the data looks like UTF-16LE (even length, frequent
///    null bytes in the high-byte positions) — decode directly.
/// 3. Not UTF-16LE — return `None`.
fn try_decode_utf16le(bytes: &[u8]) -> Option<String> {
    let (payload, has_bom) = if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        (&bytes[2..], true)
    } else {
        (bytes, false)
    };

    if payload.len() < 2 || payload.len() % 2 != 0 {
        return if has_bom { Some(String::new()) } else { None };
    }

    // Heuristic: if ≥ 40% of the high bytes are 0x00, this is very likely
    // UTF-16LE (ASCII-range text). Plain UTF-8 almost never has this pattern.
    if !has_bom {
        let null_high_bytes = payload
            .chunks_exact(2)
            .filter(|chunk| chunk[1] == 0x00)
            .count();
        let total = payload.len() / 2;
        if null_high_bytes * 10 < total * 4 {
            return None;
        }
    }

    let u16_iter = payload
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]));
    let decoded: String = char::decode_utf16(u16_iter)
        .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect();
    Some(decoded.replace('\0', ""))
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

    #[test]
    fn test_decode_utf16le_no_bom() {
        // UTF-16LE without BOM: "Hello" = [H,0,e,0,l,0,l,0,o,0]
        let input: Vec<u8> = vec![b'H', 0, b'e', 0, b'l', 0, b'l', 0, b'o', 0];
        assert_eq!(decode_output(&input), "Hello");
    }

    #[test]
    fn test_decode_utf16le_no_bom_wsl_list() {
        // Simulates wsl --list --verbose output without BOM
        let text = "  NAME    STATE    VERSION\n* Ubuntu   Running  2\n";
        let input: Vec<u8> = text
            .encode_utf16()
            .flat_map(|u| u.to_le_bytes())
            .collect();
        let decoded = decode_output(&input);
        assert!(decoded.contains("Ubuntu"));
        assert!(decoded.contains("Running"));
        assert!(!decoded.contains('\0'));
    }
}
