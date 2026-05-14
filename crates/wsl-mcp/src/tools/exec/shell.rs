use crate::tools::common::format_output;
use crate::tools::WslMcp;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslShellParams {
    #[schemars(description = "Name of the WSL distribution")]
    distro: String,
    #[schemars(
        description = "Command to execute in a login shell (loads .bashrc, .profile, etc.)"
    )]
    command: String,
    #[serde(default)]
    #[schemars(description = "User to run as")]
    user: Option<String>,
}

#[tool_router(router = shell_router, vis = "pub(in crate::tools)")]
impl WslMcp {
    #[tool(
        name = "wsl_shell",
        description = "Execute a command in a login shell inside a WSL distribution. Loads the full shell environment (.bashrc, .profile, etc.)."
    )]
    async fn wsl_shell(&self, Parameters(params): Parameters<WslShellParams>) -> String {
        match wsl_mcp_core::wsl::shell(&params.distro, &params.command, params.user.as_deref())
            .await
        {
            Ok(output) => format_output(&output),
            Err(e) => format!("Error: {e}"),
        }
    }
}
