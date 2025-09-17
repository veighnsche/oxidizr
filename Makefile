SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

.PHONY: test test-unit test-orch test-ci-local dev-shell full-smoke bdd bdd-arch bdd-core ci-local

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

bdd: bdd-core bdd-arch

bdd-arch:
	@echo "Running BDD tests for oxidizr-arch..."
	cargo test -p oxidizr-arch --features bdd --test bdd

bdd-core:
	@echo "Running BDD tests for oxidizr-cli-core..."
	cargo test -p oxidizr-cli-core --features bdd --test bdd

ci-local:
	@echo "[ci-local] fmt check"
	cargo fmt --all -- --check
	@echo "[ci-local] clippy (arch + core)"
	cargo clippy -p oxidizr-arch --all-targets --all-features -- -D warnings
	cargo clippy -p oxidizr-cli-core --all-targets --all-features -- -D warnings
	@echo "[ci-local] unit tests (workspace)"
	cargo test --workspace
	@echo "[ci-local] BDD (core)"
	cargo test -p oxidizr-cli-core --features bdd --test bdd
	@echo "[ci-local] BDD (arch)"
	cargo test -p oxidizr-arch --features bdd --test bdd
