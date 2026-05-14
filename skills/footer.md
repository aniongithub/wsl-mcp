---
tags: [core]
order: 70
---
## What NOT to do

- ❌ Do NOT run `wsl.exe` or `wsl` commands directly — use the MCP tools
- ❌ Do NOT assume a default distro — always specify which one
- ❌ Do NOT construct `sed`, `cat`, or `echo` commands for file editing — use file tools
- ✅ DO list distros first with `wsl_list`
- ✅ DO ask the user which distro to use
- ✅ DO use `wsl_exec` or `wsl_shell` for everything
- ✅ DO use file tools for reading, writing, and editing files
