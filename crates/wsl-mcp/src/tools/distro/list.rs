use crate::tools::WslMcp;
use rmcp::{tool, tool_router};

#[tool_router(router = distro_list_router, vis = "pub(in crate::tools)")]
impl WslMcp {
    #[tool(
        name = "wsl_list",
        description = "List all installed WSL distributions with state, version, and default marker."
    )]
    async fn wsl_list(&self) -> String {
        match wsl_mcp_core::wsl::list().await {
            Ok(distros) => {
                serde_json::to_string_pretty(&distros).unwrap_or_else(|_| "[]".to_string())
            }
            Err(e) => format!("Error: {e}"),
        }
    }
}

use rmcp::handler::server::wrapper::Parameters;

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslStatusParams {
    #[schemars(description = "Name of the WSL distribution to check")]
    distro: String,
}

#[tool_router(router = distro_status_router, vis = "pub(in crate::tools)")]
impl WslMcp {
    #[tool(
        name = "wsl_status",
        description = "Get detailed status of a specific WSL distribution (state, version, default)."
    )]
    async fn wsl_status(&self, Parameters(params): Parameters<WslStatusParams>) -> String {
        match wsl_mcp_core::wsl::status(&params.distro).await {
            Ok(Some(info)) => {
                serde_json::to_string_pretty(&info).unwrap_or_else(|_| "{}".to_string())
            }
            Ok(None) => format!("Distribution '{}' not found", params.distro),
            Err(e) => format!("Error: {e}"),
        }
    }
}
