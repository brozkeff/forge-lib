# agents/install.mk — agent install and clean targets
#
# Requires: AGENTS, AGENT_SRC set before include
# Provides: install-agents, clean-agents

.PHONY: install-agents clean-agents

install-agents: $(INSTALL_AGENTS)
	@$(INSTALL_AGENTS) $(AGENT_SRC) --scope "$(SCOPE)"

clean-agents: $(INSTALL_AGENTS)
	@$(INSTALL_AGENTS) $(AGENT_SRC) --clean
