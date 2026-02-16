# Verification

## Quick check

Source each utility and verify functions are defined:

```bash
FORGE_LIB="$(pwd)"

# frontmatter.sh
source "$FORGE_LIB/frontmatter.sh"
type fm_value   # should print "fm_value is a function"
type fm_body    # should print "fm_body is a function"

# strip-front.sh
source "$FORGE_LIB/strip-front.sh"
type strip_front  # should print "strip_front is a function"

# install-agents.sh (requires frontmatter.sh first)
source "$FORGE_LIB/install-agents.sh"
type deploy_agent           # should print "deploy_agent is a function"
type deploy_agents_from_dir # should print "deploy_agents_from_dir is a function"

# ensure-built.sh (requires PLUGIN_ROOT)
PLUGIN_ROOT="/tmp" source "$FORGE_LIB/ensure-built.sh"
type ensure_built  # should print "ensure_built is a function"
```

## Functional test

Create a test agent file and verify deployment:

```bash
FORGE_LIB="$(pwd)"
source "$FORGE_LIB/frontmatter.sh"
source "$FORGE_LIB/install-agents.sh"

# Create temp agent
TMPDIR="$(mktemp -d)"
cat > "$TMPDIR/test-agent.md" << 'EOF'
---
title: Test Agent
claude.name: test-verify-agent
claude.model: haiku
claude.description: Verification test agent
claude.tools: Read, Grep
---

Test body content.
EOF

# Deploy (dry-run)
deploy_agent "$TMPDIR/test-agent.md" "$TMPDIR/output" --dry-run
# Expected: [dry-run] Would install: test-verify-agent.md

# Deploy (real)
mkdir -p "$TMPDIR/output"
deploy_agent "$TMPDIR/test-agent.md" "$TMPDIR/output"
# Expected: Installed: test-verify-agent.md

# Verify output
cat "$TMPDIR/output/test-verify-agent.md"
# Expected: frontmatter with name/description/model/tools + synced-from marker + body

# Cleanup
command rm -rf "$TMPDIR"
```

## Frontmatter parsing test

```bash
FORGE_LIB="$(pwd)"
source "$FORGE_LIB/frontmatter.sh"

TMPDIR="$(mktemp -d)"
cat > "$TMPDIR/test.md" << 'EOF'
---
title: My Note
claude.name: test-name
claude.model: sonnet
description: "A quoted value"
---

Body content here.
EOF

[ "$(fm_value "$TMPDIR/test.md" "claude.name")" = "test-name" ] && echo "PASS: fm_value" || echo "FAIL: fm_value"
[ "$(fm_value "$TMPDIR/test.md" "description")" = "A quoted value" ] && echo "PASS: quoted value" || echo "FAIL: quoted value"
[ "$(fm_body "$TMPDIR/test.md" | head -1)" = "" ] && echo "PASS: fm_body (blank line)" || echo "FAIL: fm_body"
[ "$(fm_body "$TMPDIR/test.md" | tail -1)" = "Body content here." ] && echo "PASS: fm_body (content)" || echo "FAIL: fm_body"

command rm -rf "$TMPDIR"
```
