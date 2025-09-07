package setup

import (
	"fmt"
	"os"
	"path/filepath"

	"container-runner/util"
)

// buildProject compiles oxidizr-arch and installs the binary into PATH inside the container.
func buildProject() error {
	projectDir := "/root/project/oxidizr-arch"
	if _, err := os.Stat(filepath.Join(projectDir, "Cargo.toml")); os.IsNotExist(err) {
		return fmt.Errorf("Cargo.toml not found in %s", projectDir)
	}

	originalDir, _ := os.Getwd()
	_ = os.Chdir(projectDir)
	defer os.Chdir(originalDir)

	buildJobs := os.Getenv("CARGO_BUILD_JOBS")
	if buildJobs == "" {
		buildJobs = "2"
	}
	if err := util.RunCmd("cargo", "build", "--release", "-j", buildJobs); err != nil {
		return fmt.Errorf("cargo build failed: %w", err)
	}

	sourcePath := filepath.Join(projectDir, "target/release/oxidizr-arch")
	destPath := "/usr/local/bin/oxidizr-arch"
	if err := util.RunCmd("install", "-m", "0755", sourcePath, destPath); err != nil {
		return fmt.Errorf("failed to install binary: %w", err)
	}

	return util.RunCmdQuiet("oxidizr-arch", "--help")
}
