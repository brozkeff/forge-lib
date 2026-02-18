#!/usr/bin/env bash
# Test suite: Deploy parity.
# Deploys agents to temp dirs for Claude/Gemini/Codex and validates output.
#
# Requires MODULE_ROOT env var.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

AGENTS_DIR="$MODULE_ROOT/agents"
INSTALL_SCRIPT="$LIB_DIR/install-agents.sh"

echo "=== Deploy Parity ==="

# --- Setup: Deploy to temp dirs ---

TMPDIR_BASE=$(mktemp -d)
CLAUDE_DST="$TMPDIR_BASE/.claude/agents"
GEMINI_DST="$TMPDIR_BASE/.gemini/agents"
CODEX_DST="$TMPDIR_BASE/.codex/agents"

mkdir -p "$CLAUDE_DST" "$GEMINI_DST" "$CODEX_DST"

cleanup() {
  command rm -rf "$TMPDIR_BASE"
}
trap cleanup EXIT

# Deploy to all three provider dirs from the module root
(
  cd "$MODULE_ROOT"
  AGENTS_DST="$CLAUDE_DST" bash "$INSTALL_SCRIPT" "$AGENTS_DIR" > /dev/null 2>&1
  AGENTS_DST="$GEMINI_DST" bash "$INSTALL_SCRIPT" "$AGENTS_DIR" > /dev/null 2>&1
  AGENTS_DST="$CODEX_DST" bash "$INSTALL_SCRIPT" "$AGENTS_DIR" > /dev/null 2>&1
)

# --- Helper ---

count_md_files() {
  local dir="$1"
  local count=0
  for f in "$dir"/*.md; do
    if [ -f "$f" ]; then
      count=$((count + 1))
    fi
  done
  echo "$count"
}

# --- Test 1: Same agent count in all provider dirs ---

deploy_agent_count_parity() {
  local claude_count gemini_count codex_count
  claude_count=$(count_md_files "$CLAUDE_DST")
  gemini_count=$(count_md_files "$GEMINI_DST")
  codex_count=$(count_md_files "$CODEX_DST")

  assert_eq "claude count ($claude_count) == gemini count ($gemini_count)" \
    "$claude_count" "$gemini_count"
  assert_eq "claude count ($claude_count) == codex count ($codex_count)" \
    "$claude_count" "$codex_count"
}

# --- Test 2: Every deployed agent has synced-from header ---

deploy_all_have_synced_from() {
  for dir in "$CLAUDE_DST" "$GEMINI_DST" "$CODEX_DST"; do
    local provider
    provider=$(basename "$(dirname "$dir")")
    for f in "$dir"/*.md; do
      [ -f "$f" ] || continue
      local name
      name="$(basename "$f" .md)"
      if grep -q "^# synced-from:" "$f"; then
        echo "  PASS: $provider/$name has synced-from"
        PASS=$((PASS + 1))
      else
        echo "  FAIL: $provider/$name missing synced-from header"
        FAIL=$((FAIL + 1))
        ERRORS+=("$provider/$name missing synced-from")
      fi
    done
  done
}

# --- Test 3: Deployed body matches source agent body ---

deploy_body_matches_source() {
  for f in "$CLAUDE_DST"/*.md; do
    [ -f "$f" ] || continue
    local name
    name="$(basename "$f" .md)"
    local source_file="$AGENTS_DIR/$name.md"
    if [ ! -f "$source_file" ]; then
      continue
    fi

    local source_body deployed_body
    source_body="$(fm_body "$source_file")"
    # Deployed body: extract after frontmatter, strip synced-from header + separator blank line
    deployed_body=$(awk '/^---$/ { fm++; next } fm >= 2 { print }' "$f" \
      | sed '1{/^# synced-from:/d;}' \
      | sed '1{/^$/d;}')

    if [ "$source_body" = "$deployed_body" ]; then
      echo "  PASS: $name: deployed body matches source"
      PASS=$((PASS + 1))
    else
      echo "  FAIL: $name: deployed body differs from source"
      FAIL=$((FAIL + 1))
      ERRORS+=("$name: deployed body mismatch")
    fi
  done
}

# --- Test 4: Gemini agents have slugified name field ---

deploy_gemini_names_slugified() {
  for f in "$GEMINI_DST"/*.md; do
    [ -f "$f" ] || continue
    local filename
    filename="$(basename "$f" .md)"
    local gemini_name
    gemini_name="$(fm_value "$f" "name")"
    if echo "$gemini_name" | grep -qE '^[a-z][a-z0-9-]*$'; then
      echo "  PASS: $filename: gemini name '$gemini_name' is slugified"
      PASS=$((PASS + 1))
    else
      echo "  FAIL: $filename: gemini name '$gemini_name' is not slugified"
      FAIL=$((FAIL + 1))
      ERRORS+=("$filename: gemini name not slugified")
    fi
  done
}

# --- Test 5: Gemini agents have mapped tool names ---

deploy_gemini_tools_mapped() {
  local claude_tools="Read Write Edit Grep Glob Bash WebSearch WebFetch"
  for f in "$GEMINI_DST"/*.md; do
    [ -f "$f" ] || continue
    local filename
    filename="$(basename "$f" .md)"
    local has_claude_tool=0
    for tool in $claude_tools; do
      if awk '
        /^---$/ { fm++; next }
        fm == 1 && /^tools:/ { in_tools=1; next }
        fm == 1 && in_tools && /^  - / {
          val = $0
          sub("^[ ]*-[ ]*", "", val)
          gsub(/^[ ]+|[ ]+$/, "", val)
          if (val == "'"$tool"'") { found=1; exit }
        }
        fm == 1 && in_tools && /^[^ ]/ { in_tools=0 }
        fm >= 2 { exit }
        END { exit(found ? 0 : 1) }
      ' "$f" 2>/dev/null; then
        has_claude_tool=1
        break
      fi
    done
    if [ "$has_claude_tool" -eq 0 ]; then
      echo "  PASS: $filename: no unmapped Claude tool names in Gemini frontmatter"
      PASS=$((PASS + 1))
    else
      echo "  FAIL: $filename: unmapped Claude tool name found in Gemini frontmatter"
      FAIL=$((FAIL + 1))
      ERRORS+=("$filename: unmapped tool in Gemini")
    fi
  done
}

# --- Test 6: No deployed agent has model: fast or model: strong ---

deploy_model_tier_resolved() {
  for dir in "$CLAUDE_DST" "$GEMINI_DST" "$CODEX_DST"; do
    local provider
    provider=$(basename "$(dirname "$dir")")
    for f in "$dir"/*.md; do
      [ -f "$f" ] || continue
      local name model
      name="$(basename "$f" .md)"
      model="$(fm_value "$f" "model")"
      if [ "$model" = "fast" ] || [ "$model" = "strong" ]; then
        echo "  FAIL: $provider/$name: model '$model' not resolved"
        FAIL=$((FAIL + 1))
        ERRORS+=("$provider/$name: unresolved model tier '$model'")
      else
        echo "  PASS: $provider/$name: model '$model' resolved"
        PASS=$((PASS + 1))
      fi
    done
  done
}

# --- Run all ---

deploy_agent_count_parity
deploy_all_have_synced_from
deploy_body_matches_source
deploy_gemini_names_slugified
deploy_gemini_tools_mapped
deploy_model_tier_resolved

report "Deploy Parity"
