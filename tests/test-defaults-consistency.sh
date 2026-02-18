#!/usr/bin/env bash
# Test suite: defaults.yaml consistency.
# Validates roster entries, council roles, and agent config blocks.
#
# Requires MODULE_ROOT env var.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

AGENTS_DIR="$MODULE_ROOT/agents"
DEFAULTS="$MODULE_ROOT/defaults.yaml"

echo "=== Defaults Consistency ==="

# --- Helpers (pure shell to avoid BSD awk sub() conflicts) ---

extract_roster() {
  local in_agents=0 in_list=0
  while IFS= read -r line; do
    if [ "$line" = "agents:" ]; then
      in_agents=1
      continue
    fi
    if [ "$in_agents" -eq 1 ]; then
      if echo "$line" | grep -qE '^  (council|standalone):'; then
        in_list=1
        continue
      fi
      if [ "$in_list" -eq 1 ]; then
        if echo "$line" | grep -qE '^    - '; then
          echo "$line" | sed 's/^    - //'
          continue
        fi
      fi
      if echo "$line" | grep -qE '^  [a-z]'; then
        in_list=0
        continue
      fi
      if echo "$line" | grep -qE '^[a-z]'; then
        in_agents=0
        in_list=0
      fi
    fi
  done < "$DEFAULTS"
}

extract_council_roles() {
  local target="$1"
  local in_councils=0 in_target=0 in_roles=0
  while IFS= read -r line; do
    if [ "$line" = "councils:" ]; then
      in_councils=1
      continue
    fi
    if [ "$in_councils" -eq 1 ]; then
      if [ "$line" = "  ${target}:" ]; then
        in_target=1
        continue
      fi
      if [ "$in_target" -eq 1 ]; then
        if [ "$line" = "    roles:" ]; then
          in_roles=1
          continue
        fi
        if [ "$in_roles" -eq 1 ]; then
          if echo "$line" | grep -qE '^      - '; then
            echo "$line" | sed 's/^      - //'
            continue
          else
            in_roles=0
          fi
        fi
        if echo "$line" | grep -qE '^  [a-z]'; then
          in_target=0
        fi
      fi
      if echo "$line" | grep -qE '^[a-z]'; then
        in_councils=0
      fi
    fi
  done < "$DEFAULTS"
}

extract_council_names() {
  local in_councils=0
  while IFS= read -r line; do
    if [ "$line" = "councils:" ]; then
      in_councils=1
      continue
    fi
    if [ "$in_councils" -eq 1 ]; then
      if echo "$line" | grep -qE '^  [a-z].*:$'; then
        echo "$line" | sed 's/^  //; s/:$//'
        continue
      fi
      if echo "$line" | grep -qE '^[a-z]'; then
        in_councils=0
      fi
    fi
  done < "$DEFAULTS"
}

has_config_block() {
  local agent_name="$1"
  local in_agent=0 has_model=0 has_tools=0
  while IFS= read -r line; do
    if [ "$line" = "${agent_name}:" ]; then
      in_agent=1
      continue
    fi
    if [ "$in_agent" -eq 1 ]; then
      if echo "$line" | grep -qE '^  model:'; then
        has_model=1
      fi
      if echo "$line" | grep -qE '^  tools:'; then
        has_tools=1
      fi
      if echo "$line" | grep -qE '^[^ ]'; then
        break
      fi
    fi
  done < "$DEFAULTS"
  if [ "$has_model" -eq 1 ] && [ "$has_tools" -eq 1 ]; then
    return 0
  fi
  return 1
}

# --- Test 1: Every roster agent has a file ---

roster_all_agents_exist() {
  while IFS= read -r name; do
    [ -n "$name" ] || continue
    assert_file_exists "roster agent $name exists" "$AGENTS_DIR/$name.md"
  done < <(extract_roster)
}

# --- Test 2: Council roles are roster members ---

council_roles_are_roster_members() {
  local roster
  roster=$(extract_roster)

  while IFS= read -r council_name; do
    [ -n "$council_name" ] || continue
    while IFS= read -r role; do
      [ -n "$role" ] || continue
      local found=0
      while IFS= read -r member; do
        if [ "$role" = "$member" ]; then
          found=1
          break
        fi
      done < <(echo "$roster")
      assert_eq "council '$council_name' role '$role' is in roster" "1" "$found"
    done < <(extract_council_roles "$council_name")
  done < <(extract_council_names)
}

# --- Test 3: Every roster member has a config block ---

agent_config_blocks_exist() {
  while IFS= read -r name; do
    [ -n "$name" ] || continue
    if has_config_block "$name"; then
      echo "  PASS: $name has config block (model + tools)"
      PASS=$((PASS + 1))
    else
      echo "  FAIL: $name missing config block (model + tools) in defaults.yaml"
      FAIL=$((FAIL + 1))
      ERRORS+=("$name missing config block")
    fi
  done < <(extract_roster)
}

# --- Run all ---

roster_all_agents_exist
council_roles_are_roster_members
agent_config_blocks_exist

report "Defaults Consistency"
