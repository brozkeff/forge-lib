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

# Slugify a string (lowercase, replace spaces/caps with hyphens)
slugify() {
  echo "$1" | sed -e 's/\([a-z0-9]\)\([A-Z]\)/\1-\2/g' \
                -e 's/[ _]/-/g' \
                -e 's/--*/-/g' \
                | tr '[:upper:]' '[:lower:]'
}

# Map Claude tools to Gemini tools
map_tools_to_gemini() {
  local tools_str="$1"
  local mapped=""
  IFS=', ' read -r -a tools_array <<< "$tools_str"
  for tool in "${tools_array[@]}"; do
    case "$(echo "$tool" | tr '[:upper:]' '[:lower:]')" in
      read)           mapped="${mapped}${mapped:+, }read_file" ;;
      write)          mapped="${mapped}${mapped:+, }write_file" ;;
      edit|replace)   mapped="${mapped}${mapped:+, }replace" ;;
      grep)           mapped="${mapped}${mapped:+, }grep_search" ;;
      glob)           mapped="${mapped}${mapped:+, }glob" ;;
      bash|shell|run) mapped="${mapped}${mapped:+, }run_shell_command" ;;
      websearch)      mapped="${mapped}${mapped:+, }google_web_search" ;;
      webfetch)       mapped="${mapped}${mapped:+, }web_fetch" ;;
      *)              mapped="${mapped}${mapped:+, }$(slugify "$tool")" ;;
    esac
  done
  echo "$mapped"
}

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
  
  # Try fm_list first for tools (in case they are defined as a list), fallback to fm_value
  claude_tools="$(fm_list "$agent_file" "claude.tools")"
  if [ -z "$claude_tools" ]; then
    claude_tools="$(fm_value "$agent_file" "claude.tools")"
  fi

  # Fall back to generic description
  if [ -z "$claude_description" ]; then
    claude_description="$(fm_value "$agent_file" "description")"
  fi

  : "${claude_model:=sonnet}"
  : "${claude_description:=Specialist agent}"

  # Model map: translate semantic tiers for cross-platform deployment.
  # Set MODEL_MAP_FAST / MODEL_MAP_STRONG env vars to override.
  if [ -n "${MODEL_MAP_FAST:-}" ] || [ -n "${MODEL_MAP_STRONG:-}" ]; then
    case "$claude_model" in
      sonnet) claude_model="${MODEL_MAP_FAST:-$claude_model}" ;;
      opus)   claude_model="${MODEL_MAP_STRONG:-$claude_model}" ;;
    esac
  fi

  local body
  body="$(fm_body "$agent_file")"
  local frontmatter=""
  local out_file=""

  # Append sidecar tools and model overrides from config.yaml or defaults.yaml
  local sidecar_file="defaults.yaml"
  if [ -f "config.yaml" ]; then
    sidecar_file="config.yaml"
  fi

  # Model Tiers
  local tier_fast tier_strong
  tier_fast=$(awk '/^models:/ { in_models=1; next } in_models && /^  fast:/ { sub("^  fast:[ ]*", "", $0); print; next } in_models && /^[^ ]/ { in_models=0 }' "$sidecar_file")
  tier_strong=$(awk '/^models:/ { in_models=1; next } in_models && /^  strong:/ { sub("^  strong:[ ]*", "", $0); print; next } in_models && /^[^ ]/ { in_models=0 }' "$sidecar_file")
  : "${tier_fast:=sonnet}"
  : "${tier_strong:=opus}"

  if [ -f "$sidecar_file" ]; then
    local sidecar_tools
    sidecar_tools=$(awk -v agent="$claude_name" '
      $0 ~ "^" agent ":" { in_target=1; next }
      in_target && /^[^ ]/ { in_target=0 }
      in_target && $0 ~ "^  tools:" {
        val = $0
        sub("^[ ]*tools:[ ]*", "", val)
        if (val != "") {
          print val
          in_target=0
        } else {
          in_tools=1
        }
        next
      }
      in_tools && $0 ~ "^    - " {
        sub("^[ ]*-[ ]*", "")
        printf "%s%s", (count++ ? ", " : ""), $0
        next
      }
      in_tools && $0 ~ "^  [^ ]" { in_tools=0; in_target=0 }
      END { if (count) printf "\n" }
    ' "$sidecar_file")

    if [ -n "$sidecar_tools" ]; then
      claude_tools="$sidecar_tools"
    fi

    local sidecar_model
    sidecar_model=$(awk -v agent="$claude_name" '
      $0 ~ "^" agent ":" { in_target=1; next }
      in_target && /^[^ ]/ { in_target=0 }
      in_target && $0 ~ "^  model:[ ]*" {
        sub("^[ ]*model:[ ]*", "", $0)
        print
        exit
      }
    ' "$sidecar_file")

    if [ -n "$sidecar_model" ]; then
      claude_model="$sidecar_model"
    fi
  fi

  # Resolve semantic tiers
  case "$claude_model" in
    fast)   claude_model="$tier_fast" ;;
    strong) claude_model="$tier_strong" ;;
  esac

  # Model map: translate semantic tiers for cross-platform deployment.
  # Set MODEL_MAP_FAST / MODEL_MAP_STRONG env vars to override.
  if [ -n "${MODEL_MAP_FAST:-}" ] || [ -n "${MODEL_MAP_STRONG:-}" ]; then
    case "$claude_model" in
      sonnet|$tier_fast) claude_model="${MODEL_MAP_FAST:-$claude_model}" ;;
      opus|$tier_strong)   claude_model="${MODEL_MAP_STRONG:-$claude_model}" ;;
    esac
  fi

  local body
  body="$(fm_body "$agent_file")"
  local frontmatter=""
  local out_file=""

  # Detect provider and format accordingly
  if [[ "$dst_dir" == *".gemini"* ]]; then
    local gemini_name
    gemini_name="$(slugify "$claude_name")"
    local gemini_tools
    gemini_tools="$(map_tools_to_gemini "$claude_tools")"
    
    out_file="$dst_dir/${claude_name}.md" # Keep original filename for sync tracking
    
    frontmatter="---
name: ${gemini_name}
description: ${claude_description}
kind: local
model: ${claude_model}
tools:"
    IFS=', ' read -r -a t_arr <<< "$gemini_tools"
    for t in "${t_arr[@]}"; do
      frontmatter="${frontmatter}
  - ${t}"
    done
    frontmatter="${frontmatter}
---"
  else
    # Default Claude format
    out_file="$dst_dir/${claude_name}.md"
    frontmatter="---
name: ${claude_name}
description: ${claude_description}
model: ${claude_model}"

    if [ -n "$claude_tools" ]; then
      frontmatter="${frontmatter}
tools: ${claude_tools}"
    fi
    frontmatter="${frontmatter}
---"
  fi

  local content="${frontmatter}
# synced-from: ${basename_file}

${body}"

  if [ "$dry_run" = "--dry-run" ]; then
    echo "[dry-run] Would install: ${claude_name}.md to $dst_dir"
  else
    printf '%s\n' "$content" > "$out_file"
    echo "Installed: ${claude_name}.md to $dst_dir"
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
  local dry_run=""
  local clean=false
  local scope="all"
  
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --dry-run) dry_run="--dry-run" ;;
      --clean)   clean=true ;;
      --scope)
        scope="$2"
        shift
        ;;
      -h|--help)
        echo "Usage: install-agents.sh <src_dir> [--dry-run] [--clean] [--scope user|workspace|all]"
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
    echo "Usage: install-agents.sh <src_dir> [--dry-run] [--clean] [--scope user|workspace|all]"
    exit 1
  fi

  # Supported provider directories
  local provider_dirs=()
  case "$scope" in
    user)
      provider_dirs=("${HOME}/.claude/agents" "${HOME}/.gemini/agents")
      ;;
    workspace)
      provider_dirs=(".gemini/agents")
      ;;
    all)
      provider_dirs=("${HOME}/.claude/agents" "${HOME}/.gemini/agents" ".gemini/agents")
      ;;
    *)
      echo "Error: Invalid scope '$scope'. Use user, workspace, or all."
      exit 1
      ;;
  esac
  
  # Use AGENTS_DST if provided, overriding defaults
  if [ -n "${AGENTS_DST:-}" ]; then
    provider_dirs=("$AGENTS_DST")
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

  for dst_dir in "${provider_dirs[@]}"; do
    echo "Targeting provider directory: $dst_dir"
    if $clean; then
      clean_agents "$src_dir" "$dst_dir" "$dry_run"
    fi
    deploy_agents_from_dir "$src_dir" "$dst_dir" "$dry_run"
  done
}
# Run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  main "$@"
fi
