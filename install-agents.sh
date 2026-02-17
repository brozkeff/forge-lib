#!/usr/bin/env bash
# Agent deployment utilities.
#
# Usage (as library):
#   source "$FORGE_LIB/frontmatter.sh"
#   source "$FORGE_LIB/install-agents.sh"
#   deploy_agents_from_dir "path/to/agents/" "$HOME/.claude/agents" [--dry-run]
#
# Usage (as script):
#   lib/install-agents.sh agents/ [--dry-run] [--clean]

# Deploy a single agent file to the destination directory.
deploy_agent() {
  local agent_file="$1"
  local dst_dir="$2"
  local dry_run="${3:-}"
  local basename_file
  basename_file="$(basename "$agent_file")"

  # Skip templates
  [[ "$basename_file" == _Template* ]] && return 1
  [[ "$basename_file" == Template* ]] && return 1

  local claude_name
  claude_name="$(fm_value "$agent_file" "claude.name")"
  if [ -z "$claude_name" ]; then
    return 1
  fi

  local claude_model claude_description claude_tools
  claude_model="$(fm_value "$agent_file" "claude.model")"
  claude_description="$(fm_value "$agent_file" "claude.description")"
  claude_tools="$(fm_value "$agent_file" "claude.tools")"

  # Fall back to generic description
  if [ -z "$claude_description" ]; then
    claude_description="$(fm_value "$agent_file" "description")"
  fi

  : "${claude_model:=sonnet}"
  : "${claude_description:=Specialist agent}"

  local out_file="$dst_dir/${claude_name}.md"
  local body
  body="$(fm_body "$agent_file")"

  local frontmatter="---
name: ${claude_name}
description: ${claude_description}
model: ${claude_model}"

  if [ -n "$claude_tools" ]; then
    frontmatter="${frontmatter}
tools: ${claude_tools}"
  fi

  frontmatter="${frontmatter}
---"

  local content="${frontmatter}
# synced-from: ${basename_file}

${body}"

  if [ "$dry_run" = "--dry-run" ]; then
    echo "[dry-run] Would install: ${claude_name}.md"
  else
    printf '%s\n' "$content" > "$out_file"
    echo "Installed: ${claude_name}.md"
  fi
  return 0
}

# Deploy all agent files from a directory.
deploy_agents_from_dir() {
  local src_dir="$1"
  local dst_dir="$2"
  local dry_run="${3:-}"
  local count=0

  [ -d "$src_dir" ] || return 0
  mkdir -p "$dst_dir"

  for agent_file in "$src_dir"/*.md; do
    [ -f "$agent_file" ] || continue
    if deploy_agent "$agent_file" "$dst_dir" "$dry_run"; then
      count=$((count + 1))
    fi
  done

  return 0
}

# Clean agents previously installed from the source directory.
clean_agents() {
  local src_dir="$1"
  local dst_dir="$2"
  local dry_run="${3:-}"

  [ -d "$src_dir" ] || return 0
  [ -d "$dst_dir" ] || return 0

  for agent_file in "$src_dir"/*.md; do
    [ -f "$agent_file" ] || continue
    local basename_file
    basename_file="$(basename "$agent_file")"
    local claude_name
    claude_name="$(fm_value "$agent_file" "claude.name")"

    if [ -n "$claude_name" ]; then
      local out_file="$dst_dir/${claude_name}.md"
      if [ -f "$out_file" ]; then
        if grep -q "^# synced-from: ${basename_file}$" "$out_file" 2>/dev/null; then
          if [ "$dry_run" = "--dry-run" ]; then
            echo "[dry-run] Would remove: ${claude_name}.md"
          else
            command rm "$out_file"
            echo "Removed: ${claude_name}.md"
          fi
        fi
      fi
    fi
  done
}

# CLI entry point
main() {
  local src_dir=""
  local dst_dir="${HOME}/.claude/agents"
  local dry_run=""
  local clean=false

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --dry-run) dry_run="--dry-run" ;;
      --clean)   clean=true ;;
      -h|--help)
        echo "Usage: install-agents.sh <src_dir> [--dry-run] [--clean]"
        exit 0
        ;;
      *)
        if [ -d "$1" ]; then
          src_dir="$1"
        else
          echo "Error: Invalid directory: $1"
          exit 1
        fi
        ;;
    esac
    shift
  done

  if [ -z "$src_dir" ]; then
    echo "Error: Source directory required."
    echo "Usage: install-agents.sh <src_dir> [--dry-run] [--clean]"
    exit 1
  fi

  # Source dependencies if not already available
  if ! type fm_value >/dev/null 2>&1; then
    local lib_dir
    lib_dir="$(dirname "${BASH_SOURCE[0]}")"
    if [ -f "$lib_dir/frontmatter.sh" ]; then
      source "$lib_dir/frontmatter.sh"
    else
      echo "Error: frontmatter.sh not found in $lib_dir"
      exit 1
    fi
  fi

  if $clean; then
    clean_agents "$src_dir" "$dst_dir" "$dry_run"
  fi

  deploy_agents_from_dir "$src_dir" "$dst_dir" "$dry_run"
}

# Run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  main "$@"
fi
