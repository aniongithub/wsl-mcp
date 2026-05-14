#!/usr/bin/env bash
# wsl-guard.sh — PreToolUse hook for Claude Code & GitHub Copilot CLI
#
# Blocks bash/shell tool calls when the agent is operating inside a WSL
# context, forcing it to use wsl-mcp MCP tools (wsl_exec, wsl_file_read,
# etc.) instead of running commands directly on the Windows host.
#
# Detection: checks if cwd is under a WSL mount path (\\wsl$\, \\wsl.localhost\,
# /mnt/...) or if a .wsl-mcp.json marker exists in the project root.
#
# Read-only tools (view, grep, glob) and file edits are allowed through — only
# command execution is blocked.
#
# Host-safe commands (git, gh) are allowlisted and always permitted.
#
# Bypass: include USER_CONFIRMED_HOST_OPERATION=1 in the command.
#
# Supports both agent payload formats:
#   Claude Code:  { tool_name, tool_input, cwd, ... }
#   Copilot CLI:  { toolName, toolArgs, cwd, ... }

set -euo pipefail

INPUT=$(cat)

# --- Detect agent format and extract fields ---

TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // .toolName // empty')
CWD=$(echo "$INPUT" | jq -r '.cwd // empty')

# Only guard bash/shell tool calls — allow everything else through
case "$TOOL_NAME" in
  Bash|bash|shell|powershell|Shell|PowerShell) ;;
  *) exit 0 ;;
esac

TOOL_INPUT=$(echo "$INPUT" | jq -r '(.tool_input // .toolArgs // {}) | tostring')

# Check for the bypass string anywhere in the tool input
if echo "$TOOL_INPUT" | grep -q 'USER_CONFIRMED_HOST_OPERATION=1'; then
  exit 0
fi

# Check if we're in a WSL context
if [ -z "$CWD" ]; then
  exit 0
fi

# Detection strategies:
# 1. Marker file in project root
HAS_MARKER=false
if [ -f "${CWD}/.wsl-mcp.json" ]; then
  HAS_MARKER=true
fi

# 2. WSL mount path (Windows-side UNC paths converted)
IS_WSL_PATH=false
case "$CWD" in
  /mnt/wsl/*|\\\\wsl\$\\*|\\\\wsl.localhost\\*)
    IS_WSL_PATH=true
    ;;
esac

# 3. Running inside WSL (check for WSL interop)
IN_WSL=false
if [ -f /proc/sys/fs/binfmt_misc/WSLInterop ] || [ -n "${WSL_DISTRO_NAME:-}" ]; then
  IN_WSL=true
fi

# If none of the WSL indicators match, allow through
if [ "$HAS_MARKER" = false ] && [ "$IS_WSL_PATH" = false ] && [ "$IN_WSL" = false ]; then
  exit 0
fi

# --- WSL context detected: check allowlist before blocking ---

COMMAND=$(echo "$INPUT" | jq -r '(.tool_input.command // .toolArgs.command // "") | tostring')

# Commands that are safe to run on the host even in WSL context
ALLOWED_HOST_COMMANDS=(
  git
  gh
)

# Extract all meaningful commands from a shell string
all_commands() {
  local cmd="$1"
  while IFS= read -r segment; do
    segment="${segment#"${segment%%[![:space:]]*}"}"
    [ -z "$segment" ] && continue
    for token in $segment; do
      if [[ "$token" == *=* && "$token" != -* ]]; then
        continue
      fi
      case "$token" in
        cd|pushd|popd) break ;;
      esac
      basename "$token"
      break
    done
  done < <(echo "$cmd" | sed 's/ *&& */\n/g; s/ *|| */\n/g; s/ *; */\n/g; s/ *| */\n/g')
}

# Every command in the chain must be on the allowlist
ALL_ALLOWED=true
while IFS= read -r cmd_name; do
  [ -z "$cmd_name" ] && continue
  FOUND=false
  for allowed in "${ALLOWED_HOST_COMMANDS[@]}"; do
    if [ "$cmd_name" = "$allowed" ]; then
      FOUND=true
      break
    fi
  done
  if [ "$FOUND" = false ]; then
    ALL_ALLOWED=false
    break
  fi
done < <(all_commands "$COMMAND")

if [ "$ALL_ALLOWED" = true ] && [ -n "$(all_commands "$COMMAND")" ]; then
  exit 0
fi

# --- Not on the allowlist: block the tool call ---

DENY_REASON="Host execution blocked. You are in a WSL context. Use wsl-mcp tools (wsl_exec, wsl_shell, wsl_file_read, wsl_file_write, wsl_file_edit, wsl_file_list) instead of running commands directly on the host."

# Detect which agent format to use for the response
if echo "$INPUT" | jq -e '.tool_name // empty' >/dev/null 2>&1 && \
   [ -n "$(echo "$INPUT" | jq -r '.tool_name // empty')" ]; then
  # Claude Code format
  jq -n --arg reason "$DENY_REASON" '{
    hookSpecificOutput: {
      hookEventName: "PreToolUse",
      permissionDecision: "deny",
      permissionDecisionReason: $reason
    }
  }'
else
  # Copilot CLI format
  jq -n --arg reason "$DENY_REASON" '{
    permissionDecision: "deny",
    permissionDecisionReason: $reason
  }'
fi
