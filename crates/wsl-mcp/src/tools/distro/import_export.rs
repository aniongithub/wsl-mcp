use crate::tools::common::format_output;
use crate::tools::WslMcp;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslImportParams {
    #[schemars(description = "Name for the new distribution")]
    distro: String,
    #[schemars(description = "Directory where the distribution will be installed")]
    install_location: String,
    #[schemars(description = "Path to the tar file to import")]
    file: String,
    #[serde(default)]
    #[schemars(description = "WSL version (1 or 2). Defaults to 2 if not specified.")]
    version: Option<u8>,
}

#[tool_router(router = distro_import_router, vis = "pub(in crate::tools)")]
impl WslMcp {
    #[tool(
        name = "wsl_import",
        description = "Import a WSL distribution from a tar file."
    )]
    async fn wsl_import(&self, Parameters(params): Parameters<WslImportParams>) -> String {
        match wsl_mcp_core::wsl::import(
            &params.distro,
            &params.install_location,
            &params.file,
            params.version,
        )
        .await
        {
            Ok(output) => format_output(&output),
            Err(e) => format!("Error: {e}"),
        }
    }
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslExportParams {
    #[schemars(description = "Name of the distribution to export")]
    distro: String,
    #[schemars(description = "Path where the tar file will be saved")]
    file: String,
}

#[tool_router(router = distro_export_router, vis = "pub(in crate::tools)")]
impl WslMcp {
    #[tool(
        name = "wsl_export",
        description = "Export a WSL distribution to a tar file."
    )]
    async fn wsl_export(&self, Parameters(params): Parameters<WslExportParams>) -> String {
        match wsl_mcp_core::wsl::export(&params.distro, &params.file).await {
            Ok(output) => format_output(&output),
            Err(e) => format!("Error: {e}"),
        }
    }
}
