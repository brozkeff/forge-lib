#!/usr/bin/env bash
# YAML frontmatter parsing utilities. Source this, don't execute directly.
#
# Usage:
#   source "$FORGE_LIB/frontmatter.sh"
#   value="$(fm_value file.md "claude.name")"
#   body="$(fm_body file.md)"

# Extract a single frontmatter value by key.
# Handles both quoted and unquoted values. Returns empty string if not found.
fm_value() {
  local file="$1" key="$2"
  awk -v key="$key" '
    /^---$/ { fm++; next }
    fm == 1 && $0 ~ "^" key ":" {
      sub("^" key ":[ ]*", "")
      gsub(/^["'\''"]|["'\''"]$/, "")
      gsub(/^[[:space:]]+|[[:space:]]+$/, "")
      print
      exit
    }
    fm >= 2 { exit }
  ' "$file"
}

# Extract a YAML list as a comma-separated string.
fm_list() {
  local file="$1" key="$2"
  awk -v key="$key" '
    /^---$/ { fm++; next }
    fm == 1 && $0 ~ "^" key ":" { in_list=1; next }
    in_list && fm == 1 && $0 ~ "^[ ]+-[ ]+" {
      sub("^[ ]+-[ ]+", "")
      gsub(/^[[:space:]]+|[[:space:]]+$/, "")
      printf "%s%s", (count++ ? ", " : ""), $0
      next
    }
    in_list && fm == 1 && $0 ~ "^[^ ]" { in_list=0 }
    fm >= 2 { exit }
    END { if (count) printf "\n" }
  ' "$file"
}

# Extract body (everything after second ---).
fm_body() {
  awk '
    /^---$/ { fm++; next }
    fm >= 2 { print }
  ' "$1"
}
