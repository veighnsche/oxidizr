package main

import (
	"fmt"
	"log"

	"container-runner/assertions"
	"container-runner/setup"
	"container-runner/util"
	"container-runner/yamlrunner"
)

// runInContainer is the main entrypoint for the test orchestrator running inside the Docker container.
func runInContainer() error {
	log.Println("Starting Go orchestrator inside container...")

	if err := setup.Run(); err != nil {
		return fmt.Errorf("environment setup failed: %w", err)
	}

	// Always run Rust unit tests by default as part of the matrix run
	log.Println("==> Running Rust unit tests (cargo test)...")
	if err := util.RunCmd("sh", "-lc", "cd /workspace && cargo test -q"); err != nil {
		return fmt.Errorf("rust unit tests failed: %w", err)
	}

	log.Println("==> Running YAML test suites...")
	if err := yamlrunner.Run(); err != nil {
		return fmt.Errorf("YAML test suites failed: %w", err)
	}

	if err := assertions.Run(); err != nil {
		return fmt.Errorf("assertions failed: %w", err)
	}

	log.Println("Go orchestrator finished successfully.")
	return nil
}
