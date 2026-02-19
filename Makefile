.PHONY: help build test smoke spec-sync spec-sync-check compat-check verify
.DEFAULT_GOAL := help
SOURCE ?= https://github.com/jonruttan/data-contracts.git

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
	./scripts/sync_data_contracts_specs.sh --tag "$(TAG)" --source "$(SOURCE)"

spec-sync-check:
	./scripts/sync_data_contracts_specs.sh --check --source "$(SOURCE)"

compat-check:
	./scripts/verify_upstream_compat.sh --strict --source "$(SOURCE)"

verify: build test spec-sync-check compat-check
