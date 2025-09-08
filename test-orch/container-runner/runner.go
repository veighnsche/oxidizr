package main

import (
	"fmt"
	"log"
	"os"

	"container-runner/assertions"
	"container-runner/analytics"
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
	// Write analytics report to the mounted workspace so it persists on host
	// Include distro suffix when provided (ANALYTICS_DISTRO is set by host orchestrator)
	reportPath := "/workspace/TEST_DOWNLOADS_ANALYTICS.md"
	if d := os.Getenv("ANALYTICS_DISTRO"); d != "" {
		reportPath = fmt.Sprintf("/workspace/TEST_DOWNLOADS_ANALYTICS-%s.md", d)
	}
	if err := analytics.WriteReportMarkdown(reportPath); err != nil {
		log.Printf("warning: failed to write download analytics: %v", err)
	} else {
		log.Printf("Download analytics written to %s", reportPath)
	}
	return nil
}
