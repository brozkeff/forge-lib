# common.mk — shared variables and binary prerequisites for forge modules
#
# Include from modules after setting LIB_DIR:
#   LIB_DIR = $(or $(FORGE_LIB),lib)
#   include $(LIB_DIR)/mk/common.mk

LIB_DIR   ?= $(or $(FORGE_LIB),lib)
SCOPE     ?= workspace
SKILL_SRC ?= skills
AGENT_SRC ?= agents

# Skill destination directories (scope-aware)
CLAUDE_SKILLS_DST  ?= $(if $(filter workspace,$(SCOPE)),$(CURDIR)/.claude/skills,$(HOME)/.claude/skills)
CODEX_SKILLS_DST   ?= $(if $(filter workspace,$(SCOPE)),$(CURDIR)/.codex/skills,$(HOME)/.codex/skills)
OPENCODE_SKILLS_DST ?= $(if $(filter workspace,$(SCOPE)),$(CURDIR)/.opencode/skills,$(HOME)/.opencode/skills)

# Rust binaries from forge-lib submodule
INSTALL_AGENTS  ?= $(LIB_DIR)/bin/install-agents
INSTALL_SKILLS  ?= $(LIB_DIR)/bin/install-skills
VALIDATE_MODULE ?= $(LIB_DIR)/bin/validate-module
YAML_CLI        ?= $(LIB_DIR)/bin/yaml

# Binary prerequisite: build forge-lib when binaries are missing
$(INSTALL_AGENTS) $(INSTALL_SKILLS) $(VALIDATE_MODULE) $(YAML_CLI): init
	@$(MAKE) -C $(LIB_DIR) build

# Derive AGENTS and SKILLS from defaults.yaml.
# Priority: yq (system) → yaml (forge-lib) → empty (pass AGENTS/SKILLS explicitly).
ifneq ($(wildcard defaults.yaml),)
  AGENTS ?= $(shell { command -v yq >/dev/null 2>&1 \
    && yq -r '(.agents // {}) | keys | .[]' defaults.yaml \
    || $(YAML_CLI) keys defaults.yaml .agents; } 2>/dev/null | tr '\n' ' ')

  SKILLS ?= $(shell { command -v yq >/dev/null 2>&1 \
    && yq -r '(.skills.claude // {}) | keys | .[]' defaults.yaml \
    || $(YAML_CLI) keys defaults.yaml .skills.claude; } 2>/dev/null | tr '\n' ' ')
endif
