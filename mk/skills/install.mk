# skills/install.mk — skill install and clean targets
#
# Requires: SKILLS, SKILL_SRC set before include
# Provides: install-skills, install-skills-{claude,gemini,codex,opencode}, clean-skills

.PHONY: install-skills install-skills-claude install-skills-gemini install-skills-codex install-skills-opencode
.PHONY: clean-skills

# Template for file-based providers (claude, codex, opencode)
define skill_provider_install
install-skills-$(1): $$(INSTALL_SKILLS)
	@if [ "$$(SCOPE)" = "all" ]; then \
	  $$(INSTALL_SKILLS) $$(SKILL_SRC) --provider $(1) --scope "$$(SCOPE)" --dst "$$(CURDIR)/.$(1)/skills"; \
	  $$(INSTALL_SKILLS) $$(SKILL_SRC) --provider $(1) --scope "$$(SCOPE)" --dst "$$(HOME)/.$(1)/skills"; \
	elif [ "$$(SCOPE)" = "workspace" ]; then \
	  $$(INSTALL_SKILLS) $$(SKILL_SRC) --provider $(1) --scope "$$(SCOPE)" --dst "$$(CURDIR)/.$(1)/skills"; \
	elif [ "$$(SCOPE)" = "user" ]; then \
	  $$(INSTALL_SKILLS) $$(SKILL_SRC) --provider $(1) --scope "$$(SCOPE)" --dst "$$(HOME)/.$(1)/skills"; \
	else \
	  echo "Error: Invalid SCOPE '$$(SCOPE)'. Use workspace, user, or all."; \
	  exit 1; \
	fi
endef

$(eval $(call skill_provider_install,claude))
$(eval $(call skill_provider_install,codex))
$(eval $(call skill_provider_install,opencode))

# Gemini uses its own CLI
install-skills-gemini: $(INSTALL_SKILLS)
	@if command -v gemini >/dev/null 2>&1; then \
	  $(INSTALL_SKILLS) $(SKILL_SRC) --provider gemini --scope "$(SCOPE)"; \
	else \
	  echo "  skip gemini skill install (gemini CLI not installed)"; \
	fi

install-skills: install-skills-claude install-skills-gemini install-skills-codex install-skills-opencode

# Clean installed skills from all provider directories
clean-skills:
	@for dir in .claude/skills .gemini/skills .codex/skills .opencode/skills; do \
	  for s in $(SKILLS); do \
	    command rm -rf "$$dir/$$s" 2>/dev/null || true; \
	  done; \
	done
	@echo "Cleaned installed skills."
