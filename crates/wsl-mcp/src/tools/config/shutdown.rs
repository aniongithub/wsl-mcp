use crate::tools::common::format_output;
use crate::tools::WslMcp;
use rmcp::{tool, tool_router};

#[tool_router(router = shutdown_router, vis = "pub(in crate::tools)")]
impl WslMcp {
    #[tool(
        name = "wsl_shutdown",
        description = "Shut down the entire WSL subsystem. WARNING: This stops ALL running distributions."
    )]
    async fn wsl_shutdown(&self) -> String {
        match wsl_mcp_core::wsl::shutdown().await {
            Ok(output) => format_output(&output),
            Err(e) => format!("Error: {e}"),
        }
    }
}
