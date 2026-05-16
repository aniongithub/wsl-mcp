use crate::tools::WslMcp;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{NumberOrString, ProgressNotificationParam, ProgressToken};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, tool, tool_router};
use wsl_mcp_core::cli::OutputLine;

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct WslExecBatchParams {
    #[schemars(description = "Name of the WSL distribution")]
    distro: String,
    #[schemars(description = "Commands to execute sequentially (stops on first failure)")]
    commands: Vec<String>,
    #[serde(default)]
    #[schemars(description = "User to run as")]
    user: Option<String>,
}

#[tool_router(router = exec_batch_router, vis = "pub(in crate::tools)")]
impl WslMcp {
    #[tool(
        name = "wsl_exec_batch",
        description = "Execute multiple commands sequentially in a WSL distribution. Stops on first failure. Useful for setup sequences."
    )]
    async fn wsl_exec_batch(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<WslExecBatchParams>,
    ) -> String {
        let mut results = Vec::new();
        let mut all_success = true;
        let mut global_line_count: u64 = 0;

        for (cmd_idx, cmd) in params.commands.iter().enumerate() {
            // Notify which command is starting
            global_line_count += 1;
            let _ = ctx
                .peer
                .notify_progress(ProgressNotificationParam {
                    progress_token: ProgressToken(NumberOrString::Number(
                        global_line_count as i64,
                    )),
                    progress: (cmd_idx + 1) as f64,
                    total: Some(params.commands.len() as f64),
                    message: Some(format!("Running command {}: {cmd}", cmd_idx + 1)),
                })
                .await;

            match wsl_mcp_core::wsl::exec_streaming(
                &params.distro,
                cmd,
                None,
                params.user.as_deref(),
            )
            .await
            {
                Ok((mut rx, handle)) => {
                    // Stream progress for this command
                    while let Some(line) = rx.recv().await {
                        global_line_count += 1;
                        let msg = match &line {
                            OutputLine::Stdout(s) => s.clone(),
                            OutputLine::Stderr(s) => format!("[stderr] {s}"),
                        };
                        let _ = ctx
                            .peer
                            .notify_progress(ProgressNotificationParam {
                                progress_token: ProgressToken(NumberOrString::Number(
                                    global_line_count as i64,
                                )),
                                progress: (cmd_idx + 1) as f64,
                                total: Some(params.commands.len() as f64),
                                message: Some(msg),
                            })
                            .await;
                    }

                    match handle.await {
                        Ok(Ok(output)) => {
                            let success = output.exit_code == 0;
                            results.push(serde_json::json!({
                                "command": cmd,
                                "exit_code": output.exit_code,
                                "stdout": output.stdout,
                                "stderr": output.stderr,
                            }));
                            if !success {
                                all_success = false;
                                break;
                            }
                        }
                        Ok(Err(e)) => {
                            results.push(serde_json::json!({
                                "command": cmd,
                                "error": e.to_string(),
                            }));
                            all_success = false;
                            break;
                        }
                        Err(e) => {
                            results.push(serde_json::json!({
                                "command": cmd,
                                "error": format!("task join failed: {e}"),
                            }));
                            all_success = false;
                            break;
                        }
                    }
                }
                Err(e) => {
                    results.push(serde_json::json!({
                        "command": cmd,
                        "error": e.to_string(),
                    }));
                    all_success = false;
                    break;
                }
            }
        }

        serde_json::json!({
            "results": results,
            "success": all_success,
        })
        .to_string()
    }
}
