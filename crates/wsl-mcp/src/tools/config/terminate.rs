use crate::tools::common::format_output;
use crate::tools::WslMcp;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslTerminateParams {
    #[schemars(description = "Name of the WSL distribution to terminate")]
    distro: String,
}

#[tool_router(router = terminate_router, vis = "pub(in crate::tools)")]
impl WslMcp {
    #[tool(
        name = "wsl_terminate",
        description = "Terminate a running WSL distribution."
    )]
    async fn wsl_terminate(&self, Parameters(params): Parameters<WslTerminateParams>) -> String {
        match wsl_mcp_core::wsl::terminate(&params.distro).await {
            Ok(output) => format_output(&output),
            Err(e) => format!("Error: {e}"),
        }
    }
}
