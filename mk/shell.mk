# shell.mk — targets for shell-only forge modules (no Rust, no skills, no agents)
#
# Provides: test, lint, check

.PHONY: test lint check

test:
	@if [ -f tests/test.sh ]; then bash tests/test.sh; else echo "No tests defined"; fi

lint:
	@if find . -name '*.sh' -not -path '*/target/*' | grep -q .; then \
	  find . -name '*.sh' -not -path '*/target/*' | xargs shellcheck -S warning 2>/dev/null || true; \
	fi

check:
	@test -f module.yaml && echo "  ok module.yaml" || echo "  MISSING module.yaml"
	@test -d hooks && echo "  ok hooks/" || echo "  MISSING hooks/"
