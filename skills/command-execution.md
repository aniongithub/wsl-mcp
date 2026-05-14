---
tags: [core]
order: 30
---
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
