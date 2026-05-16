# wsl-mcp installer for Windows
# Downloads the latest release binary and configures MCP clients.
#
# Usage:
#   irm https://github.com/aniongithub/wsl-mcp/releases/latest/download/install.ps1 | iex
#   .\install.ps1 -InstallDir "C:\tools\wsl-mcp"

[CmdletBinding()]
param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\wsl-mcp",
    [switch]$SkipMcpConfig
)

$ErrorActionPreference = "Stop"
$Repo = "aniongithub/wsl-mcp"

# ── Detect architecture ─────────────────────────────────────────

function Get-Platform {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64"  { return "windows-x64" }
        "x86"    { return "windows-x64" }
        "ARM64"  { return "windows-arm64" }
        default  { throw "Unsupported architecture: $arch" }
    }
}

# ── Get latest release tag ──────────────────────────────────────

function Get-LatestVersion {
    $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
    return $release.tag_name
}

# ── Main ────────────────────────────────────────────────────────

$Platform = Get-Platform
Write-Host "==> Detected platform: $Platform"

$Version = Get-LatestVersion
if (-not $Version) {
    Write-Error "Could not determine latest release version. Check: https://github.com/$Repo/releases"
    exit 1
}
Write-Host "==> Latest version: $Version"

$ZipName = "wsl-mcp-$Platform.zip"
$DownloadUrl = "https://github.com/$Repo/releases/download/$Version/$ZipName"

# Create install directory
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

# Download and extract
$TempZip = Join-Path $env:TEMP $ZipName
Write-Host "==> Downloading $ZipName..."
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZip -UseBasicParsing

Write-Host "==> Extracting to $InstallDir..."
Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
Remove-Item $TempZip -ErrorAction SilentlyContinue

$BinaryPath = Join-Path $InstallDir "wsl-mcp.exe"

# Verify
if (Test-Path $BinaryPath) {
    $ver = & $BinaryPath --version 2>$null
    Write-Host "==> Installed: $ver"
} else {
    Write-Warning "Binary downloaded but wsl-mcp.exe not found in $InstallDir"
}

# ── Add to PATH ─────────────────────────────────────────────────

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path = "$env:Path;$InstallDir"
    Write-Host "==> Added $InstallDir to user PATH"
} else {
    Write-Host "==> $InstallDir already in PATH"
}

# ── Install SKILL.md ────────────────────────────────────────────

$SkillUrl = "https://raw.githubusercontent.com/$Repo/main/SKILL.md"
$SkillDirs = @(
    "$env:USERPROFILE\.copilot\skills\wsl-mcp",
    "$env:USERPROFILE\.claude\skills\wsl-mcp",
    "$env:USERPROFILE\.agents\skills\wsl-mcp",
    "$env:LOCALAPPDATA\wsl-mcp"
)

Write-Host ""
Write-Host "==> Installing SKILL.md for agent discovery..."
foreach ($dir in $SkillDirs) {
    try {
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        Invoke-WebRequest -Uri $SkillUrl -OutFile (Join-Path $dir "SKILL.md") -UseBasicParsing
        Write-Host "    $dir\SKILL.md"
    } catch {
        Write-Host "    ! Could not install to $dir"
    }
}

# ── Install hooks ───────────────────────────────────────────────

$HookDir = "$env:LOCALAPPDATA\wsl-mcp\hooks"
New-Item -ItemType Directory -Force -Path $HookDir | Out-Null

$GuardUrl = "https://raw.githubusercontent.com/$Repo/main/.github/hooks/wsl-guard.sh"
$LoaderUrl = "https://raw.githubusercontent.com/$Repo/main/.github/hooks/wsl-skill-loader.sh"
$GuardPath = Join-Path $HookDir "wsl-guard.sh"
$LoaderPath = Join-Path $HookDir "wsl-skill-loader.sh"

Write-Host ""
Write-Host "==> Installing hooks..."
try {
    Invoke-WebRequest -Uri $GuardUrl -OutFile $GuardPath -UseBasicParsing
    Write-Host "    $GuardPath"
} catch { Write-Host "    ! Could not download guard hook" }

try {
    Invoke-WebRequest -Uri $LoaderUrl -OutFile $LoaderPath -UseBasicParsing
    Write-Host "    $LoaderPath"
} catch { Write-Host "    ! Could not download skill-loader hook" }

# ── Configure Copilot CLI hooks ─────────────────────────────────

$CopilotHooksDir = "$env:USERPROFILE\.copilot\hooks"

if (Test-Path "$env:USERPROFILE\.copilot") {
    New-Item -ItemType Directory -Force -Path $CopilotHooksDir | Out-Null

    # Guard hook
    @{
        version = 1
        hooks = @{
            preToolUse = @(
                @{
                    type = "command"
                    bash = $GuardPath.Replace('\', '/')
                    timeoutSec = 5
                }
            )
        }
    } | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $CopilotHooksDir "wsl-guard.json")
    Write-Host "  > Copilot CLI -- created wsl-guard.json"

    # Skill-loader hook
    @{
        version = 1
        hooks = @{
            sessionStart = @(
                @{
                    type = "command"
                    bash = $LoaderPath.Replace('\', '/')
                    timeoutSec = 5
                }
            )
        }
    } | ConvertTo-Json -Depth 5 | Set-Content (Join-Path $CopilotHooksDir "wsl-skill-loader.json")
    Write-Host "  > Copilot CLI -- created wsl-skill-loader.json"
}

# ── Configure Claude Code hooks ─────────────────────────────────

$ClaudeSettings = "$env:USERPROFILE\.claude\settings.json"

if (Test-Path "$env:USERPROFILE\.claude") {
    $settings = @{}
    if (Test-Path $ClaudeSettings) {
        try { $settings = Get-Content $ClaudeSettings -Raw | ConvertFrom-Json -AsHashtable } catch {}
    }

    if (-not $settings.ContainsKey("hooks")) { $settings["hooks"] = @{} }

    # PreToolUse guard
    $hasGuard = $false
    if ($settings.hooks.ContainsKey("PreToolUse")) {
        foreach ($group in $settings.hooks.PreToolUse) {
            foreach ($h in $group.hooks) {
                if ($h.command -like "*wsl-guard*") { $hasGuard = $true }
            }
        }
    } else {
        $settings.hooks["PreToolUse"] = @()
    }

    if (-not $hasGuard) {
        $settings.hooks.PreToolUse += @{
            matcher = "Bash"
            hooks = @(
                @{ type = "command"; command = $GuardPath.Replace('\', '/'); timeout = 5 }
            )
        }
        Write-Host "  > Claude Code -- added PreToolUse hook"
    } else {
        Write-Host "  > Claude Code -- PreToolUse hook already configured"
    }

    # SessionStart loader
    $hasLoader = $false
    if ($settings.hooks.ContainsKey("SessionStart")) {
        foreach ($group in $settings.hooks.SessionStart) {
            foreach ($h in $group.hooks) {
                if ($h.command -like "*wsl-skill-loader*") { $hasLoader = $true }
            }
        }
    } else {
        $settings.hooks["SessionStart"] = @()
    }

    if (-not $hasLoader) {
        $settings.hooks.SessionStart += @{
            hooks = @(
                @{ type = "command"; command = $LoaderPath.Replace('\', '/'); timeout = 5 }
            )
        }
        Write-Host "  > Claude Code -- added SessionStart hook"
    } else {
        Write-Host "  > Claude Code -- SessionStart hook already configured"
    }

    $settings | ConvertTo-Json -Depth 10 | Set-Content $ClaudeSettings
}

# ── Configure MCP clients ──────────────────────────────────────

if ($SkipMcpConfig) {
    Write-Host ""
    Write-Host "==> Skipping MCP client configuration (-SkipMcpConfig)"
    Write-Host ""
    Write-Host "Done! wsl-mcp is installed."
    exit 0
}

function Configure-McpClient {
    param([string]$ConfigFile, [string]$ClientName)

    $mcpServerEntry = @{
        command = $BinaryPath
        args    = @("serve")
        type    = "stdio"
    }

    try {
        if (Test-Path $ConfigFile) {
            $content = Get-Content -Raw $ConfigFile | ConvertFrom-Json
            if (-not $content.mcpServers) {
                $content | Add-Member -NotePropertyName "mcpServers" -NotePropertyValue ([PSCustomObject]@{})
            }
            if ($content.mcpServers.PSObject.Properties.Name -contains "wsl-mcp") {
                Write-Host "  > $ClientName -- already configured"
                return
            }
            $content.mcpServers | Add-Member -NotePropertyName "wsl-mcp" -NotePropertyValue ([PSCustomObject]$mcpServerEntry)
            $content | ConvertTo-Json -Depth 10 | Set-Content $ConfigFile -Encoding UTF8
            Write-Host "  > $ClientName -- added to $ConfigFile"
        } else {
            $dir = Split-Path $ConfigFile -Parent
            if ($dir) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
            $config = [PSCustomObject]@{
                mcpServers = [PSCustomObject]@{
                    "wsl-mcp" = [PSCustomObject]$mcpServerEntry
                }
            }
            $config | ConvertTo-Json -Depth 10 | Set-Content $ConfigFile -Encoding UTF8
            Write-Host "  > $ClientName -- created $ConfigFile"
        }
    } catch {
        Write-Host "  ! $ClientName -- could not update $ConfigFile"
    }
}

Write-Host ""
Write-Host "==> Configuring MCP clients..."

# Copilot CLI
if (Test-Path "$env:USERPROFILE\.copilot") {
    Configure-McpClient "$env:USERPROFILE\.copilot\mcp-config.json" "GitHub Copilot"
}

# VS Code
$VsCodeDir = "$env:APPDATA\Code\User"
if (Test-Path $VsCodeDir) {
    Configure-McpClient "$VsCodeDir\mcp.json" "VS Code"
}

# Cursor
if (Test-Path "$env:USERPROFILE\.cursor") {
    Configure-McpClient "$env:USERPROFILE\.cursor\mcp.json" "Cursor"
}

# Claude Code
Configure-McpClient "$env:USERPROFILE\.claude.json" "Claude Code"

# ── Check WSL ───────────────────────────────────────────────────

Write-Host ""
Write-Host "Prerequisites:"
try {
    $wslVer = wsl.exe --version 2>$null | Select-Object -First 1
    Write-Host "  > WSL: $wslVer"
} catch {
    Write-Host "  ! WSL not found -- install with: wsl --install"
}

Write-Host ""
Write-Host "Done! wsl-mcp is ready to use."
