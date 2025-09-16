SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

.PHONY: test test-unit test-orch test-ci-local dev-shell full-smoke

test: test-unit test-orch

test-unit:
	@echo "Running unit tests..."
	cargo test

test-orch:
	@echo "Running orchestrated tests..."
	cd test-orch/host-orchestrator && sudo go run .

test-ci-local:
	@echo "Running local CI tests via host-orchestrator..."
	cd test-orch/host-orchestrator && sudo go run . --test-ci

dev-shell:
	@echo "Launching interactive Ubuntu dev shell with replacements applied..."
	bash scripts/ubuntu_dev_shell.sh

full-smoke:
	@echo "Running full destructive smoke in disposable Ubuntu container..."
	bash scripts/ubuntu_full_smoke.sh
