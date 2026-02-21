# lint.mk — linting targets for forge modules
#
# Requires: SKILL_SRC, AGENT_SRC set before include (from common.mk)
# Provides: lint-schema, lint-shell

.PHONY: lint-schema lint-shell

lint-schema:
	@if ! command -v mdschema >/dev/null 2>&1; then \
	  echo "  SKIP mdschema (not installed — brew install jackchuka/tap/mdschema)"; \
	else \
	  if [ -f $(SKILL_SRC)/.mdschema.yml ]; then \
	    echo "  skills ($(SKILL_SRC)/.mdschema.yml)"; \
	    mdschema check "$(SKILL_SRC)/*/SKILL.md" --schema $(SKILL_SRC)/.mdschema.yml; \
	  fi; \
	  if [ -f $(AGENT_SRC)/.mdschema.yml ]; then \
	    echo "  agents ($(AGENT_SRC)/.mdschema.yml)"; \
	    mdschema check "$(AGENT_SRC)/*.md" --schema $(AGENT_SRC)/.mdschema.yml; \
	  fi; \
	fi

lint-shell:
	@if find . -name '*.sh' -not -path '*/target/*' -not -path '*/lib/*' | grep -q .; then \
	  if command -v shellcheck >/dev/null 2>&1; then \
	    find . -name '*.sh' -not -path '*/target/*' -not -path '*/lib/*' -print0 | xargs -0 shellcheck -S warning; \
	  else \
	    echo "  SKIP shellcheck (not installed)"; \
	  fi; \
	fi
