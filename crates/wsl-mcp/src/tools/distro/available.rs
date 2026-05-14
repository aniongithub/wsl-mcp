use crate::tools::WslMcp;
use crate::tools::common::format_output;
use rmcp::{tool, tool_router};

#[tool_router(router = distro_available_router, vis = "pub(super)")]
impl WslMcp {
    #[tool(
        name = "wsl_available",
        description = "List available WSL distributions that can be installed from the Microsoft Store."
    )]
    async fn wsl_available(&self) -> String {
        match wsl_mcp_core::wsl::available().await {
            Ok(output) => format_output(&output),
            Err(e) => format!("Error: {e}"),
        }
    }
}
