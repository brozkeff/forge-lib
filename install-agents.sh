#!/usr/bin/env bash
# Agent deployment utilities. Source this, don't execute directly.
#
# Requires: frontmatter.sh (fm_value, fm_body)
#
# Usage:
#   source "$FORGE_LIB/frontmatter.sh"
#   source "$FORGE_LIB/install-agents.sh"
#   deploy_agent "path/to/agent.md" "$HOME/.claude/agents" [--dry-run]
#   deploy_agents_from_dir "path/to/agents/" "$HOME/.claude/agents" [--dry-run]

# Deploy a single agent file to the destination directory.
# Reads claude.* frontmatter, writes Claude Code agent format.
# Returns 0 on success, 1 if skipped (no claude.name).
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
# Returns the number of agents deployed.
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

  echo "$count"
}
