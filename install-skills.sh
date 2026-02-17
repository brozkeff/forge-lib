#!/usr/bin/env bash
# install-skills.sh — Deploy agent skills to ~/.gemini/skills/
#
# Usage:
#   lib/install-skills.sh skills/ [--dry-run] [--scope user|workspace]

set -euo pipefail

# Deploy a single skill directory.
deploy_skill() {
  local skill_dir="$1"
  local dry_run="${2:-}"
  local scope="${3:-user}"

  if [ ! -d "$skill_dir" ]; then
    echo "Error: Skill directory not found: $skill_dir"
    return 1
  fi

  local skill_name
  skill_name=$(basename "$skill_dir")

  if [ "$dry_run" = "--dry-run" ]; then
    echo "[dry-run] Would install skill: $skill_name (scope: $scope)"
  else
    # We use the gemini CLI to install the skill
    echo "Installing skill: $skill_name..."
    if ! gemini skills install "$skill_dir" --scope "$scope" --yes; then
      echo "Error: Failed to install skill $skill_name"
      return 1
    fi
  fi
  return 0
}

# Deploy all skill directories from a parent directory.
deploy_skills_from_dir() {
  local parent_dir="$1"
  local dry_run="${2:-}"
  local scope="${3:-user}"

  if [ ! -d "$parent_dir" ]; then
    echo "Error: Skills parent directory not found: $parent_dir"
    return 1
  fi

  for skill_dir in "$parent_dir"/*; do
    if [ -d "$skill_dir" ] && [ -f "$skill_dir/SKILL.md" ]; then
      deploy_skill "$skill_dir" "$dry_run" "$scope"
    fi
  done
  return 0
}

# CLI entry point
main() {
  local skills_dir=""
  local dry_run=""
  local scope="user"

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --dry-run) dry_run="--dry-run" ;;
      --scope)
        shift
        scope="$1"
        ;;
      -h|--help)
        echo "Usage: install-skills.sh <skills_dir> [--dry-run] [--scope user|workspace]"
        exit 0
        ;;
      *)
        if [ -d "$1" ]; then
          skills_dir="$1"
        else
          echo "Error: Invalid directory: $1"
          exit 1
        fi
        ;;
    esac
    shift
  done

  if [ -z "$skills_dir" ]; then
    echo "Error: Skills directory required."
    echo "Usage: install-skills.sh <skills_dir> [--dry-run] [--scope user|workspace]"
    exit 1
  fi

  # Check if gemini CLI is available
  if ! command -v gemini >/dev/null 2>&1; then
    echo "Skipping skill installation (gemini CLI not found)"
    exit 0
  fi

  deploy_skills_from_dir "$skills_dir" "$dry_run" "$scope"
}

# Run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  main "$@"
fi
