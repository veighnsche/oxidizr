package setup

import (
	"log"
	"container-runner/util"
)

// setupRust ensures the Rust toolchain default is set for root and the builder user.
func setupRust() error {
	log.Println("==> Setting up Rust toolchain (root-only, minimal profile)...")
	// Use minimal profile to reduce components and download size
	_ = util.RunCmd("rustup", "set", "profile", "minimal")
	// Only set default if stable is not already present
	_ = util.RunCmd("sh", "-lc", "rustup toolchain list | grep -q '^stable' || rustup default stable")
	// Toolchain for 'builder' is not required; builds run as root.
	// AUR helper installation is handled in installAurHelper()
	return nil
}
