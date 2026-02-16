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

Add forge-lib as a git submodule at `lib/`:

```bash
cd your-module/
git submodule add https://github.com/N4M3Z/forge-lib.git lib
```

Users cloning your module should use `--recurse-submodules`:

```bash
git clone --recurse-submodules https://github.com/N4M3Z/your-module.git
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

## Updating forge-lib

All Forge modules include forge-lib as a git submodule at `lib/`. When forge-lib is updated, each module must pull the new version:

```bash
# Inside a module directory
git submodule update --remote lib
git add lib
git commit -m "chore: update forge-lib submodule"
git push
```

Inside forge-core, update Core/lib and all modules at once:

```bash
# Update forge-core's copy
git submodule update --remote Core/lib
git add Core/lib
# Update each module's copy
for m in Modules/*/; do
  git -C "$m" submodule update --remote lib 2>/dev/null && \
  git -C "$m" add lib && \
  git -C "$m" diff --cached --quiet || \
  git -C "$m" commit -m "chore: update forge-lib submodule"
done
git add Modules/
git commit -m "chore: update forge-lib across all modules"
```

## Dependencies

- bash 4+ (macOS ships 3.2 but BSD awk is sufficient)
- awk (BSD or GNU)
- cargo (only if using `ensure-built.sh`)
- git (for submodule management)
