package setup

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"container-runner/util"
)

// buildProject compiles oxidizr-arch and installs the binary into PATH inside the container.
func buildProject() error {
	projectDir := "/workspace"
	
	// Always build using the project root Cargo.toml (src-2 has been renamed to src)
	cargoToml := filepath.Join(projectDir, "Cargo.toml")
	binaryName := "oxidizr-arch"
	stampPath := "/usr/local/bin/.oxidizr_build_hash"
	
	if _, err := os.Stat(cargoToml); os.IsNotExist(err) {
		return fmt.Errorf("Cargo.toml not found at %s", cargoToml)
	}

	originalDir, _ := os.Getwd()
	_ = os.Chdir(projectDir)
	defer os.Chdir(originalDir)

	// Build stamp: skip if current git commit hash matches stamp and binary exists,
    // unless forced via FORCE_RUST_REBUILD=1
    force := os.Getenv("FORCE_RUST_REBUILD") == "1"
    var currentHash string
    if err := util.RunCmdQuiet("git", "rev-parse", "HEAD"); err == nil {
        // Capture via shell to get output
        // Using sh -lc to capture command output into a file for comparison
        _ = util.RunCmd("sh", "-lc", "git rev-parse HEAD > /tmp/.cur_hash 2>/dev/null")
        if b, err2 := os.ReadFile("/tmp/.cur_hash"); err2 == nil {
            currentHash = strings.TrimSpace(string(b))
        }
    }
    
    destPath := fmt.Sprintf("/usr/local/bin/%s", binaryName)
    if !force && currentHash != "" {
        if b, err := os.ReadFile(stampPath); err == nil && strings.TrimSpace(string(b)) == currentHash {
            if _, err2 := os.Stat(destPath); err2 == nil {
                // Up-to-date; skip rebuild
                return util.RunCmdQuiet(binaryName, "--help")
            }
        }
    }

	buildJobs := os.Getenv("CARGO_BUILD_JOBS")
	if buildJobs == "" {
		buildJobs = "2"
	}
	if err := util.RunCmd("cargo", "build", "--release", "-j", buildJobs); err != nil {
		return fmt.Errorf("cargo build failed: %w", err)
	}

	sourcePath := fmt.Sprintf("target/release/%s", binaryName)
	if err := util.RunCmd("install", "-m", "0755", sourcePath, destPath); err != nil {
		return fmt.Errorf("failed to install binary: %w", err)
	}
	
	// Write new stamp if we have the hash
	if currentHash != "" {
		_ = os.WriteFile(stampPath, []byte(currentHash+"\n"), 0644)
	}

	return util.RunCmdQuiet(binaryName, "--help")
}
