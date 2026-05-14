use crate::tools::WslMcp;
use crate::tools::common::format_output;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslUnregisterParams {
    #[schemars(description = "Name of the distribution to unregister. WARNING: This permanently deletes the distribution and its filesystem.")]
    distro: String,
}

#[tool_router(router = distro_unregister_router, vis = "pub(super)")]
impl WslMcp {
    #[tool(
        name = "wsl_unregister",
        description = "Unregister (delete) a WSL distribution and its entire filesystem. This is destructive and cannot be undone."
    )]
    async fn wsl_unregister(
        &self,
        Parameters(params): Parameters<WslUnregisterParams>,
    ) -> String {
        match wsl_mcp_core::wsl::unregister(&params.distro).await {
            Ok(output) => format_output(&output),
            Err(e) => format!("Error: {e}"),
        }
    }
}
