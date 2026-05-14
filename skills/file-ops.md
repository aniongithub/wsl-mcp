---
tags: [core]
order: 40
---
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
