#!/usr/bin/env bash
# Test suite: Agent frontmatter validation.
# Validates all agents in agents/*.md against forge module conventions.
#
# Requires MODULE_ROOT env var.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

AGENTS_DIR="$MODULE_ROOT/agents"
DEFAULTS="$MODULE_ROOT/defaults.yaml"

echo "=== Agent Frontmatter ==="

# --- Helpers ---

roster_count() {
  local count=0
  local in_council=0
  while IFS= read -r line; do
    if echo "$line" | grep -qE "^  council:"; then
      in_council=1
      continue
    fi
    if [ "$in_council" -eq 1 ]; then
      if echo "$line" | grep -qE "^    - "; then
        count=$((count + 1))
      elif echo "$line" | grep -qE "^  [a-z]"; then
        in_council=0
      elif echo "$line" | grep -qE "^[a-z]"; then
        in_council=0
      fi
    fi
  done < "$DEFAULTS"

  local in_standalone=0
  while IFS= read -r line; do
    if echo "$line" | grep -qE "^  standalone:"; then
      in_standalone=1
      continue
    fi
    if [ "$in_standalone" -eq 1 ]; then
      if echo "$line" | grep -qE "^    - "; then
        count=$((count + 1))
      elif echo "$line" | grep -qE "^  [a-z]"; then
        in_standalone=0
      elif echo "$line" | grep -qE "^[a-z]"; then
        in_standalone=0
      fi
    fi
  done < "$DEFAULTS"

  echo "$count"
}

# --- Test 1: Agent count matches roster ---

agent_count_matches_roster() {
  local file_count=0
  for f in "$AGENTS_DIR"/*.md; do
    [ -f "$f" ] || continue
    file_count=$((file_count + 1))
  done

  local roster_total
  roster_total=$(roster_count)

  assert_eq "agent_count_matches_roster (files=$file_count, roster=$roster_total)" \
    "$roster_total" "$file_count"
}

# --- Test 2: Required frontmatter keys ---

agent_required_frontmatter_keys() {
  local required_keys="title description claude.name claude.model claude.description claude.tools"
  for f in "$AGENTS_DIR"/*.md; do
    [ -f "$f" ] || continue
    local name
    name="$(basename "$f" .md)"
    for key in $required_keys; do
      local val
      val="$(fm_value "$f" "$key")"
      assert_not_empty "$name has $key" "$val"
    done
  done
}

# --- Test 3: Filename matches claude.name ---

agent_filename_matches_claude_name() {
  for f in "$AGENTS_DIR"/*.md; do
    [ -f "$f" ] || continue
    local filename
    filename="$(basename "$f" .md)"
    local claude_name
    claude_name="$(fm_value "$f" "claude.name")"
    assert_eq "$filename: filename matches claude.name" "$filename" "$claude_name"
  done
}

# --- Test 4: claude.name is PascalCase ---

agent_claude_name_is_pascalcase() {
  for f in "$AGENTS_DIR"/*.md; do
    [ -f "$f" ] || continue
    local name
    name="$(fm_value "$f" "claude.name")"
    assert_match "$name is PascalCase" "$name" '^[A-Z][a-zA-Z0-9]+$'
  done
}

# --- Test 5: claude.model is valid ---

agent_model_is_valid() {
  local valid="sonnet opus haiku fast strong"
  for f in "$AGENTS_DIR"/*.md; do
    [ -f "$f" ] || continue
    local name model
    name="$(fm_value "$f" "claude.name")"
    model="$(fm_value "$f" "claude.model")"
    local found=0
    for v in $valid; do
      if [ "$model" = "$v" ]; then
        found=1
        break
      fi
    done
    assert_eq "$name: model '$model' is valid" "1" "$found"
  done
}

# --- Test 6: claude.description contains USE WHEN ---

agent_description_has_use_when() {
  for f in "$AGENTS_DIR"/*.md; do
    [ -f "$f" ] || continue
    local name desc
    name="$(fm_value "$f" "claude.name")"
    desc="$(fm_value "$f" "claude.description")"
    assert_contains "$name: description has USE WHEN" "$desc" "USE WHEN"
  done
}

# --- Test 7: Body has required sections ---

agent_body_has_required_sections() {
  local headings=("## Role" "## Expertise" "## Instructions" "## Output Format" "## Constraints")
  for f in "$AGENTS_DIR"/*.md; do
    [ -f "$f" ] || continue
    local name body
    name="$(fm_value "$f" "claude.name")"
    body="$(fm_body "$f")"
    for heading in "${headings[@]}"; do
      assert_contains "$name: has '$heading'" "$body" "$heading"
    done
  done
}

# --- Test 8: Constraints has honesty clause ---

agent_constraints_has_honesty_clause() {
  for f in "$AGENTS_DIR"/*.md; do
    [ -f "$f" ] || continue
    local name body
    name="$(fm_value "$f" "claude.name")"
    body="$(fm_body "$f")"
    assert_contains "$name: honesty clause (say so)" "$body" "say so"
  done
}

# --- Test 9: Constraints has team clause ---

agent_constraints_has_team_clause() {
  for f in "$AGENTS_DIR"/*.md; do
    [ -f "$f" ] || continue
    local name body
    name="$(fm_value "$f" "claude.name")"
    body="$(fm_body "$f")"
    assert_contains "$name: team clause (SendMessage)" "$body" "SendMessage"
  done
}

# --- Test 10: Blockquote has shipped-with marker ---

agent_blockquote_has_shipped_with() {
  for f in "$AGENTS_DIR"/*.md; do
    [ -f "$f" ] || continue
    local name body
    name="$(fm_value "$f" "claude.name")"
    body="$(fm_body "$f")"
    assert_contains "$name: shipped-with marker" "$body" "Shipped with forge-"
  done
}

# --- Run all ---

agent_count_matches_roster
agent_required_frontmatter_keys
agent_filename_matches_claude_name
agent_claude_name_is_pascalcase
agent_model_is_valid
agent_description_has_use_when
agent_body_has_required_sections
agent_constraints_has_honesty_clause
agent_constraints_has_team_clause
agent_blockquote_has_shipped_with

report "Agent Frontmatter"
