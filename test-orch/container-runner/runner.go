package main

import (
	"fmt"
	"log"

	"container-runner/assertions"
	"container-runner/setup"
	"container-runner/yamlrunner"
)

// runInContainer is the main entrypoint for the test orchestrator running inside the Docker container.
func runInContainer() error {
	log.Println("Starting Go orchestrator inside container...")

	if err := setup.Run(); err != nil {
		return fmt.Errorf("environment setup failed: %w", err)
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
