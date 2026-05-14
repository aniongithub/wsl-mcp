---
tags: [core]
order: 0
---
# WSL MCP Skill

You have access to `wsl-mcp`, an MCP server that manages WSL (Windows Subsystem for Linux) distributions.

## Core Rules

**Use ONLY the MCP tools listed here.** Do not invoke `wsl.exe` or `wsl` CLI commands directly — the MCP tools wrap the CLI with proper error handling, output decoding, and escaping. Direct CLI usage bypasses these safeguards.

**Always specify which distro to run in.** There is no implicit default — the agent must choose the target distribution for every command.
