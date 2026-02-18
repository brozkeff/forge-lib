#!/usr/bin/env bash
# Test suite: Module structure.
# Validates basic module files and submodule state.
#
# Requires MODULE_ROOT env var.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

echo "=== Module Structure ==="

# --- Test 1: module.yaml exists with required keys ---

module_yaml_exists() {
  local yaml="$MODULE_ROOT/module.yaml"
  assert_file_exists "module.yaml exists" "$yaml"

  if [ -f "$yaml" ]; then
    for key in name version description; do
      local val
      val=$(awk -v key="$key" '
        $0 ~ "^" key ":" {
          val = $0
          sub("^" key ":[ ]*", "", val)
          gsub(/^["'\''"]|["'\''"]$/, "", val)
          print val
          exit
        }
      ' "$yaml")
      assert_not_empty "module.yaml has $key" "$val"
    done
  fi
}

# --- Test 2: plugin.json exists and is valid JSON ---

plugin_json_valid() {
  local pjson="$MODULE_ROOT/.claude-plugin/plugin.json"
  assert_file_exists "plugin.json exists" "$pjson"

  if [ -f "$pjson" ]; then
    if python3 -c "import json, sys; json.load(open(sys.argv[1]))" "$pjson" 2>/dev/null; then
      echo "  PASS: plugin.json is valid JSON"
      PASS=$((PASS + 1))
    else
      echo "  FAIL: plugin.json is not valid JSON"
      FAIL=$((FAIL + 1))
      ERRORS+=("plugin.json invalid JSON")
    fi
  fi
}

# --- Test 3: lib/frontmatter.sh exists (submodule initialized) ---

lib_submodule_initialized() {
  assert_file_exists "lib/frontmatter.sh exists" "$MODULE_ROOT/lib/frontmatter.sh"
}

# --- Run all ---

module_yaml_exists
plugin_json_valid
lib_submodule_initialized

report "Module Structure"
