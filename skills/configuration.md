---
tags: [core]
order: 50
---
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
