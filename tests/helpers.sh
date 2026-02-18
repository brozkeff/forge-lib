#!/usr/bin/env bash
# Shared test infrastructure for forge module architecture tests.
# Source this file — do not execute directly.
#
# Requires MODULE_ROOT env var pointing to the consuming module.

set -euo pipefail

if [ -z "${MODULE_ROOT:-}" ]; then
  echo "ERROR: MODULE_ROOT env var is required" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Inline frontmatter parsing (previously sourced from frontmatter.sh)
fm_value() {
  local file="$1" key="$2"
  awk -v key="$key" '
    /^---$/ { if (++fm==2) exit; next }
    fm==1 && $0 ~ "^" key ":[ ]*" {
      val=$0; sub("^" key ":[ ]*", "", val)
      gsub(/^["'"'"']|["'"'"']$/, "", val)
      if (val != "") { print val; exit }
    }
  ' "$file"
}

fm_body() {
  local file="$1"
  awk '/^---$/ { if (++fm==2) { body=1; next } next } body { print }' "$file"
}

# Counters
PASS=0
FAIL=0
ERRORS=()

# --- Assert functions ---

assert_eq() {
  local desc="$1" expected="$2" actual="$3"
  if [ "$expected" = "$actual" ]; then
    echo "  PASS: $desc"
    PASS=$((PASS + 1))
  else
    echo "  FAIL: $desc (expected '$expected', got '$actual')"
    FAIL=$((FAIL + 1))
    ERRORS+=("$desc")
  fi
}

assert_contains() {
  local desc="$1" haystack="$2" needle="$3"
  if echo "$haystack" | grep -qF "$needle"; then
    echo "  PASS: $desc"
    PASS=$((PASS + 1))
  else
    echo "  FAIL: $desc (does not contain '$needle')"
    FAIL=$((FAIL + 1))
    ERRORS+=("$desc")
  fi
}

assert_match() {
  local desc="$1" string="$2" regex="$3"
  if echo "$string" | grep -qE "$regex"; then
    echo "  PASS: $desc"
    PASS=$((PASS + 1))
  else
    echo "  FAIL: $desc ('$string' does not match /$regex/)"
    FAIL=$((FAIL + 1))
    ERRORS+=("$desc")
  fi
}

assert_file_exists() {
  local desc="$1" path="$2"
  if [ -f "$path" ]; then
    echo "  PASS: $desc"
    PASS=$((PASS + 1))
  else
    echo "  FAIL: $desc (file not found: $path)"
    FAIL=$((FAIL + 1))
    ERRORS+=("$desc")
  fi
}

assert_not_empty() {
  local desc="$1" value="$2"
  if [ -n "$value" ]; then
    echo "  PASS: $desc"
    PASS=$((PASS + 1))
  else
    echo "  FAIL: $desc (value is empty)"
    FAIL=$((FAIL + 1))
    ERRORS+=("$desc")
  fi
}

# --- Report ---

report() {
  local suite="${1:-tests}"
  echo ""
  echo "--- $suite ---"
  echo "  Passed: $PASS"
  echo "  Failed: $FAIL"
  if [ ${#ERRORS[@]} -gt 0 ]; then
    echo "  Failures:"
    for err in "${ERRORS[@]}"; do
      echo "    - $err"
    done
  fi
  echo ""
  if [ "$FAIL" -gt 0 ]; then
    return 1
  fi
  return 0
}
