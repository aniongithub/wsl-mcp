use serde::{Deserialize, Serialize};

use crate::cli::{run_wsl, CliOutput};
use crate::error::{Error, Result};

/// A WSL distribution entry from `wsl --list --verbose`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroInfo {
    pub name: String,
    pub state: String,
    pub version: u8,
    pub is_default: bool,
}

/// Parse the output of `wsl.exe --list --verbose`.
///
/// Format:
/// ```text
///   NAME                   STATE           VERSION
/// * Ubuntu-24.04           Running         2
///   Debian                 Stopped         2
/// ```
pub fn parse_list_verbose(output: &str) -> Vec<DistroInfo> {
    let mut distros = Vec::new();

    for line in output.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let is_default = trimmed.starts_with('*');
        let content = if is_default {
            trimmed[1..].trim()
        } else {
            trimmed
        };

        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let version = parts[parts.len() - 1].parse::<u8>().unwrap_or(2);
        let state = parts[parts.len() - 2].to_string();
        // Name may contain spaces — everything before the state
        let name = parts[..parts.len() - 2].join(" ");

        distros.push(DistroInfo {
            name,
            state,
            version,
            is_default,
        });
    }

    distros
}

// ---------------------------------------------------------------------------
// Distro queries
// ---------------------------------------------------------------------------

/// List all installed WSL distributions.
pub async fn list() -> Result<Vec<DistroInfo>> {
    let output = run_wsl(&["--list", "--verbose"], false).await?;
    if output.exit_code != 0 {
        return Err(Error::WslCommand {
            exit_code: output.exit_code,
            stderr: output.stderr,
        });
    }
    Ok(parse_list_verbose(&output.stdout))
}

/// Get status of a specific distribution.
pub async fn status(distro: &str) -> Result<Option<DistroInfo>> {
    let distros = list().await?;
    Ok(distros.into_iter().find(|d| d.name == distro))
}

/// List available distributions from the store.
pub async fn available() -> Result<CliOutput> {
    run_wsl(&["--list", "--online"], false).await
}

// ---------------------------------------------------------------------------
// Command execution
// ---------------------------------------------------------------------------

/// Execute a command inside a WSL distribution.
pub async fn exec(
    distro: &str,
    command: &str,
    workdir: Option<&str>,
    user: Option<&str>,
) -> Result<CliOutput> {
    let mut args: Vec<&str> = vec!["-d", distro];
    if let Some(dir) = workdir {
        args.push("--cd");
        args.push(dir);
    }
    if let Some(u) = user {
        args.push("-u");
        args.push(u);
    }
    args.push("--");
    args.push("sh");
    args.push("-c");
    args.push(command);
    run_wsl(&args, false).await
}

/// Execute a command in a login shell inside a WSL distribution.
pub async fn shell(distro: &str, command: &str, user: Option<&str>) -> Result<CliOutput> {
    let mut args: Vec<&str> = vec!["-d", distro];
    if let Some(u) = user {
        args.push("-u");
        args.push(u);
    }
    args.push("--");
    args.push("bash");
    args.push("-lc");
    args.push(command);
    run_wsl(&args, false).await
}

// ---------------------------------------------------------------------------
// Distro lifecycle
// ---------------------------------------------------------------------------

/// Install a new WSL distribution.
pub async fn install(distro: &str) -> Result<CliOutput> {
    run_wsl(&["--install", "-d", distro], false).await
}

/// Unregister (delete) a WSL distribution.
pub async fn unregister(distro: &str) -> Result<CliOutput> {
    run_wsl(&["--unregister", distro], false).await
}

/// Set the default WSL distribution.
pub async fn set_default(distro: &str) -> Result<CliOutput> {
    run_wsl(&["--set-default", distro], false).await
}

/// Import a WSL distribution from a tar file.
pub async fn import(
    distro: &str,
    install_location: &str,
    file: &str,
    version: Option<u8>,
) -> Result<CliOutput> {
    let mut args = vec!["--import", distro, install_location, file];
    let ver_str;
    if let Some(v) = version {
        ver_str = v.to_string();
        args.push("--version");
        args.push(&ver_str);
    }
    run_wsl(&args, false).await
}

/// Export a WSL distribution to a tar file.
pub async fn export(distro: &str, file: &str) -> Result<CliOutput> {
    run_wsl(&["--export", distro, file], false).await
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Terminate a running WSL distribution.
pub async fn terminate(distro: &str) -> Result<CliOutput> {
    run_wsl(&["--terminate", distro], false).await
}

/// Shut down the entire WSL subsystem.
pub async fn shutdown() -> Result<CliOutput> {
    run_wsl(&["--shutdown"], false).await
}

/// Set the WSL version for a distribution.
pub async fn set_version(distro: &str, version: u8) -> Result<CliOutput> {
    let ver_str = version.to_string();
    run_wsl(&["--set-version", distro, &ver_str], false).await
}

/// Get WSL system info (`wsl --version`).
pub async fn system_info() -> Result<CliOutput> {
    run_wsl(&["--version"], false).await
}

// ---------------------------------------------------------------------------
// File operations
// ---------------------------------------------------------------------------

/// Read a file from a WSL distribution.
pub async fn file_read(distro: &str, path: &str, user: Option<&str>) -> Result<CliOutput> {
    let cmd = crate::file_ops::read_file_command(path);
    exec(distro, &cmd, None, user).await
}

/// Write (create or overwrite) a file in a WSL distribution.
pub async fn file_write(
    distro: &str,
    path: &str,
    content: &str,
    user: Option<&str>,
) -> Result<CliOutput> {
    let cmd = crate::file_ops::write_file_command(path, content);
    exec(distro, &cmd, None, user).await
}

/// Surgical edit: replace exactly one occurrence of `old_str` with `new_str`.
pub async fn file_edit(
    distro: &str,
    path: &str,
    old_str: &str,
    new_str: &str,
    user: Option<&str>,
) -> Result<String> {
    let read_output = file_read(distro, path, user).await?;
    if read_output.exit_code != 0 {
        return Err(Error::FileRead(format!(
            "Failed to read {path}: {}",
            read_output.stderr.trim()
        )));
    }

    let modified = crate::file_ops::apply_edit(&read_output.stdout, old_str, new_str)?;

    let write_output = file_write(distro, path, &modified, user).await?;
    if write_output.exit_code != 0 {
        return Err(Error::FileEdit(format!(
            "Failed to write {path}: {}",
            write_output.stderr.trim()
        )));
    }

    Ok(format!("Edit applied to {path}"))
}

/// List directory contents in a WSL distribution.
pub async fn file_list(distro: &str, path: &str, user: Option<&str>) -> Result<CliOutput> {
    let cmd = crate::file_ops::list_dir_command(path);
    exec(distro, &cmd, None, user).await
}

/// Get disk usage for a WSL distribution.
pub async fn disk_info(distro: &str) -> Result<CliOutput> {
    exec(distro, "df -h /", None, None).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_list_verbose_basic() {
        let output = "\
  NAME                   STATE           VERSION
* Ubuntu-24.04           Running         2
  Debian                 Stopped         2
";
        let distros = parse_list_verbose(output);
        assert_eq!(distros.len(), 2);

        assert_eq!(distros[0].name, "Ubuntu-24.04");
        assert_eq!(distros[0].state, "Running");
        assert_eq!(distros[0].version, 2);
        assert!(distros[0].is_default);

        assert_eq!(distros[1].name, "Debian");
        assert_eq!(distros[1].state, "Stopped");
        assert_eq!(distros[1].version, 2);
        assert!(!distros[1].is_default);
    }

    #[test]
    fn test_parse_list_verbose_single() {
        let output = "\
  NAME      STATE           VERSION
* Ubuntu    Running         2
";
        let distros = parse_list_verbose(output);
        assert_eq!(distros.len(), 1);
        assert_eq!(distros[0].name, "Ubuntu");
        assert!(distros[0].is_default);
    }

    #[test]
    fn test_parse_list_verbose_empty() {
        let output = "  NAME      STATE           VERSION\n";
        let distros = parse_list_verbose(output);
        assert!(distros.is_empty());
    }

    #[test]
    fn test_parse_list_verbose_wsl1() {
        let output = "\
  NAME      STATE           VERSION
  Legacy    Stopped         1
";
        let distros = parse_list_verbose(output);
        assert_eq!(distros.len(), 1);
        assert_eq!(distros[0].version, 1);
        assert!(!distros[0].is_default);
    }
}
