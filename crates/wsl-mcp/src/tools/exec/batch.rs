use crate::tools::WslMcp;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};

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

#[tool_router(router = exec_batch_router, vis = "pub(super)")]
impl WslMcp {
    #[tool(
        name = "wsl_exec_batch",
        description = "Execute multiple commands sequentially in a WSL distribution. Stops on first failure. Useful for setup sequences."
    )]
    async fn wsl_exec_batch(
        &self,
        Parameters(params): Parameters<WslExecBatchParams>,
    ) -> String {
        let mut results = Vec::new();
        let mut all_success = true;

        for cmd in &params.commands {
            match wsl_mcp_core::wsl::exec(&params.distro, cmd, None, params.user.as_deref()).await
            {
                Ok(output) => {
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
