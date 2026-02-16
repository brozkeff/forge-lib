# Installation

## As part of forge-core

No installation needed. forge-core's dispatch binary exports `FORGE_LIB` pointing to this directory. All modules can source utilities directly:

```bash
source "$FORGE_LIB/frontmatter.sh"
```

## Standalone (for independent modules)

Clone directly into the module's `lib/` directory — `lib/` IS forge-lib:

```bash
cd your-module/
git clone https://github.com/<user>/forge-lib.git lib
```

Add `lib/` to your module's `.gitignore` — it's a runtime dependency, not committed.

Then resolve in your scripts:

```bash
FORGE_LIB="${FORGE_LIB:-$MODULE_ROOT/lib}"
```

This checks the forge-core env var first, falls back to the local clone.

## Updating

```bash
cd lib && git pull
```

Or if using forge-core, update the main repo (forge-lib is part of the tree).
