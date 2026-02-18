#!/usr/bin/env bash
# Test suite: Skill integrity.
# Validates skills/*/SKILL.{md,yaml} structure and content.
#
# Requires MODULE_ROOT env var.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

SKILLS_DIR="$MODULE_ROOT/skills"

echo "=== Skill Integrity ==="

# --- Test 1: Every skill has both SKILL.md and SKILL.yaml ---

skill_has_both_files() {
  for skill_dir in "$SKILLS_DIR"/*/; do
    [ -d "$skill_dir" ] || continue
    local name
    name="$(basename "$skill_dir")"
    assert_file_exists "$name has SKILL.md" "$skill_dir/SKILL.md"
    assert_file_exists "$name has SKILL.yaml" "$skill_dir/SKILL.yaml"
  done
}

# --- Test 2: SKILL.yaml has required keys ---

skill_yaml_required_keys() {
  local required="name description argument-hint"
  for skill_dir in "$SKILLS_DIR"/*/; do
    [ -d "$skill_dir" ] || continue
    local name
    name="$(basename "$skill_dir")"
    local yaml_file="$skill_dir/SKILL.yaml"
    [ -f "$yaml_file" ] || continue
    for key in $required; do
      local val
      val=$(awk -v key="$key" '
        $0 ~ "^" key ":" {
          val = $0
          sub("^" key ":[ ]*", "", val)
          gsub(/^["'\''"]|["'\''"]$/, "", val)
          print val
          exit
        }
      ' "$yaml_file")
      assert_not_empty "$name SKILL.yaml has $key" "$val"
    done
  done
}

# --- Test 3: SKILL.yaml name matches directory ---

skill_yaml_name_matches_directory() {
  for skill_dir in "$SKILLS_DIR"/*/; do
    [ -d "$skill_dir" ] || continue
    local dir_name
    dir_name="$(basename "$skill_dir")"
    local yaml_file="$skill_dir/SKILL.yaml"
    [ -f "$yaml_file" ] || continue
    local yaml_name
    yaml_name=$(awk '
      /^name:/ {
        val = $0
        sub("^name:[ ]*", "", val)
        gsub(/^["'\''"]|["'\''"]$/, "", val)
        print val
        exit
      }
    ' "$yaml_file")
    assert_eq "$dir_name: SKILL.yaml name matches directory" "$dir_name" "$yaml_name"
  done
}

# --- Test 4: SKILL.md has frontmatter with name and description ---

skill_md_has_frontmatter() {
  for skill_dir in "$SKILLS_DIR"/*/; do
    [ -d "$skill_dir" ] || continue
    local name
    name="$(basename "$skill_dir")"
    local md_file="$skill_dir/SKILL.md"
    [ -f "$md_file" ] || continue
    local fm_name
    fm_name="$(fm_value "$md_file" "name")"
    local fm_desc
    fm_desc="$(fm_value "$md_file" "description")"
    assert_not_empty "$name SKILL.md has name" "$fm_name"
    assert_not_empty "$name SKILL.md has description" "$fm_desc"
  done
}

# --- Test 5: Council skills (not Demo) contain "Gate Check" ---

skill_council_has_gate_check() {
  for skill_dir in "$SKILLS_DIR"/*/; do
    [ -d "$skill_dir" ] || continue
    local name
    name="$(basename "$skill_dir")"
    if [ "$name" = "Demo" ]; then
      continue
    fi
    local md_file="$skill_dir/SKILL.md"
    [ -f "$md_file" ] || continue
    local body
    body="$(fm_body "$md_file")"
    assert_contains "$name: has Gate Check" "$body" "Gate Check"
  done
}

# --- Test 6: Council skills (not Demo) contain "Sequential Fallback" ---

skill_council_has_sequential_fallback() {
  for skill_dir in "$SKILLS_DIR"/*/; do
    [ -d "$skill_dir" ] || continue
    local name
    name="$(basename "$skill_dir")"
    if [ "$name" = "Demo" ]; then
      continue
    fi
    local md_file="$skill_dir/SKILL.md"
    [ -f "$md_file" ] || continue
    local body
    body="$(fm_body "$md_file")"
    assert_contains "$name: has Sequential Fallback" "$body" "Sequential Fallback"
  done
}

# --- Run all ---

skill_has_both_files
skill_yaml_required_keys
skill_yaml_name_matches_directory
skill_md_has_frontmatter
skill_council_has_gate_check
skill_council_has_sequential_fallback

report "Skill Integrity"
