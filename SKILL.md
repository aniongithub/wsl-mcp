---
name: wsl-mcp
description: Manage WSL distributions via MCP tools — distro lifecycle, command execution, file operations
tools:
  - wsl_list
  - wsl_status
  - wsl_install
  - wsl_unregister
  - wsl_set_default
  - wsl_import
  - wsl_export
  - wsl_available
  - wsl_exec
  - wsl_shell
  - wsl_exec_batch
  - wsl_file_read
  - wsl_file_write
  - wsl_file_edit
  - wsl_file_list
  - wsl_terminate
  - wsl_shutdown
  - wsl_set_version
  - wsl_system_info
  - wsl_disk_info
---

# WSL MCP Skill

You have access to `wsl-mcp`, an MCP server that manages WSL (Windows Subsystem for Linux) distributions.

## Core Rules

**Use ONLY the MCP tools listed here.** Do not invoke `wsl.exe` or `wsl` CLI commands directly — the MCP tools wrap the CLI with proper error handling, output decoding, and escaping. Direct CLI usage bypasses these safeguards.

**Always specify which distro to run in.** There is no implicit default — the agent must choose the target distribution for every command.

## Choosing a Distro

1. **List available distros** with `wsl_list` to see what's installed
2. **Ask the user** which distro to use if multiple are available
3. **Install a new one** with `wsl_install` if needed

## Workflow: Distro Management

### 1. List installed distros
```
wsl_list()
→ JSON array: [{ name, state, version, is_default }]
```

### 2. Check a specific distro
```
wsl_status(distro: "Ubuntu-24.04")
```

### 3. Install a new distro
```
wsl_install(distro: "Ubuntu-24.04")
```

### 4. Set the default distro
```
wsl_set_default(distro: "Ubuntu-24.04")
```

### 5. Import/Export distros
```
wsl_export(distro: "Ubuntu-24.04", file: "C:\\backups\\ubuntu.tar")
wsl_import(distro: "my-env", install_location: "C:\\WSL\\my-env", file: "C:\\backups\\ubuntu.tar")
```

### 6. List available distros from the store
```
wsl_available()
```

### 7. Unregister (delete) a distro — destructive!
```
wsl_unregister(distro: "old-distro")
```

## Workflow: Command Execution

### Run a command in a distro
```
wsl_exec(distro: "Ubuntu-24.04", command: "cargo build", workdir: "/home/user/project")
→ { exit_code, stdout, stderr }
```

### Run with full shell environment
Use `wsl_shell` when the command needs `.bashrc`, `.profile`, nvm, pyenv, etc.:
```
wsl_shell(distro: "Ubuntu-24.04", command: "source ~/.nvm/nvm.sh && node app.js")
```

### Run multiple commands sequentially
Stops on first failure — useful for setup sequences:
```
wsl_exec_batch(
    distro: "Ubuntu-24.04",
    commands: ["apt-get update", "apt-get install -y nodejs", "node --version"],
    user: "root"
)
→ { results: [...], success: bool }
```

## File Operations

**All operations target files inside WSL distros — no need to construct shell commands.**

### Reading files
```
wsl_file_read(distro: "Ubuntu-24.04", path: "/home/user/project/src/main.rs")
wsl_file_read(distro: "Ubuntu-24.04", path: "/home/user/project/src/main.rs", start_line: 10, end_line: 25)
```
Returns content with line numbers. Supports optional line range.

### Writing files
```
wsl_file_write(distro: "Ubuntu-24.04", path: "/home/user/project/new_file.rs", content: "fn main() {}")
```
Creates parent directories automatically.

### Editing files (surgical replacement)
```
wsl_file_edit(distro: "Ubuntu-24.04", path: "src/lib.rs", old_str: "fn old()", new_str: "fn new()")
```
`old_str` must match exactly once in the file. Include surrounding context to make it unique.

### Listing directories
```
wsl_file_list(distro: "Ubuntu-24.04", path: "/home/user/project/src")
```
Shows non-hidden files up to 2 levels deep.

### When to use file tools vs exec
- ✅ **Use file tools** for reading, writing, and editing source files
- ✅ **Use exec** for running builds, tests, and commands
- ❌ **Don't** construct `sed`, `cat`, or `echo` commands via exec for file editing

## Configuration & System Info

### Terminate a distro
```
wsl_terminate(distro: "Ubuntu-24.04")
```

### Shut down all of WSL — stops ALL distros
```
wsl_shutdown()
```

### Set WSL version for a distro
```
wsl_set_version(distro: "Ubuntu-24.04", version: 2)
```

### Get system info
```
wsl_system_info()
→ WSL version, kernel version, etc.
```

### Get disk usage
```
wsl_disk_info(distro: "Ubuntu-24.04")
→ df -h output for the distro's root filesystem
```

## Self-Healing

If `wsl_install` or `wsl_exec` returns errors:
1. Read the error output carefully
2. Fix the issue (install missing packages, correct paths, etc.)
3. Retry the command
4. Repeat until successful

## What NOT to do

- ❌ Do NOT run `wsl.exe` or `wsl` commands directly — use the MCP tools
- ❌ Do NOT assume a default distro — always specify which one
- ❌ Do NOT construct `sed`, `cat`, or `echo` commands for file editing — use file tools
- ✅ DO list distros first with `wsl_list`
- ✅ DO ask the user which distro to use
- ✅ DO use `wsl_exec` or `wsl_shell` for everything
- ✅ DO use file tools for reading, writing, and editing files
