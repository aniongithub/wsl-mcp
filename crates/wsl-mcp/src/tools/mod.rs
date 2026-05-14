pub mod common;
mod config;
mod distro;
mod exec;
mod files;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::{tool_handler, ServerHandler};

#[derive(Debug, Clone)]
pub struct WslMcp;

impl WslMcp {
    pub fn new() -> Self {
        Self
    }

    fn combined_router() -> ToolRouter<Self> {
        Self::distro_list_router()
            + Self::distro_status_router()
            + Self::distro_install_router()
            + Self::distro_unregister_router()
            + Self::distro_set_default_router()
            + Self::distro_import_router()
            + Self::distro_export_router()
            + Self::distro_available_router()
            + Self::exec_router()
            + Self::shell_router()
            + Self::exec_batch_router()
            + Self::file_read_router()
            + Self::file_write_router()
            + Self::file_edit_router()
            + Self::file_list_router()
            + Self::terminate_router()
            + Self::shutdown_router()
            + Self::set_version_router()
            + Self::system_info_router()
            + Self::disk_info_router()
    }
}

#[tool_handler(router = Self::combined_router())]
impl ServerHandler for WslMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(rmcp::model::Implementation::new(
                "wsl-mcp",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "WSL MCP — a unified MCP server for managing WSL distributions. \
                 Provides tools for distro lifecycle management (wsl_list, wsl_install, \
                 wsl_unregister, etc.), command execution (wsl_exec, wsl_shell), \
                 file operations (wsl_file_read/write/edit/list), and system \
                 configuration (wsl_terminate, wsl_shutdown, wsl_set_version).",
            )
    }
}
