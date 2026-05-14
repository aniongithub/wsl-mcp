use crate::tools::common::format_output;
use crate::tools::WslMcp;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslSetDefaultParams {
    #[schemars(description = "Name of the distribution to set as default")]
    distro: String,
}

#[tool_router(router = distro_set_default_router, vis = "pub(in crate::tools)")]
impl WslMcp {
    #[tool(
        name = "wsl_set_default",
        description = "Set the default WSL distribution."
    )]
    async fn wsl_set_default(&self, Parameters(params): Parameters<WslSetDefaultParams>) -> String {
        match wsl_mcp_core::wsl::set_default(&params.distro).await {
            Ok(output) => format_output(&output),
            Err(e) => format!("Error: {e}"),
        }
    }
}
