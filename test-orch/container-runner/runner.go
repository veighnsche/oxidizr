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
		log.Printf("❌ Setup failed: %v", err)
		return fmt.Errorf("environment setup failed: %w", err)
	}
	log.Println("✅ Setup complete")

	// Always run Rust unit tests by default as part of the matrix run
	log.Println("==> Running Rust unit tests (cargo test)...")
	if err := util.RunCmd("sh", "-lc", "cd /workspace && cargo test -q"); err != nil {
		log.Printf("❌ Rust unit tests failed: %v", err)
		return fmt.Errorf("rust unit tests failed: %w", err)
	}
	log.Println("✅ Rust unit tests passed")

	log.Println("==> Running YAML test suites...")
	if err := yamlrunner.Run(); err != nil {
		log.Printf("❌ YAML test suites failed: %v", err)
		return fmt.Errorf("YAML test suites failed: %w", err)
	}
	log.Println("✅ YAML test suites passed")

	if err := assertions.Run(); err != nil {
		log.Printf("❌ Assertions failed: %v", err)
		return fmt.Errorf("assertions failed: %w", err)
	}
	log.Println("✅ Assertions passed")

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
