# forge-lib

Shared shell utilities for Forge modules. Provides frontmatter parsing, lazy Rust compilation, agent deployment, and markdown processing.

## Usage

### Inside forge-core

Already available via `FORGE_LIB` env var (set by dispatch):

```bash
source "$FORGE_LIB/frontmatter.sh"
value="$(fm_value file.md "claude.name")"
```

### Standalone module

Clone directly into the module's `lib/` directory:

```bash
cd your-module/
git clone https://github.com/<user>/forge-lib.git lib
```

The `lib/` directory IS forge-lib — no nesting. Then in your scripts:

```bash
MODULE_ROOT="$(builtin cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FORGE_LIB="${FORGE_LIB:-$MODULE_ROOT/lib}"

if [ ! -d "$FORGE_LIB" ]; then
  echo "forge-lib not found. Run: git clone <url> $MODULE_ROOT/lib" >&2
  exit 1
fi

source "$FORGE_LIB/frontmatter.sh"
```

## Utilities

| File | Functions | Purpose |
|------|-----------|---------|
| `frontmatter.sh` | `fm_value`, `fm_body` | Parse YAML frontmatter from markdown files |
| `ensure-built.sh` | `ensure_built` | Lazy Rust compilation (cargo build on first use) |
| `install-agents.sh` | `deploy_agent`, `deploy_agents_from_dir` | Deploy agent markdown files to `~/.claude/agents/` |
| `strip-front.sh` | `strip_front` | Strip YAML frontmatter and H1 heading from markdown |

## Dependencies

- bash 4+ (macOS ships 3.2 but BSD awk is sufficient)
- awk (BSD or GNU)
- cargo (only if using `ensure-built.sh`)
- git (only for standalone clone)
