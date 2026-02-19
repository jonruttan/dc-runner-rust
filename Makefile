.PHONY: help build test smoke spec-sync spec-sync-check compat-check verify
.DEFAULT_GOAL := help
SOURCE ?=

help:
	@echo "build - cargo build"
	@echo "test  - cargo test"
	@echo "smoke - runner adapter help/smoke"
	@echo "spec-sync TAG=<upstream-tag> [SOURCE=<path-or-url>] - sync pinned upstream specs snapshot"
	@echo "spec-sync-check [SOURCE=<path-or-url>] - verify upstream lock/snapshot integrity"
	@echo "compat-check [SOURCE=<path-or-url>] - verify runner compatibility against pinned upstream snapshot"
	@echo "verify - build + test + spec-sync-check + compat-check"

build:
	cargo build --manifest-path spec_runner_cli/Cargo.toml

test:
	cargo test --manifest-path spec_runner_cli/Cargo.toml

smoke:
	./runner_adapter.sh governance --help || true

spec-sync:
	@test -n "$(TAG)" || (echo "ERROR: TAG is required (make spec-sync TAG=<upstream-tag>)" >&2; exit 2)
	@if [ -n "$(SOURCE)" ]; then \
		./scripts/sync_data_contracts_specs.sh --tag "$(TAG)" --source "$(SOURCE)"; \
	else \
		./scripts/sync_data_contracts_specs.sh --tag "$(TAG)"; \
	fi

spec-sync-check:
	@if [ -n "$(SOURCE)" ]; then \
		./scripts/sync_data_contracts_specs.sh --check --source "$(SOURCE)"; \
	else \
		./scripts/sync_data_contracts_specs.sh --check; \
	fi

compat-check:
	@if [ -n "$(SOURCE)" ]; then \
		./scripts/verify_upstream_compat.sh --strict --source "$(SOURCE)"; \
	else \
		./scripts/verify_upstream_compat.sh --strict; \
	fi

verify: build test spec-sync-check compat-check
