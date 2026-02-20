# agents/verify.mk — agent verification targets
#
# Requires: AGENTS set before include
# Provides: verify-agents

.PHONY: verify-agents

verify-agents:
	@missing=0; \
	for provider in claude gemini codex opencode; do \
	  if [ "$(SCOPE)" = "all" ]; then \
	    dirs="$(CURDIR)/.$$provider/agents $(HOME)/.$$provider/agents"; \
	  elif [ "$(SCOPE)" = "workspace" ]; then \
	    dirs="$(CURDIR)/.$$provider/agents"; \
	  elif [ "$(SCOPE)" = "user" ]; then \
	    dirs="$(HOME)/.$$provider/agents"; \
	  else \
	    echo "Invalid SCOPE: $(SCOPE)"; exit 1; \
	  fi; \
	  for dst in $$dirs; do \
	    echo "Verifying $$provider agents in $$dst..."; \
	    for a in $(AGENTS); do \
	      if test -f "$$dst/$$a.md" || test -f "$$dst/$$a.toml"; then \
	        echo "  ok $$a"; \
	      else \
	        echo "  missing $$a"; \
	        missing=1; \
	      fi; \
	    done; \
	  done; \
	done; \
	if [ $$missing -ne 0 ]; then \
	  echo "Run 'make install' to deploy agents."; \
	fi; \
	test $$missing -eq 0
