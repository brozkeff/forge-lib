#!/usr/bin/env bash
# Generate codex-compatible specialist skills from agents/*.md
#
# Usage:
#   lib/generate-agent-skills.sh <agents_dir> <out_dir> [--dry-run]

set -euo pipefail

SCRIPT_DIR="$(builtin cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FORGE_LIB="${FORGE_LIB:-$SCRIPT_DIR}"

if [ ! -f "$FORGE_LIB/frontmatter.sh" ]; then
  echo "Error: frontmatter.sh not found in $FORGE_LIB"
  exit 1
fi

source "$FORGE_LIB/frontmatter.sh"

yaml_escape() {
  echo "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

generate_skill_from_agent() {
  local agent_file="$1"
  local out_dir="$2"
  local dry_run="${3:-}"

  local file_name
  file_name="$(basename "$agent_file")"

  local agent_name
  agent_name="$(fm_value "$agent_file" "claude.name")"
  if [ -z "$agent_name" ]; then
    agent_name="$(fm_value "$agent_file" "title")"
  fi

  if [ -z "$agent_name" ]; then
    echo "Skipping $file_name: missing claude.name/title"
    return 0
  fi

  local agent_desc
  agent_desc="$(fm_value "$agent_file" "claude.description")"
  if [ -z "$agent_desc" ]; then
    agent_desc="$(fm_value "$agent_file" "description")"
  fi
  : "${agent_desc:=Specialist skill}"

  local desc_escaped
  desc_escaped="$(yaml_escape "$agent_desc")"

  local skill_dir="$out_dir/$agent_name"
  local skill_md="$skill_dir/SKILL.md"
  local skill_yaml="$skill_dir/SKILL.yaml"
  local body
  body="$(fm_body "$agent_file")"

  if [ "$dry_run" = "--dry-run" ]; then
    echo "[dry-run] Would generate skill wrapper: $agent_name from $file_name"
    return 0
  fi

  mkdir -p "$skill_dir"

  cat > "$skill_md" <<SKILL_MD
---
name: $agent_name
description: "$desc_escaped"
argument-hint: "[task, files, or question for $agent_name]"
---

# $agent_name

> Generated from agents/$file_name. Do not edit manually.

Use the specialist guidance below to handle the user's request.

$body
SKILL_MD

  cat > "$skill_yaml" <<SKILL_YAML
name: $agent_name
description: "$desc_escaped"
argument-hint: "[task, files, or question for $agent_name]"
providers:
  claude:
    enabled: false
  gemini:
    enabled: false
  codex:
    enabled: true
generation:
  source: generated-from-agent
  agent: $agent_name
  synced-from: $file_name
SKILL_YAML

  echo "Generated wrapper skill: $agent_name"
}

main() {
  local agents_dir=""
  local out_dir=""
  local dry_run=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --dry-run) dry_run="--dry-run" ;;
      -h|--help)
        echo "Usage: generate-agent-skills.sh <agents_dir> <out_dir> [--dry-run]"
        exit 0
        ;;
      *)
        if [ -z "$agents_dir" ]; then
          agents_dir="$1"
        elif [ -z "$out_dir" ]; then
          out_dir="$1"
        else
          echo "Error: Unexpected argument: $1"
          exit 1
        fi
        ;;
    esac
    shift
  done

  if [ -z "$agents_dir" ] || [ -z "$out_dir" ]; then
    echo "Error: agents_dir and out_dir are required"
    echo "Usage: generate-agent-skills.sh <agents_dir> <out_dir> [--dry-run]"
    exit 1
  fi

  if [ ! -d "$agents_dir" ]; then
    echo "Error: agents directory not found: $agents_dir"
    exit 1
  fi

  mkdir -p "$out_dir"

  local agent_file
  for agent_file in "$agents_dir"/*.md; do
    [ -f "$agent_file" ] || continue
    generate_skill_from_agent "$agent_file" "$out_dir" "$dry_run"
  done
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  main "$@"
fi
