#!/usr/bin/env bash
# lib/sync-rosters.sh -- Sync council definitions from defaults.yaml to SKILL.md files.
#
# Usage:
#   lib/sync-rosters.sh [defaults.yaml]

set -euo pipefail

DEFAULTS_FILE="${1:-defaults.yaml}"

if [ ! -f "$DEFAULTS_FILE" ]; then
  echo "Error: $DEFAULTS_FILE not found."
  exit 1
fi

# Simple YAML parser for the 'councils' section using awk
get_council_roles() {
  local council="$1"
  awk -v council="$council" '
    /^councils:/ { in_councils=1; next }
    in_councils && $0 ~ "^  " council ":" { in_target=1; next }
    in_target && /^  [^ ]/ { in_target=0; in_councils=0 }
    in_target && $0 ~ "^      - " {
      sub("^      - ", "")
      print
    }
  ' "$DEFAULTS_FILE"
}

update_skill_md() {
  local council_name="$1"
  
  # Portable capitalization for PascalCase directory matching
  local cap_council
  cap_council=$(echo "$council_name" | awk '{print toupper(substr($0,1,1)) substr($0,2)}')
  local skill_md="skills/${cap_council}Council/SKILL.md"
  
  if [ ! -f "$skill_md" ]; then
    skill_md="skills/${council_name}Council/SKILL.md"
  fi

  # Generic Council is in skills/Council/SKILL.md
  if [ "$council_name" == "generic" ]; then
    skill_md="skills/Council/SKILL.md"
  fi

  if [ ! -f "$skill_md" ]; then
    return 0
  fi

  local roles
  roles=$(get_council_roles "$council_name")
  if [ -z "$roles" ]; then
    return 0
  fi

  echo "Syncing $skill_md..."
  
  # Prepare the replacement text for the specialists section
  local role_list=""
  while read -r role; do
    role_list="${role_list}${role_list:+, }$role"
  done <<< "$roles"

  # Update the "Default (always)" or "roles" mention in Step 3
  # Use a different delimiter to avoid issues with special characters
  sed -i.bak "s/\*\*Default (always)\*\*: .*/\*\*Default (always)\*\*: $role_list/" "$skill_md"
  rm "${skill_md}.bak"
}

# Run for known councils
update_skill_md "developer"
update_skill_md "product"
update_skill_md "generic"
update_skill_md "knowledge"

echo "Roster synchronization complete."
