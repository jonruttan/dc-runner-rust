.PHONY: help build test smoke
.DEFAULT_GOAL := help

help:
	@echo "build - cargo build"
	@echo "test  - cargo test"
	@echo "smoke - runner adapter help/smoke"

build:
	cargo build --manifest-path spec_runner_cli/Cargo.toml

test:
	cargo test --manifest-path spec_runner_cli/Cargo.toml

smoke:
	./runner_adapter.sh governance --help || true
