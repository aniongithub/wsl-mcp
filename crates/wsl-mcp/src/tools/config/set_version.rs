use crate::tools::WslMcp;
use crate::tools::common::format_output;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslSetVersionParams {
    #[schemars(description = "Name of the WSL distribution")]
    distro: String,
    #[schemars(description = "WSL version to set (1 or 2)")]
    version: u8,
}

#[tool_router(router = set_version_router, vis = "pub(super)")]
impl WslMcp {
    #[tool(
        name = "wsl_set_version",
        description = "Set the WSL version (1 or 2) for a specific distribution."
    )]
    async fn wsl_set_version(
        &self,
        Parameters(params): Parameters<WslSetVersionParams>,
    ) -> String {
        match wsl_mcp_core::wsl::set_version(&params.distro, params.version).await {
            Ok(output) => format_output(&output),
            Err(e) => format!("Error: {e}"),
        }
    }
}
