use crate::tools::common::format_output;
use crate::tools::WslMcp;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslInstallParams {
    #[schemars(description = "Name of the distribution to install (e.g. \"Ubuntu-24.04\")")]
    distro: String,
}

#[tool_router(router = distro_install_router, vis = "pub(in crate::tools)")]
impl WslMcp {
    #[tool(
        name = "wsl_install",
        description = "Install a new WSL distribution from the Microsoft Store. Returns full output for self-healing on failure."
    )]
    async fn wsl_install(&self, Parameters(params): Parameters<WslInstallParams>) -> String {
        match wsl_mcp_core::wsl::install(&params.distro).await {
            Ok(output) => format_output(&output),
            Err(e) => format!("Error: {e}"),
        }
    }
}
