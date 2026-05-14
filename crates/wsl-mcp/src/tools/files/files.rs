use crate::tools::WslMcp;
use crate::tools::common::format_output;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

// ---------------------------------------------------------------------------
// wsl_file_read
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslFileReadParams {
    #[schemars(description = "Name of the WSL distribution")]
    distro: String,
    #[schemars(description = "Path to the file inside the distribution")]
    path: String,
    #[serde(default)]
    #[schemars(description = "Start line number (1-based, inclusive)")]
    start_line: Option<usize>,
    #[serde(default)]
    #[schemars(description = "End line number (1-based, inclusive). Use -1 or omit for end of file.")]
    end_line: Option<i64>,
    #[serde(default)]
    #[schemars(description = "User to run as")]
    user: Option<String>,
}

#[tool_router(router = file_read_router, vis = "pub(super)")]
impl WslMcp {
    #[tool(
        name = "wsl_file_read",
        description = "Read file content from inside a WSL distribution. Returns content with line numbers. Supports optional line range."
    )]
    async fn wsl_file_read(&self, Parameters(params): Parameters<WslFileReadParams>) -> String {
        match wsl_mcp_core::wsl::file_read(&params.distro, &params.path, params.user.as_deref())
            .await
        {
            Ok(output) => {
                if output.exit_code != 0 {
                    return format_output(&output);
                }
                let end =
                    params.end_line.and_then(|e| if e < 0 { None } else { Some(e as usize) });
                wsl_mcp_core::file_ops::format_with_line_numbers(
                    &output.stdout,
                    params.start_line,
                    end,
                )
            }
            Err(e) => format!("Error: {e}"),
        }
    }
}

// ---------------------------------------------------------------------------
// wsl_file_write
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslFileWriteParams {
    #[schemars(description = "Name of the WSL distribution")]
    distro: String,
    #[schemars(description = "Path to the file inside the distribution")]
    path: String,
    #[schemars(description = "File content to write")]
    content: String,
    #[serde(default)]
    #[schemars(description = "User to run as")]
    user: Option<String>,
}

#[tool_router(router = file_write_router, vis = "pub(super)")]
impl WslMcp {
    #[tool(
        name = "wsl_file_write",
        description = "Create or overwrite a file in a WSL distribution. Creates parent directories automatically."
    )]
    async fn wsl_file_write(&self, Parameters(params): Parameters<WslFileWriteParams>) -> String {
        match wsl_mcp_core::wsl::file_write(
            &params.distro,
            &params.path,
            &params.content,
            params.user.as_deref(),
        )
        .await
        {
            Ok(output) => {
                if output.exit_code == 0 {
                    format!("File written to {}", params.path)
                } else {
                    format_output(&output)
                }
            }
            Err(e) => format!("Error: {e}"),
        }
    }
}

// ---------------------------------------------------------------------------
// wsl_file_edit
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslFileEditParams {
    #[schemars(description = "Name of the WSL distribution")]
    distro: String,
    #[schemars(description = "Path to the file inside the distribution")]
    path: String,
    #[schemars(description = "The exact string in the file to replace. Must match exactly once.")]
    old_str: String,
    #[schemars(description = "The new string to replace old_str with")]
    new_str: String,
    #[serde(default)]
    #[schemars(description = "User to run as")]
    user: Option<String>,
}

#[tool_router(router = file_edit_router, vis = "pub(super)")]
impl WslMcp {
    #[tool(
        name = "wsl_file_edit",
        description = "Make a surgical edit to a file in a WSL distribution. Replaces exactly one occurrence of old_str with new_str. The old_str must match exactly one location in the file — include enough surrounding context to make it unique."
    )]
    async fn wsl_file_edit(&self, Parameters(params): Parameters<WslFileEditParams>) -> String {
        match wsl_mcp_core::wsl::file_edit(
            &params.distro,
            &params.path,
            &params.old_str,
            &params.new_str,
            params.user.as_deref(),
        )
        .await
        {
            Ok(msg) => msg,
            Err(e) => format!("Error: {e}"),
        }
    }
}

// ---------------------------------------------------------------------------
// wsl_file_list
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslFileListParams {
    #[schemars(description = "Name of the WSL distribution")]
    distro: String,
    #[serde(default)]
    #[schemars(description = "Path to the directory inside the distribution (defaults to home directory)")]
    path: Option<String>,
    #[serde(default)]
    #[schemars(description = "User to run as")]
    user: Option<String>,
}

#[tool_router(router = file_list_router, vis = "pub(super)")]
impl WslMcp {
    #[tool(
        name = "wsl_file_list",
        description = "List directory contents in a WSL distribution. Shows non-hidden files up to 2 levels deep."
    )]
    async fn wsl_file_list(&self, Parameters(params): Parameters<WslFileListParams>) -> String {
        let path = params.path.as_deref().unwrap_or(".");
        match wsl_mcp_core::wsl::file_list(&params.distro, path, params.user.as_deref()).await {
            Ok(output) => {
                if output.exit_code == 0 {
                    output.stdout
                } else {
                    format_output(&output)
                }
            }
            Err(e) => format!("Error: {e}"),
        }
    }
}
