# skills/verify.mk — skill verification targets
#
# Requires: SKILLS set before include
# Provides: verify-skills, verify-skills-{claude,gemini,codex,opencode}

.PHONY: verify-skills verify-skills-claude verify-skills-gemini verify-skills-codex verify-skills-opencode

# Template for file-based providers (claude, codex, opencode)
define skill_provider_verify
verify-skills-$(1):
	@missing=0; \
	if [ "$$(SCOPE)" = "all" ]; then \
	  for dst in "$$(CURDIR)/.$(1)/skills" "$$(HOME)/.$(1)/skills"; do \
	    echo "Verifying $(1) skills in $$$$dst..."; \
	    for s in $$(SKILLS); do \
	      if test -f "$$$$dst/$$$$s/SKILL.md"; then echo "  ok $$$$s"; \
	      else echo "  missing $$$$s"; missing=1; fi; \
	    done; \
	  done; \
	else \
	  _dst=$$$$(if [ "$$(SCOPE)" = "workspace" ]; then echo "$$(CURDIR)/.$(1)/skills"; else echo "$$(HOME)/.$(1)/skills"; fi); \
	  echo "Verifying $(1) skills in $$$$_dst..."; \
	  for s in $$(SKILLS); do \
	    if test -f "$$$$_dst/$$$$s/SKILL.md"; then echo "  ok $$$$s"; \
	    else echo "  missing $$$$s"; missing=1; fi; \
	  done; \
	fi; \
	test $$$$missing -eq 0
endef

$(eval $(call skill_provider_verify,claude))
$(eval $(call skill_provider_verify,codex))
$(eval $(call skill_provider_verify,opencode))

# Gemini uses its own CLI
verify-skills-gemini:
	@if command -v gemini >/dev/null 2>&1; then \
	  echo "Verifying Gemini skills via CLI..."; \
	  out_file=$$(mktemp); \
	  if gemini skills list > "$$out_file" 2>&1; then \
	    for s in $(SKILLS); do \
	      if grep -q "$$s" "$$out_file"; then echo "  ok $$s"; fi; \
	    done; \
	  else \
	    if [ "$${GEMINI_VERIFY_STRICT:-0}" = "1" ]; then \
	      cat "$$out_file"; command rm -f "$$out_file"; exit 1; \
	    fi; \
	    echo "  skip gemini skill verification (non-interactive or unauthenticated)"; \
	  fi; \
	  command rm -f "$$out_file"; \
	else \
	  echo "  skip gemini skill verification (gemini CLI not installed)"; \
	fi

verify-skills: verify-skills-claude verify-skills-gemini verify-skills-codex verify-skills-opencode
