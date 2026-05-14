#!/usr/bin/env bash
# wsl-skill-loader.sh — SessionStart hook for Claude Code & Copilot CLI
#
# When a session starts in a WSL context (inside WSL, or with a .wsl-mcp.json
# marker), injects the wsl-mcp SKILL.md content as additionalContext so the
# agent automatically knows how to use wsl-mcp tools.
#
# Supports both agent payload formats:
#   Claude Code:  { tool_name, tool_input, cwd, ... }
#   Copilot CLI:  { toolName, toolArgs, cwd, ... }

set -euo pipefail

INPUT=$(cat)

# Extract working directory from the payload
CWD=$(echo "$INPUT" | jq -r '.cwd // empty')

if [ -z "$CWD" ]; then
  exit 0
fi

# Check if we're in a WSL context
IN_WSL=false

# 1. Marker file
if [ -f "${CWD}/.wsl-mcp.json" ]; then
  IN_WSL=true
fi

# 2. Running inside WSL
if [ -f /proc/sys/fs/binfmt_misc/WSLInterop ] || [ -n "${WSL_DISTRO_NAME:-}" ]; then
  IN_WSL=true
fi

if [ "$IN_WSL" = false ]; then
  exit 0
fi

# Look for SKILL.md in order of preference
SKILL_PATH=""
SEARCH_PATHS=(
  "${HOME}/.local/share/wsl-mcp/SKILL.md"
  "${HOME}/.copilot/skills/wsl-mcp/SKILL.md"
  "${HOME}/.claude/skills/wsl-mcp/SKILL.md"
  "${HOME}/.agents/skills/wsl-mcp/SKILL.md"
)

for p in "${SEARCH_PATHS[@]}"; do
  if [ -f "$p" ]; then
    SKILL_PATH="$p"
    break
  fi
done

if [ -z "$SKILL_PATH" ]; then
  exit 0
fi

SKILL_CONTENT=$(cat "$SKILL_PATH")

jq -n --arg ctx "$SKILL_CONTENT" '{ "additionalContext": $ctx }'
