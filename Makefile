.PHONY: test test-unit test-orch

test: test-unit test-orch

test-unit:
	@echo "Running unit tests..."
	cargo test

test-orch:
	@echo "Running orchestrated tests..."
	cd test-orch/host-orchestrator && sudo go run .
