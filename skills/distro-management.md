---
tags: [core]
order: 20
---
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
