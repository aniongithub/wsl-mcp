use crate::tools::WslMcp;
use crate::tools::common::format_output;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

// ---------------------------------------------------------------------------
// wsl_system_info
// ---------------------------------------------------------------------------

#[tool_router(router = system_info_router, vis = "pub(super)")]
impl WslMcp {
    #[tool(
        name = "wsl_system_info",
        description = "Get WSL system information (version, kernel version, etc.)."
    )]
    async fn wsl_system_info(&self) -> String {
        match wsl_mcp_core::wsl::system_info().await {
            Ok(output) => format_output(&output),
            Err(e) => format!("Error: {e}"),
        }
    }
}

// ---------------------------------------------------------------------------
// wsl_disk_info
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslDiskInfoParams {
    #[schemars(description = "Name of the WSL distribution to check disk usage for")]
    distro: String,
}

#[tool_router(router = disk_info_router, vis = "pub(super)")]
impl WslMcp {
    #[tool(
        name = "wsl_disk_info",
        description = "Get disk usage information for a WSL distribution."
    )]
    async fn wsl_disk_info(&self, Parameters(params): Parameters<WslDiskInfoParams>) -> String {
        match wsl_mcp_core::wsl::disk_info(&params.distro).await {
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
