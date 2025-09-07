.PHONY: test test-unit test-orch test-ci-local

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
