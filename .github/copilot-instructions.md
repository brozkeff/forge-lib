# Copilot Instructions for forge-lib

## What This Project Does

forge-lib is the shared Rust library and CLI toolkit for the Forge ecosystem. It provides frontmatter parsing, agent deployment, skill installation, and module validation -- consumed as a git submodule (`lib/`) by other Forge modules. Not a standalone plugin.

## High-Level Architecture

### Library Modules (`src/`)

Six functional modules, each with a `mod.rs` + sibling `tests.rs`:

| Module | Key Functions | Purpose |
|--------|--------------|---------|
| `parse` | `fm_value`, `fm_body`, `fm_list`, `split_frontmatter` | YAML frontmatter extraction from markdown |
| `strip` | `strip_front`, `strip_front_keep` | Remove metadata and H1 headings for clean AI input |
| `sidecar` | `SidecarConfig::load`, `agent_value`, `skill_value` | Load agent/skill YAML configuration files |
| `deploy` | `deploy_agents_from_dir`, `clean_agents`, `scope_dirs` | Multi-provider agent deployment pipeline |
| `skill` | `plan_skills_from_dir`, `generate_skills_from_agents_dir` | Skill lifecycle and wrapper generation |
| `validate` | `validate_structure`, `validate_agent_frontmatter`, `validate_skills` | Convention enforcement test suites |

### CLI Binaries (`src/bin/`)

| Binary | Purpose |
|--------|---------|
| `strip-front` | Strip YAML frontmatter and H1 heading from markdown files |
| `install-agents` | Deploy agent markdown to Claude/Gemini/Codex/OpenCode directories |
| `install-skills` | Install skills with provider-specific routing and wrapper generation |
| `validate-module` | Convention test suite for forge modules (5 suites) |

All binaries support `--version`, `--help`, and all four providers.

## Build, Test, Lint

```bash
make build    # cargo build --release + symlink to bin/
make test     # cargo test (~240 tests)
make lint     # cargo fmt --check + clippy pedantic
make check    # verify bin/ symlinks
make clean    # cargo clean + rm bin/
```

## Key Conventions

- **Error handling**: `Option<T>` / `Result<T, String>` -- no custom error enums
- **Safety**: `#![forbid(unsafe_code)]` strictly enforced
- **I/O separation**: Library functions are pure (no I/O), binaries handle all file system operations
- **Clippy pedantic**: All warnings enabled
- **YAML**: `serde_yaml` for all parsing
- **Testing**: Unit tests in `src/<module>/tests.rs`, integration tests in `tests/`

## Submodule Consumption

Other Forge modules pull forge-lib as a git submodule:

```bash
git submodule add https://github.com/N4M3Z/forge-lib.git lib
make -C lib build
```

Makefile integration pattern:

```makefile
LIB_DIR = $(or $(FORGE_LIB),lib)
INSTALL_AGENTS := $(LIB_DIR)/bin/install-agents

$(INSTALL_AGENTS):
	@$(MAKE) -C $(LIB_DIR) build
```

## File Organization

```
src/                     # Library modules + binary entry points
  parse/                 # Frontmatter parsing
  strip/                 # Markdown stripping
  sidecar/               # YAML config loading
  deploy/                # Agent deployment pipeline
  skill/                 # Skill installation planning
  validate/              # Convention validation suites
  bin/                   # CLI binary entry points
tests/                   # Integration tests
bin/                     # Symlinked binaries (make build)
Cargo.toml               # Rust crate manifest
Makefile                 # Build orchestration
module.yaml              # Forge module metadata
```

## Git Conventions

Conventional Commits: `type: description`. Lowercase, no trailing period, no scope.

Types: `feat`, `fix`, `docs`
