use crate::tools::common::format_output;
use crate::tools::WslMcp;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{NumberOrString, ProgressNotificationParam, ProgressToken};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, tool, tool_router};
use wsl_mcp_core::cli::OutputLine;

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslShellParams {
    #[schemars(description = "Name of the WSL distribution")]
    distro: String,
    #[schemars(
        description = "Command to execute in a login shell (loads .bashrc, .profile, etc.)"
    )]
    command: String,
    #[serde(default)]
    #[schemars(description = "User to run as")]
    user: Option<String>,
}

#[tool_router(router = shell_router, vis = "pub(in crate::tools)")]
impl WslMcp {
    #[tool(
        name = "wsl_shell",
        description = "Execute a command in a login shell inside a WSL distribution. Loads the full shell environment (.bashrc, .profile, etc.)."
    )]
    async fn wsl_shell(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<WslShellParams>,
    ) -> String {
        match wsl_mcp_core::wsl::shell_streaming(
            &params.distro,
            &params.command,
            params.user.as_deref(),
        )
        .await
        {
            Ok((mut rx, handle)) => {
                let mut line_count: u64 = 0;
                while let Some(line) = rx.recv().await {
                    line_count += 1;
                    let msg = match &line {
                        OutputLine::Stdout(s) => s.clone(),
                        OutputLine::Stderr(s) => format!("[stderr] {s}"),
                    };
                    let _ = ctx
                        .peer
                        .notify_progress(ProgressNotificationParam {
                            progress_token: ProgressToken(NumberOrString::Number(
                                line_count as i64,
                            )),
                            progress: line_count as f64,
                            total: None,
                            message: Some(msg),
                        })
                        .await;
                }
                match handle.await {
                    Ok(Ok(output)) => format_output(&output),
                    Ok(Err(e)) => format!("Error: {e}"),
                    Err(e) => format!("Error: task join failed: {e}"),
                }
            }
            Err(e) => format!("Error: {e}"),
        }
    }
}
