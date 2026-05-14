use crate::tools::common::format_output;
use crate::tools::WslMcp;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslExecParams {
    #[schemars(description = "Name of the WSL distribution to run the command in")]
    distro: String,
    #[schemars(description = "Command to execute inside the distribution")]
    command: String,
    #[serde(default)]
    #[schemars(description = "Working directory inside the distribution")]
    workdir: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "User to run the command as (defaults to distribution's default user)"
    )]
    user: Option<String>,
}

#[tool_router(router = exec_router, vis = "pub(in crate::tools)")]
impl WslMcp {
    #[tool(
        name = "wsl_exec",
        description = "Execute a command inside a WSL distribution. Returns exit code, stdout, and stderr."
    )]
    async fn wsl_exec(&self, Parameters(params): Parameters<WslExecParams>) -> String {
        match wsl_mcp_core::wsl::exec(
            &params.distro,
            &params.command,
            params.workdir.as_deref(),
            params.user.as_deref(),
        )
        .await
        {
            Ok(output) => format_output(&output),
            Err(e) => format!("Error: {e}"),
        }
    }
}
